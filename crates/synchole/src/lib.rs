//! Public SDK facade for applications and platform bindings.

use std::sync::Arc;

pub use synchole_core::{
    BatteryPolicy, CollectionId, ConnectionPreference, DeviceId, HybridLogicalClock, ObjectId,
    PeerId, Platform, Result, SessionId, StorageMode, SyncholeConfig, SyncholeError, Timestamp,
    TransportPolicy, UserId, VersionVector,
};
pub use synchole_discovery::{
    DiscoveryAnnouncement, DiscoveryBackend, EndpointTransport, PeerDescriptor, PeerDiscovery,
    PeerEndpoint, PeerPresence,
};
pub use synchole_identity::{
    AllowListTrustPolicy, DeviceIdentity, EnrollmentToken, IdentityProvider, KeyAlgorithm, Keyring,
    PublicKeyBytes, SignatureBytes, TrustPolicy,
};
pub use synchole_nat::{
    Candidate, CandidateKind, CandidateProtocol, HolePunchPlan, NatReport, NatTraversal, NatType,
};
pub use synchole_relay::{RelayClient, RelayPolicy, RelayRegion, RelayReservation};
#[cfg(feature = "native-sqlite")]
pub use synchole_storage::SQLiteStore;
pub use synchole_storage::{
    ChangeCursor, ChangePage, ChangeRecord, Migration, MigrationPlan, ObjectKey, RelationalRow,
    SqlValue, StorageConfig, StoragePayload, StoredObject, SyncStore,
};
pub use synchole_sync::{
    ApplyReport, Conflict, ConflictResolver, KeepBothResolver, LastWriterWinsResolver, Resolution,
    SyncEngine, SyncReport, SyncScope,
};
pub use synchole_transport::{
    CipherSuite, ConnectionPolicy, IncomingConnectionHandler, PacketClass, PathSelection,
    PeerConnection, SecureTransport, SessionCrypto, TransportFrame, TransportMetrics,
    TransportPath,
};

pub struct Synchole {
    config: SyncholeConfig,
    store: Arc<dyn SyncStore>,
    engine: SyncEngine,
}

impl Synchole {
    pub fn builder() -> SyncholeBuilder {
        SyncholeBuilder::default()
    }

    pub fn config(&self) -> &SyncholeConfig {
        &self.config
    }

    pub async fn put_binary(
        &self,
        collection: CollectionId,
        object_id: ObjectId,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let object = StoredObject::new(
            collection,
            object_id,
            StoragePayload::Binary {
                codec: "application/octet-stream".to_owned(),
                bytes,
            },
            VersionVector::new(),
            Timestamp::default(),
        );
        self.store.put_object(object).await
    }

    pub async fn put_document(
        &self,
        collection: CollectionId,
        object_id: ObjectId,
        document: serde_json::Value,
    ) -> Result<()> {
        let object = StoredObject::new(
            collection,
            object_id,
            StoragePayload::Document(document),
            VersionVector::new(),
            Timestamp::default(),
        );
        self.store.put_object(object).await
    }

    pub async fn get_object(
        &self,
        collection: CollectionId,
        object_id: ObjectId,
    ) -> Result<Option<StoredObject>> {
        self.store
            .get_object(&ObjectKey::new(collection, object_id))
            .await
    }

    pub async fn apply_remote_object(&self, object: StoredObject) -> Result<ApplyReport> {
        self.engine.apply_remote_object(object).await
    }

    pub async fn sync_once(&self) -> Result<SyncReport> {
        self.engine
            .sync_once(SyncScope {
                max_changes: self.config.sync_batch_size,
                ..SyncScope::default()
            })
            .await
    }
}

#[derive(Default)]
pub struct SyncholeBuilder {
    config: SyncholeConfig,
    storage: Option<Arc<dyn SyncStore>>,
    discovery: Option<Arc<dyn PeerDiscovery>>,
    transport: Option<Arc<dyn SecureTransport>>,
    identity: Option<Arc<dyn IdentityProvider>>,
    resolver: Option<Arc<dyn ConflictResolver>>,
}

impl SyncholeBuilder {
    pub fn with_config(mut self, config: SyncholeConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_storage(mut self, storage: Arc<dyn SyncStore>) -> Self {
        self.storage = Some(storage);
        self
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
        self.resolver = Some(resolver);
        self
    }

    pub async fn build(self) -> Result<Synchole> {
        let storage = self.storage.ok_or_else(|| {
            SyncholeError::InvalidInput("SyncholeBuilder requires a storage backend".to_owned())
        })?;

        storage
            .configure(StorageConfig::new(self.config.storage_mode.clone()))
            .await?;

        let mut engine = SyncEngine::new(storage.clone());
        if let Some(discovery) = self.discovery {
            engine = engine.with_discovery(discovery);
        }
        if let Some(transport) = self.transport {
            engine = engine.with_transport(transport);
        }
        if let Some(identity) = self.identity {
            engine = engine.with_identity(identity);
        }
        if let Some(resolver) = self.resolver {
            engine = engine.with_conflict_resolver(resolver);
        }

        Ok(Synchole {
            config: self.config,
            store: storage,
            engine,
        })
    }
}
