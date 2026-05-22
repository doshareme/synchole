//! Sync protocol, conflict resolution, and replication orchestration.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use synchole_core::{
    Causality, CollectionId, ObjectId, PeerId, Result, SessionId, SyncholeError, Timestamp,
    VersionVector,
};
use synchole_discovery::PeerDiscovery;
use synchole_identity::IdentityProvider;
use synchole_storage::{ObjectKey, StoredObject, SyncStore};
use synchole_transport::{
    ConnectionPolicy, PacketClass, SecureTransport, TransportFrame, TransportPath,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncScope {
    pub collections: Vec<CollectionId>,
    pub include_tombstones: bool,
    pub max_changes: usize,
}

impl Default for SyncScope {
    fn default() -> Self {
        Self {
            collections: Vec::new(),
            include_tombstones: true,
            max_changes: 512,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReport {
    pub peers_discovered: usize,
    pub peers_contacted: usize,
    pub peers_failed: usize,
    pub objects_sent: usize,
    pub objects_received: usize,
    pub conflicts_detected: usize,
    pub selected_paths: Vec<TransportPath>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SyncMessage {
    Hello {
        peer_id: PeerId,
        supported_versions: Vec<u16>,
    },
    Have {
        scope: SyncScope,
        heads: Vec<(ObjectKey, VersionVector)>,
    },
    Need {
        keys: Vec<ObjectKey>,
    },
    Object {
        object: StoredObject,
    },
    Ack {
        applied: Vec<ObjectKey>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    pub key: ObjectKey,
    pub local: StoredObject,
    pub remote: StoredObject,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Resolution {
    KeepLocal,
    KeepRemote,
    Merge(StoredObject),
    KeepBoth(Vec<StoredObject>),
}

#[async_trait]
pub trait ConflictResolver: Send + Sync {
    async fn resolve(&self, conflict: Conflict) -> Result<Resolution>;
}

#[derive(Clone, Debug, Default)]
pub struct LastWriterWinsResolver;

#[async_trait]
impl ConflictResolver for LastWriterWinsResolver {
    async fn resolve(&self, conflict: Conflict) -> Result<Resolution> {
        if conflict.remote.updated_at >= conflict.local.updated_at {
            Ok(Resolution::KeepRemote)
        } else {
            Ok(Resolution::KeepLocal)
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct KeepBothResolver;

#[async_trait]
impl ConflictResolver for KeepBothResolver {
    async fn resolve(&self, conflict: Conflict) -> Result<Resolution> {
        let mut local = conflict.local;
        let mut remote = conflict.remote;
        local.key.object_id = ObjectId::from(format!("{}@local", local.key.object_id));
        remote.key.object_id = ObjectId::from(format!("{}@remote", remote.key.object_id));
        Ok(Resolution::KeepBoth(vec![local, remote]))
    }
}

pub struct SyncEngine {
    store: Arc<dyn SyncStore>,
    discovery: Option<Arc<dyn PeerDiscovery>>,
    transport: Option<Arc<dyn SecureTransport>>,
    identity: Option<Arc<dyn IdentityProvider>>,
    resolver: Arc<dyn ConflictResolver>,
}

impl SyncEngine {
    pub fn new(store: Arc<dyn SyncStore>) -> Self {
        Self {
            store,
            discovery: None,
            transport: None,
            identity: None,
            resolver: Arc::new(LastWriterWinsResolver),
        }
    }

    pub fn with_discovery(mut self, discovery: Arc<dyn PeerDiscovery>) -> Self {
        self.discovery = Some(discovery);
        self
    }

    pub fn with_transport(mut self, transport: Arc<dyn SecureTransport>) -> Self {
        self.transport = Some(transport);
        self
    }

    pub fn with_identity(mut self, identity: Arc<dyn IdentityProvider>) -> Self {
        self.identity = Some(identity);
        self
    }

    pub fn with_conflict_resolver(mut self, resolver: Arc<dyn ConflictResolver>) -> Self {
        self.resolver = resolver;
        self
    }

    pub async fn sync_once(&self, scope: SyncScope) -> Result<SyncReport> {
        let mut report = SyncReport::default();
        let (Some(discovery), Some(transport), Some(identity)) =
            (&self.discovery, &self.transport, &self.identity)
        else {
            return Ok(report);
        };

        let local_identity = identity.current_device().await?;
        let peers = discovery.list_peers(&local_identity.user_id).await?;
        report.peers_discovered = peers.len();

        for peer in peers {
            let policy = ConnectionPolicy::default();
            let path = transport.best_path(&peer, policy.clone()).await;
            let connection = match transport.connect(&peer, policy).await {
                Ok(connection) => connection,
                Err(_) => {
                    report.peers_failed = report.peers_failed.saturating_add(1);
                    continue;
                }
            };

            if let Ok(path) = path {
                report.selected_paths.push(path.selected_path);
            }

            let hello = SyncMessage::Hello {
                peer_id: peer.peer_id.clone(),
                supported_versions: vec![1],
            };
            let payload =
                serde_json::to_vec(&hello).map_err(|err| SyncholeError::Sync(err.to_string()))?;
            connection
                .send(TransportFrame {
                    session_id: SessionId::from("sync-once"),
                    stream_id: 0,
                    sequence: 0,
                    class: PacketClass::Control,
                    created_at: Timestamp::default(),
                    payload: Bytes::from(payload),
                })
                .await?;

            let page = self.store.scan_changes(None, scope.max_changes).await?;
            report.objects_sent = report.objects_sent.saturating_add(page.records.len());
            report.peers_contacted = report.peers_contacted.saturating_add(1);
        }

        Ok(report)
    }

    pub async fn apply_remote_object(&self, remote: StoredObject) -> Result<ApplyReport> {
        let local = self.store.get_object(&remote.key).await?;
        match local {
            None => {
                self.store.put_object(remote).await?;
                Ok(ApplyReport::Applied)
            }
            Some(local) => match local.version.compare(&remote.version) {
                Causality::Equal | Causality::After => Ok(ApplyReport::Skipped),
                Causality::Before => {
                    self.store.put_object(remote).await?;
                    Ok(ApplyReport::Applied)
                }
                Causality::Concurrent => {
                    let key = remote.key.clone();
                    let remote_candidate = remote.clone();
                    let resolution = self
                        .resolver
                        .resolve(Conflict { key, local, remote })
                        .await?;
                    self.apply_resolution(resolution, remote_candidate).await
                }
            },
        }
    }

    async fn apply_resolution(
        &self,
        resolution: Resolution,
        remote_candidate: StoredObject,
    ) -> Result<ApplyReport> {
        match resolution {
            Resolution::KeepLocal => Ok(ApplyReport::ConflictKeptLocal),
            Resolution::KeepRemote => {
                self.store.put_object(remote_candidate).await?;
                Ok(ApplyReport::ConflictKeptRemote)
            }
            Resolution::Merge(object) => {
                self.store.put_object(object).await?;
                Ok(ApplyReport::ConflictMerged)
            }
            Resolution::KeepBoth(objects) => {
                for object in objects {
                    self.store.put_object(object).await?;
                }
                Ok(ApplyReport::ConflictKeptBoth)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyReport {
    Applied,
    Skipped,
    ConflictKeptLocal,
    ConflictKeptRemote,
    ConflictMerged,
    ConflictKeptBoth,
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use synchole_storage::{StoragePayload, StoredObject};

    use super::*;

    fn object(device: &str, wall_time_ms: u64) -> StoredObject {
        let mut version = VersionVector::new();
        version.increment(device.into());
        StoredObject::new(
            "notes".into(),
            "1".into(),
            StoragePayload::Document(json!({"title": device})),
            version,
            Timestamp {
                wall_time_ms,
                logical: 0,
            },
        )
    }

    #[tokio::test]
    async fn last_writer_wins_selects_newer_remote() {
        let resolver = LastWriterWinsResolver;
        let conflict = Conflict {
            key: ObjectKey::new("notes".into(), "1".into()),
            local: object("a", 1),
            remote: object("b", 2),
        };

        assert_eq!(
            resolver.resolve(conflict).await.expect("resolve"),
            Resolution::KeepRemote
        );
    }
}
