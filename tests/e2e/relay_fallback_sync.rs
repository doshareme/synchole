//! End-to-end scenario: relay fallback.

#[test]
#[ignore = "requires relay simulator"]
fn relay_fallback_syncs_when_direct_paths_fail() {
    // Arrange peers behind symmetric NATs with direct transfer disabled.
    // Sync should reserve a relay, send encrypted frames, and converge data.
}
