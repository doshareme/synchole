//! End-to-end scenario: offline edits and reconnect conflict.

#[test]
#[ignore = "requires simulator adapters"]
fn offline_devices_reconnect_and_resolve_conflict() {
    // Device A and B edit the same object while offline.
    // On reconnect, version vectors should detect concurrency.
    // The configured resolver should converge both stores deterministically.
}
