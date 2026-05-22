//! NAT traversal primitives for direct peer paths.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use synchole_core::{PeerId, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatType {
    OpenInternet,
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateKind {
    Host,
    ServerReflexive,
    PeerReflexive,
    Relay,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub kind: CandidateKind,
    pub address: String,
    pub port: u16,
    pub protocol: CandidateProtocol,
    pub priority: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateProtocol {
    Udp,
    Tcp,
    WebRtcDataChannel,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolePunchPlan {
    pub local_candidates: Vec<Candidate>,
    pub remote_candidates: Vec<Candidate>,
    pub simultaneous_open_window_ms: u64,
    pub max_attempts: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NatReport {
    pub nat_type: NatType,
    pub public_candidates: Vec<Candidate>,
    pub hairpinning_supported: bool,
}

#[async_trait]
pub trait NatTraversal: Send + Sync {
    async fn probe(&self) -> Result<NatReport>;
    async fn gather_candidates(&self) -> Result<Vec<Candidate>>;
    async fn build_plan(&self, peer_id: &PeerId, remote: Vec<Candidate>) -> Result<HolePunchPlan>;
    async fn punch(&self, plan: HolePunchPlan) -> Result<Vec<Candidate>>;
}
