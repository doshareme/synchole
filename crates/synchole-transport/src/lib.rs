//! Secure transport, tunnel selection, and encrypted frame contracts.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use synchole_core::{PeerId, Result, SessionId, Timestamp};
use synchole_discovery::PeerDescriptor;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherSuite {
    NoiseX25519ChaChaPolyBlake3,
    Tls13AesGcm,
    WebRtcDtlsSrtp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportPath {
    DirectUdp,
    NatTraversalUdp,
    DirectTcp,
    WebRtcDataChannel,
    Relay,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PacketClass {
    Control,
    SyncDelta,
    SnapshotChunk,
    KeepAlive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportFrame {
    pub session_id: SessionId,
    pub stream_id: u64,
    pub sequence: u64,
    pub class: PacketClass,
    pub created_at: Timestamp,
    pub payload: Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionPolicy {
    pub preferred_paths: Vec<TransportPath>,
    pub max_frame_bytes: usize,
    pub require_forward_secrecy: bool,
}

impl Default for ConnectionPolicy {
    fn default() -> Self {
        Self {
            preferred_paths: vec![
                TransportPath::DirectUdp,
                TransportPath::NatTraversalUdp,
                TransportPath::DirectTcp,
                TransportPath::WebRtcDataChannel,
                TransportPath::Relay,
            ],
            max_frame_bytes: 1_048_576,
            require_forward_secrecy: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathSelection {
    pub peer_id: PeerId,
    pub selected_path: TransportPath,
    pub expected_rtt_ms: Option<u32>,
    pub relay_region: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportMetrics {
    pub path: TransportPath,
    pub sent_bytes: u64,
    pub received_bytes: u64,
    pub rtt_ms: Option<u32>,
    pub retransmits: u64,
}

#[async_trait]
pub trait PeerConnection: Send + Sync {
    fn peer_id(&self) -> PeerId;
    fn path(&self) -> TransportPath;
    async fn send(&self, frame: TransportFrame) -> Result<()>;
    async fn receive(&self) -> Result<Option<TransportFrame>>;
    async fn close(&self) -> Result<()>;
    async fn metrics(&self) -> Result<TransportMetrics>;
}

#[async_trait]
pub trait IncomingConnectionHandler: Send + Sync {
    async fn on_connection(&self, connection: Arc<dyn PeerConnection>) -> Result<()>;
}

#[async_trait]
pub trait SecureTransport: Send + Sync {
    async fn connect(
        &self,
        peer: &PeerDescriptor,
        policy: ConnectionPolicy,
    ) -> Result<Arc<dyn PeerConnection>>;

    async fn listen(&self, handler: Arc<dyn IncomingConnectionHandler>) -> Result<()>;
    async fn best_path(
        &self,
        peer: &PeerDescriptor,
        policy: ConnectionPolicy,
    ) -> Result<PathSelection>;
}

pub trait SessionCrypto: Send + Sync {
    fn cipher_suite(&self) -> CipherSuite;
    fn seal(&self, sequence: u64, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>;
    fn open(&self, sequence: u64, aad: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_prefers_direct_before_relay() {
        let policy = ConnectionPolicy::default();

        assert_eq!(
            policy.preferred_paths.first(),
            Some(&TransportPath::DirectUdp)
        );
        assert_eq!(policy.preferred_paths.last(), Some(&TransportPath::Relay));
    }
}
