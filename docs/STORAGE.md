# Storage Architecture

Synchole uses SQLite as the default persistence engine on every supported
platform. The sync engine talks to `SyncStore`, so it is independent of the
selected storage format.

## Storage Modes

### Binary Mode

Binary mode stores each object as an opaque compact blob. It is optimized for
high-throughput replication and small write amplification.

Use it when:

- The application already owns serialization.
- Local structured queries are unnecessary.
- Snapshot and delta replication speed is more important than ad hoc access.

Supported codec choices include `postcard`, `bincode`, `rkyv`, and custom codecs.

### SQL Mode

SQL mode stores synchronized entities as relational rows. It is optimized for
local indexed access, joins, projections, and migrations.

Use it when:

- The application wants structured local reads without deserializing every object.
- Schema migrations are part of the product lifecycle.
- Sync collections map naturally to tables.

The scaffold includes migration metadata. Production adapters should add
application-controlled projection hooks so the sync log and relational tables
remain transactionally consistent.

### Document Mode

Document mode stores JSON, BLOB, or hybrid document records inside SQLite. It is
suited to document sync, key-value sync, and CRDT-like objects.

Use it when:

- Records have evolving schemas.
- Objects need JSON queries where SQLite JSON functions are available.
- CRDT payloads are stored as JSON or binary states.

## Unified Trait

All modes implement the same trait:

```rust
#[async_trait]
pub trait SyncStore: Send + Sync {
    async fn configure(&self, config: StorageConfig) -> Result<()>;
    async fn mode(&self) -> StorageMode;
    async fn apply_migrations(&self) -> Result<()>;
    async fn get_object(&self, key: &ObjectKey) -> Result<Option<StoredObject>>;
    async fn put_object(&self, object: StoredObject) -> Result<()>;
    async fn delete_object(
        &self,
        key: ObjectKey,
        version: VersionVector,
        updated_at: Timestamp,
    ) -> Result<()>;
    async fn scan_changes(&self, since: Option<ChangeCursor>, limit: usize) -> Result<ChangePage>;
    async fn compact(&self) -> Result<()>;
}
```

## SQLite Native Defaults

The native store configures SQLite with:

- WAL journaling.
- `synchronous = NORMAL`.
- foreign keys enabled.
- a sync object table.
- a change log table.
- a schema migration table.

Mobile applications should still coordinate this with platform lifecycle and
backup policies. Browser applications should run storage in a worker and prefer
OPFS-backed SQLite builds.

## Mode Test Matrix

Every sync behavior should run against:

- `StorageMode::Binary`
- `StorageMode::Sql`
- `StorageMode::Document`

The file `crates/synchole-storage/tests/mode_matrix.rs` shows the reusable test
shape. End-to-end tests should instantiate every mode through the same
`SyncStore` trait object.
