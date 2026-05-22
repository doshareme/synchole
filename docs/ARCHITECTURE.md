# Synchole Architecture

Synchole is a Rust-first SDK for device-to-device synchronization across mobile,
desktop, and browser runtimes. It is intentionally organized as a set of narrow
crates so applications can replace discovery, transport, identity, and storage
adapters without changing the sync protocol.

## High-Level Architecture

```text
Application
    |
    v
synchole SDK facade
    |
    +-- identity          Device identity, enrollment, signatures, trust policy
    +-- discovery         Coordination server, LAN discovery, WebRTC signaling
    +-- nat               STUN/TURN/ICE-style candidate gathering and hole punching
    +-- relay             DERP-like encrypted relay fallback
    +-- transport         Encrypted sessions, path selection, framing, metrics
    +-- sync              Change exchange, conflict detection, resolution
    +-- storage           SQLite binary, SQL, and document modes
    +-- bindings          Kotlin/Java, Swift, C ABI, WASM
```

The normal path is direct peer-to-peer transfer:

1. The device loads a stable identity from platform key storage.
2. Discovery returns enrolled peers and candidate endpoints.
3. NAT traversal gathers local candidates and attempts simultaneous open.
4. Transport performs an authenticated encrypted handshake.
5. Sync exchanges compact change heads, requests missing objects, and applies
   remote updates through the storage trait.
6. If a direct path fails, transport selects relay fallback. Relay servers only
   see encrypted frames.

## Crate Layout

| Crate | Role |
| --- | --- |
| `synchole` | Public SDK facade and re-exports. |
| `synchole-core` | IDs, version vectors, hybrid logical clocks, config, errors. |
| `synchole-identity` | Device identity, keyring, signatures, trust policy. |
| `synchole-discovery` | Peer descriptors and discovery backends. |
| `synchole-nat` | NAT type probing, candidate gathering, hole punching plans. |
| `synchole-relay` | Relay regions, reservations, encrypted relay client. |
| `synchole-transport` | Secure tunnel traits, frames, path selection, metrics. |
| `synchole-storage` | SQLite storage abstraction and native SQLite store. |
| `synchole-sync` | Sync engine, protocol messages, conflict resolution. |
| `synchole-bindings` | Platform binding crate for mobile, desktop, and browser APIs. |

## Key Traits

The core extension points are:

```rust
pub trait SyncStore: Send + Sync {
    async fn configure(&self, config: StorageConfig) -> Result<()>;
    async fn get_object(&self, key: &ObjectKey) -> Result<Option<StoredObject>>;
    async fn put_object(&self, object: StoredObject) -> Result<()>;
    async fn scan_changes(&self, since: Option<ChangeCursor>, limit: usize) -> Result<ChangePage>;
}

pub trait PeerDiscovery: Send + Sync {
    async fn announce(&self, announcement: DiscoveryAnnouncement) -> Result<()>;
    async fn resolve_peer(&self, peer_id: &PeerId) -> Result<Option<PeerDescriptor>>;
    async fn list_peers(&self, user_id: &UserId) -> Result<Vec<PeerDescriptor>>;
}

pub trait SecureTransport: Send + Sync {
    async fn connect(
        &self,
        peer: &PeerDescriptor,
        policy: ConnectionPolicy,
    ) -> Result<Arc<dyn PeerConnection>>;
}

pub trait ConflictResolver: Send + Sync {
    async fn resolve(&self, conflict: Conflict) -> Result<Resolution>;
}
```

The real source of truth is in the crates. This document shows the intended
contracts at a glance.

## Core Structs and Enums

- `DeviceIdentity`: user ID, device ID, signing key, encryption key, and expiry.
- `PeerDescriptor`: peer ID, endpoints, relay regions, presence, and last seen time.
- `Candidate`: host, server-reflexive, peer-reflexive, or relay connectivity candidate.
- `TransportFrame`: encrypted stream frame metadata and payload.
- `StoredObject`: collection, object ID, payload, version vector, timestamp, tombstone bit.
- `StoragePayload`: binary blob, relational row, or JSON/document payload.
- `VersionVector`: detects before, after, equal, and concurrent updates.
- `SyncMessage`: hello, have, need, object, and acknowledgement protocol messages.
- `Resolution`: keep local, keep remote, merge, or keep both.

## Example Sync Flow

```rust
let store = SQLiteStore::open("sync.db", StorageConfig::new(StorageMode::Document)).await?;

let client = Synchole::builder()
    .with_config(SyncholeConfig::default())
    .with_storage(Arc::new(store))
    .with_identity(identity_provider)
    .with_discovery(discovery)
    .with_transport(transport)
    .with_conflict_resolver(Arc::new(LastWriterWinsResolver))
    .build()
    .await?;

client.put_document("notes".into(), "n1".into(), serde_json::json!({
    "title": "Offline first"
})).await?;

let report = client.sync_once().await?;
```

Internally, `sync_once` discovers peers, selects the best path, opens an
encrypted session, exchanges change metadata, pulls missing objects, detects
causal conflicts with version vectors, and applies the configured resolver.

## Platform Binding Strategy

| Platform | Binding | Storage | Network |
| --- | --- | --- | --- |
| Android | JNI plus Kotlin coroutine wrapper, packaged as AAR | SQLite through native Rust or Android SQLite adapter | UDP/TCP, Android network callbacks, WorkManager |
| iOS | UniFFI Swift bindings, packaged as XCFramework | SQLite native or app-provided connection | Network.framework, background tasks |
| macOS | UniFFI or C ABI | SQLite native | UDP/TCP, Network.framework |
| Windows | C ABI, optional C# wrapper | SQLite native | UDP/TCP, WinHTTP/WebSocket adapters |
| Linux | C ABI and native Rust | SQLite native | UDP/TCP, systemd/user service integration |
| Browser | wasm-bindgen | OPFS-backed SQLite WASM where available | WebRTC data channels, WebSocket signaling, relay fallback |
| Web Worker | wasm-bindgen worker API | OPFS SQLite WASM | WebRTC or WebSocket relay |

## Suggested Dependencies

Production adapters can use:

- Async runtime: `tokio`, `futures`.
- SQLite: `rusqlite` for bundled native SQLite, OPFS SQLite WASM for browsers.
- Serialization: `serde`, `serde_json`, `postcard`, `bincode`, `rkyv`.
- Crypto: `ed25519-dalek`, `x25519-dalek`, `chacha20poly1305`, `blake3`, `zeroize`.
- Transport building blocks: QUIC or UDP stacks, WebRTC crates, platform network APIs.
- Bindings: `jni`, `uniffi`, `cbindgen`, `wasm-bindgen`.
- Testing: `proptest`, `criterion`, platform build targets, simulated networks.

Exact dependency versions should be pinned and audited during release hardening.

## Performance Design

- Prefer direct UDP or WebRTC data channels before relay.
- Exchange version heads before transferring object payloads.
- Batch changes by collection and peer path quality.
- Use WAL mode and prepared statements for SQLite native builds.
- Keep relay connections idle-light with adaptive keepalives.
- Schedule mobile sync by battery state, network type, and foreground/background status.
- Support snapshot chunks for large initial sync and delta frames for steady-state sync.

## Limitations and Future Improvements

- The current repo is an architecture scaffold with a concrete SQLite store and
  public trait contracts. Production transport, relay, and platform keyring
  adapters still need implementation.
- Browser SQLite support depends on OPFS, SharedArrayBuffer availability, and
  deployment headers.
- Multi-writer SQL table projection needs app-defined schema ownership rules.
- CRDT support should be expanded with typed CRDT collections and property tests.
- Relay service implementation, abuse protection, rate limits, and observability
  should live in a separate deployable service workspace.
