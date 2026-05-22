//! End-to-end scenario: direct peer-to-peer sync.

#[test]
#[ignore = "requires network simulator"]
fn direct_peer_to_peer_sync_prefers_direct_path() {
    // Arrange two devices with reachable UDP endpoints.
    // Sync should select TransportPath::DirectUdp and never reserve a relay.
}
