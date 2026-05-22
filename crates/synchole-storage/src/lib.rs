//! Persistence layer used by the synchronization engine.
//!
//! SQLite is the default storage engine. The same sync protocol talks to
//! [`SyncStore`] regardless of whether objects are persisted as opaque binary
//! blobs, relational rows, or document-style records.

pub mod codec;
pub mod mode;
#[cfg(feature = "native-sqlite")]
pub mod sqlite;
pub mod traits;

pub use codec::{BinaryCodec, CodecId};
pub use mode::{Migration, MigrationPlan, StorageConfig};
#[cfg(feature = "native-sqlite")]
pub use sqlite::SQLiteStore;
pub use traits::{
    ChangeCursor, ChangeOperation, ChangePage, ChangeRecord, ObjectKey, RelationalRow, SqlValue,
    StoragePayload, StoredObject, SyncStore,
};
