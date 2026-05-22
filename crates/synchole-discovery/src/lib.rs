//! Peer discovery and coordination metadata.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use synchole_core::{DeviceId, PeerId, Result, Timestamp, UserId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryBackend {
    CoordinationServer,
    Mdns,
    DnsSd,
    BluetoothLe,
    BrowserSignaling,
    StaticConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerPresence {
    Online,
    Idle,
    Offline,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEndpoint {
    pub address: String,
    pub port: u16,
    pub transport: EndpointTransport,
    pub priority: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndpointTransport {
    Udp,
    Tcp,
    WebRtc,
    WebSocket,
    Relay,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDescriptor {
    pub peer_id: PeerId,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub display_name: String,
    pub endpoints: Vec<PeerEndpoint>,
    pub relay_regions: Vec<String>,
    pub presence: PeerPresence,
    pub last_seen: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryAnnouncement {
    pub descriptor: PeerDescriptor,
    pub capabilities: Vec<String>,
    pub signed_at: Timestamp,
    pub signature: Vec<u8>,
}

#[async_trait]
pub trait PeerDiscovery: Send + Sync {
    async fn announce(&self, announcement: DiscoveryAnnouncement) -> Result<()>;
    async fn withdraw(&self, peer_id: &PeerId) -> Result<()>;
    async fn resolve_peer(&self, peer_id: &PeerId) -> Result<Option<PeerDescriptor>>;
    async fn list_peers(&self, user_id: &UserId) -> Result<Vec<PeerDescriptor>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_priority_is_orderable_by_application_policy() {
        let mut endpoints = [
            PeerEndpoint {
                address: "relay.example".to_owned(),
                port: 443,
                transport: EndpointTransport::Relay,
                priority: 100,
            },
            PeerEndpoint {
                address: "10.0.0.2".to_owned(),
                port: 41641,
                transport: EndpointTransport::Udp,
                priority: 10,
            },
        ];
        endpoints.sort_by_key(|endpoint| endpoint.priority);
        assert_eq!(endpoints[0].transport, EndpointTransport::Udp);
    }
}
