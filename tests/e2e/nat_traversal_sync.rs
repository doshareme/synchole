//! End-to-end scenario: NAT traversal.

#[test]
#[ignore = "requires network simulator"]
fn nat_traversal_path_syncs_after_hole_punching() {
    // Arrange two devices behind compatible cone NATs.
    // Candidate gathering should produce server-reflexive candidates.
    // Hole punching should establish TransportPath::NatTraversalUdp.
}
