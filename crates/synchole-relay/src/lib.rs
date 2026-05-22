//! Relay fallback contracts.
//!
//! A production deployment would implement a DERP-like low-latency relay service
//! that forwards encrypted frames without access to plaintext payloads.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use synchole_core::{PeerId, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayRegion {
    pub id: String,
    pub display_name: String,
    pub endpoints: Vec<String>,
    pub priority: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayReservation {
    pub region_id: String,
    pub relay_peer_id: PeerId,
    pub expires_at_ms: u64,
    pub resume_token: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayPolicy {
    pub allowed_regions: Vec<String>,
    pub max_buffered_bytes: usize,
    pub require_e2e_encryption: bool,
}

impl Default for RelayPolicy {
    fn default() -> Self {
        Self {
            allowed_regions: Vec::new(),
            max_buffered_bytes: 16 * 1_048_576,
            require_e2e_encryption: true,
        }
    }
}

#[async_trait]
pub trait RelayClient: Send + Sync {
    async fn list_regions(&self) -> Result<Vec<RelayRegion>>;
    async fn reserve(&self, peer_id: &PeerId, policy: RelayPolicy) -> Result<RelayReservation>;
    async fn send_encrypted(&self, reservation: &RelayReservation, frame: Vec<u8>) -> Result<()>;
    async fn receive_encrypted(&self, reservation: &RelayReservation) -> Result<Option<Vec<u8>>>;
}
