use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use synchole_core::{CollectionId, ObjectId, Result, StorageMode, Timestamp, VersionVector};

use crate::mode::StorageConfig;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObjectKey {
    pub collection: CollectionId,
    pub object_id: ObjectId,
}

impl ObjectKey {
    pub fn new(collection: CollectionId, object_id: ObjectId) -> Self {
        Self {
            collection,
            object_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SqlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Bool(bool),
    Blob(Vec<u8>),
    Json(Value),
}

pub type RelationalRow = BTreeMap<String, SqlValue>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StoragePayload {
    Binary { codec: String, bytes: Vec<u8> },
    SqlRow { table: String, row: RelationalRow },
    Document(Value),
}

impl StoragePayload {
    pub fn mode(&self) -> StorageMode {
        match self {
            Self::Binary { .. } => StorageMode::Binary,
            Self::SqlRow { .. } => StorageMode::Sql,
            Self::Document(_) => StorageMode::Document,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StoredObject {
    pub key: ObjectKey,
    pub payload: StoragePayload,
    pub version: VersionVector,
    pub updated_at: Timestamp,
    pub deleted: bool,
}

impl StoredObject {
    pub fn new(
        collection: CollectionId,
        object_id: ObjectId,
        payload: StoragePayload,
        version: VersionVector,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            key: ObjectKey::new(collection, object_id),
            payload,
            version,
            updated_at,
            deleted: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ChangeCursor(pub u64);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeOperation {
    Put,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub cursor: ChangeCursor,
    pub key: ObjectKey,
    pub version: VersionVector,
    pub operation: ChangeOperation,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangePage {
    pub records: Vec<ChangeRecord>,
    pub next_cursor: Option<ChangeCursor>,
}

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
