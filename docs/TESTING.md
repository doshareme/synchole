# Testing Strategy

The test strategy treats Synchole as a distributed system, not only as a Rust
library. Every feature must have unit coverage, cross-layer integration coverage,
and at least one end-to-end scenario in the simulated device network.

## Test Matrix

| Area | Unit | Integration | End-to-end | Security | Performance | Platform build |
| --- | --- | --- | --- | --- | --- | --- |
| Core IDs, clocks, versions | yes | yes | indirect | indirect | no | all |
| Identity and trust | yes | yes | yes | yes | no | all |
| Peer discovery | yes | yes | yes | yes | latency | all |
| NAT traversal | yes | yes | yes | downgrade tests | path setup latency | native, browser where possible |
| Secure transport | yes | yes | yes | replay, tamper, unauthorized peer | throughput, RTT | all |
| Relay fallback | yes | yes | yes | relay plaintext resistance | relay latency | all |
| Storage binary mode | yes | yes | yes | encrypted payload boundary | read/write speed | all |
| Storage SQL mode | yes | yes | yes | migration authorization | query/write speed | all |
| Storage document mode | yes | yes | yes | malformed document handling | JSON query speed | all |
| Conflict resolution | yes | yes | yes | malicious version vectors | resolver cost | all |
| Bindings | smoke | host integration | app harness | handle misuse | call overhead | target platforms |

## Required Test Classes

1. Unit tests

   Validate small deterministic pieces: ID parsing, version vector causality,
   conflict resolver decisions, path ordering, storage serialization, migration
   ordering, and trust policy rejection.

2. Integration tests

   Compose storage, identity, discovery, transport, and sync through in-memory
   adapters. These tests should run without external services.

3. End-to-end tests

   Use a deterministic network simulator to create multiple devices and routes:
   direct peer-to-peer, NAT traversal path, relay fallback, offline/reconnect,
   and concurrent conflict scenarios.

4. Platform build tests

   Verify that the same source builds for Android, iOS, Windows, macOS, Linux,
   and WebAssembly/browser targets. Platform tests should be split into build
   smoke tests and runtime harness tests.

5. Storage mode tests

   Run the same sync scenario against binary SQLite mode, SQL SQLite mode, and
   document SQLite mode. The sync engine should not branch on storage mode.

6. Security tests

   Verify encryption, identity validation, replay protection, downgrade
   rejection, tamper detection, unauthorized peer rejection, and revocation.

7. Performance tests

   Benchmark sync throughput, setup latency, relay latency, SQLite write/read
   speed, large snapshot transfer, incremental delta transfer, and mobile
   battery-sensitive scheduling.

## Example Files

- `crates/synchole-core/src/clock.rs`: unit test for hybrid logical clock.
- `crates/synchole-core/src/version.rs`: unit test for version vector causality.
- `crates/synchole-identity/tests/identity_trust.rs`: trust policy test.
- `crates/synchole-storage/tests/mode_matrix.rs`: storage mode matrix test.
- `crates/synchole-sync/tests/conflict_resolution.rs`: conflict resolver test.
- `tests/integration/storage_identity_sync.rs`: cross-layer integration harness.
- `tests/e2e/direct_p2p_sync.rs`: direct transfer scenario.
- `tests/e2e/nat_traversal_sync.rs`: NAT traversal scenario.
- `tests/e2e/relay_fallback_sync.rs`: relay fallback scenario.
- `tests/e2e/offline_reconnect_conflict.rs`: offline and reconnect scenario.
- `tests/security/replay_and_identity.rs`: security scenario.
- `tests/platform/build_targets.md`: platform build checklist.
- `benches/sync_throughput.rs`: benchmark harness.

## CI Workflow

The recommended CI workflow is in `.github/workflows/ci.yml` and includes:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- storage feature matrix
- Windows, macOS, and Linux build matrix
- Android target build
- iOS target build on macOS
- WASM build
- security test job
- performance benchmark smoke job

Long-running network simulations and mobile device-lab tests should run nightly.
