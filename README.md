# Synchole

Synchole is a Rust-first, multiplatform device-to-device synchronization framework.
It is designed for offline-first applications that need encrypted peer sync across
Android, iOS, Windows, macOS, Linux, and browser/Web Worker environments.

The network architecture is inspired by Tailscale-style private networking:
stable device identity, peer discovery, NAT traversal, encrypted peer tunnels,
direct transfer when possible, and relay fallback when direct paths fail.

## Workspace

- `crates/synchole`: public SDK facade.
- `crates/synchole-core`: shared IDs, errors, clocks, config, and version vectors.
- `crates/synchole-identity`: device identity, signatures, trust policy, and keyring traits.
- `crates/synchole-discovery`: peer discovery traits and coordination metadata.
- `crates/synchole-transport`: secure tunnel traits, path selection, and frame types.
- `crates/synchole-nat`: STUN/TURN/ICE-style NAT traversal contracts.
- `crates/synchole-relay`: DERP-like relay fallback contracts.
- `crates/synchole-storage`: SQLite storage abstraction with binary, SQL, and document modes.
- `crates/synchole-sync`: sync protocol, conflict resolution, and replication engine.
- `crates/synchole-bindings`: Kotlin/Java, Swift, C ABI, and WASM binding surface.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Storage](docs/STORAGE.md)
- [Testing](docs/TESTING.md)
- [Security](docs/SECURITY.md)

## Example

```rust
use std::sync::Arc;

use synchole::{
    CollectionId, ObjectId, SQLiteStore, StorageConfig, StorageMode, Synchole,
    SyncholeConfig,
};

# async fn example() -> synchole::Result<()> {
let store = SQLiteStore::open("app-sync.db", StorageConfig::new(StorageMode::Binary)).await?;

let sdk = Synchole::builder()
    .with_config(SyncholeConfig::default())
    .with_storage(Arc::new(store))
    .build()
    .await?;

sdk.put_binary(
    CollectionId::from("notes"),
    ObjectId::from("note-1"),
    b"encrypted application payload".to_vec(),
)
.await?;

let report = sdk.sync_once().await?;
println!("synced {} peers", report.peers_contacted);
# Ok(())
# }
```

This repository is a production-oriented architecture scaffold. Network adapters,
platform keychains, and production relay services are intentionally behind traits so
applications can choose concrete backends without changing sync semantics.
