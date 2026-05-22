//! Security scenario examples.

#[test]
#[ignore = "requires crypto transport implementation"]
fn rejects_replayed_or_unauthorized_frames() {
    // 1. Establish an authenticated encrypted session.
    // 2. Replay a previously accepted frame sequence number.
    // 3. Assert the receiver rejects it.
    // 4. Attempt sync with an unenrolled device identity.
    // 5. Assert authorization fails before object exchange.
}
