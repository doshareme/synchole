//! Example integration harness.
//!
//! This file documents the intended shape for cross-layer tests. It belongs in a
//! future simulator crate once in-memory discovery and transport adapters exist.

#[test]
#[ignore = "requires simulator adapters"]
fn storage_identity_transport_sync_stack() {
    // 1. Create two device identities in the same user trust graph.
    // 2. Create three SQLite stores: binary, SQL, and document mode.
    // 3. Attach in-memory discovery and encrypted loopback transport.
    // 4. Write data on device A.
    // 5. Run sync from A to B.
    // 6. Assert B receives the object and rejects a third unauthorized device.
}
