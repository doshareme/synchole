use std::{
    convert::TryFrom,
    path::Path,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use synchole_core::{Result, StorageMode, SyncholeError, Timestamp, VersionVector};

use crate::traits::SyncStore;
use crate::{
    ChangeCursor, ChangeOperation, ChangePage, ChangeRecord, ObjectKey, RelationalRow,
    StorageConfig, StoragePayload, StoredObject,
};

#[derive(Clone)]
pub struct SQLiteStore {
    conn: Arc<Mutex<Connection>>,
    config: Arc<Mutex<StorageConfig>>,
}

impl SQLiteStore {
    pub async fn open(path: impl AsRef<Path>, config: StorageConfig) -> Result<Self> {
        let conn = Connection::open(path).map_err(to_storage_error)?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
            config: Arc::new(Mutex::new(config)),
        };
        store.apply_pragmas()?;
        store.init_schema()?;
        store.apply_migrations().await?;
        Ok(store)
    }

    pub async fn open_in_memory(config: StorageConfig) -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(to_storage_error)?;
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
            config: Arc::new(Mutex::new(config)),
        };
        store.apply_pragmas()?;
        store.init_schema()?;
        store.apply_migrations().await?;
        Ok(store)
    }

    fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| SyncholeError::Storage("sqlite connection lock poisoned".to_owned()))?;
        f(&guard)
    }

    fn with_config<T>(&self, f: impl FnOnce(&StorageConfig) -> T) -> Result<T> {
        let guard = self
            .config
            .lock()
            .map_err(|_| SyncholeError::Storage("storage config lock poisoned".to_owned()))?;
        Ok(f(&guard))
    }

    fn apply_pragmas(&self) -> Result<()> {
        let pragmas = self.with_config(|config| config.sqlite_pragmas.clone())?;
        self.with_conn(|conn| {
            for (name, value) in pragmas {
                let sql = format!("PRAGMA {name} = {value}");
                conn.execute_batch(&sql).map_err(to_storage_error)?;
            }
            Ok(())
        })
    }

    fn init_schema(&self) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS synchole_objects (
                    collection TEXT NOT NULL,
                    object_id TEXT NOT NULL,
                    mode TEXT NOT NULL,
                    codec TEXT,
                    table_name TEXT,
                    payload BLOB,
                    json_payload TEXT,
                    row_payload TEXT,
                    version_json TEXT NOT NULL,
                    deleted INTEGER NOT NULL DEFAULT 0,
                    updated_wall_time_ms INTEGER NOT NULL,
                    updated_logical INTEGER NOT NULL,
                    PRIMARY KEY(collection, object_id)
                );

                CREATE TABLE IF NOT EXISTS synchole_changes (
                    cursor INTEGER PRIMARY KEY AUTOINCREMENT,
                    collection TEXT NOT NULL,
                    object_id TEXT NOT NULL,
                    version_json TEXT NOT NULL,
                    operation TEXT NOT NULL,
                    updated_wall_time_ms INTEGER NOT NULL,
                    updated_logical INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS synchole_schema_migrations (
                    version INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    applied_at_ms INTEGER NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_synchole_changes_cursor
                    ON synchole_changes(cursor);
                CREATE INDEX IF NOT EXISTS idx_synchole_objects_collection
                    ON synchole_objects(collection);
                "#,
            )
            .map_err(to_storage_error)
        })
    }

    fn record_change(
        &self,
        conn: &Connection,
        key: &ObjectKey,
        version: &VersionVector,
        operation: ChangeOperation,
        updated_at: Timestamp,
    ) -> Result<()> {
        let version_json = serde_json::to_string(version).map_err(to_storage_error)?;
        conn.execute(
            r#"
            INSERT INTO synchole_changes (
                collection, object_id, version_json, operation,
                updated_wall_time_ms, updated_logical
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                key.collection.as_str(),
                key.object_id.as_str(),
                version_json,
                operation_label(&operation),
                i64::try_from(updated_at.wall_time_ms)
                    .map_err(|_| SyncholeError::Storage("timestamp too large".to_owned()))?,
                i64::from(updated_at.logical),
            ],
        )
        .map_err(to_storage_error)?;
        Ok(())
    }
}

#[async_trait]
impl crate::SyncStore for SQLiteStore {
    async fn configure(&self, config: StorageConfig) -> Result<()> {
        {
            let mut guard = self
                .config
                .lock()
                .map_err(|_| SyncholeError::Storage("storage config lock poisoned".to_owned()))?;
            *guard = config;
        }
        self.apply_pragmas()?;
        self.init_schema()?;
        self.apply_migrations().await
    }

    async fn mode(&self) -> StorageMode {
        self.with_config(|config| config.mode.clone())
            .unwrap_or(StorageMode::Binary)
    }

    async fn apply_migrations(&self) -> Result<()> {
        let migrations = self.with_config(|config| config.migrations.migrations.clone())?;
        self.with_conn(|conn| {
            for migration in migrations {
                let already_applied = conn
                    .query_row(
                        "SELECT 1 FROM synchole_schema_migrations WHERE version = ?1",
                        params![migration.version],
                        |row| row.get::<_, i64>(0),
                    )
                    .optional()
                    .map_err(to_storage_error)?
                    .is_some();

                if !already_applied {
                    conn.execute_batch(&migration.sql)
                        .map_err(to_storage_error)?;
                    conn.execute(
                        "INSERT INTO synchole_schema_migrations(version, name, applied_at_ms)
                         VALUES (?1, ?2, strftime('%s','now') * 1000)",
                        params![migration.version, migration.name],
                    )
                    .map_err(to_storage_error)?;
                }
            }
            Ok(())
        })
    }

    async fn get_object(&self, key: &ObjectKey) -> Result<Option<StoredObject>> {
        self.with_conn(|conn| {
            conn.query_row(
                r#"
                SELECT mode, codec, table_name, payload, json_payload, row_payload,
                       version_json, deleted, updated_wall_time_ms, updated_logical
                FROM synchole_objects
                WHERE collection = ?1 AND object_id = ?2
                "#,
                params![key.collection.as_str(), key.object_id.as_str()],
                |row| {
                    let mode: String = row.get(0)?;
                    let codec: Option<String> = row.get(1)?;
                    let table_name: Option<String> = row.get(2)?;
                    let payload: Option<Vec<u8>> = row.get(3)?;
                    let json_payload: Option<String> = row.get(4)?;
                    let row_payload: Option<String> = row.get(5)?;
                    let version_json: String = row.get(6)?;
                    let deleted: i64 = row.get(7)?;
                    let wall: i64 = row.get(8)?;
                    let logical: i64 = row.get(9)?;

                    Ok((
                        mode,
                        codec,
                        table_name,
                        payload,
                        json_payload,
                        row_payload,
                        version_json,
                        deleted,
                        wall,
                        logical,
                    ))
                },
            )
            .optional()
            .map_err(to_storage_error)?
            .map(|row| row_to_object(key.clone(), row))
            .transpose()
        })
    }

    async fn put_object(&self, object: StoredObject) -> Result<()> {
        self.with_conn(|conn| {
            let version_json = serde_json::to_string(&object.version).map_err(to_storage_error)?;
            let (mode, codec, table_name, payload, json_payload, row_payload) =
                payload_columns(&object.payload)?;

            conn.execute(
                r#"
                INSERT INTO synchole_objects (
                    collection, object_id, mode, codec, table_name, payload, json_payload,
                    row_payload, version_json, deleted, updated_wall_time_ms, updated_logical
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT(collection, object_id) DO UPDATE SET
                    mode = excluded.mode,
                    codec = excluded.codec,
                    table_name = excluded.table_name,
                    payload = excluded.payload,
                    json_payload = excluded.json_payload,
                    row_payload = excluded.row_payload,
                    version_json = excluded.version_json,
                    deleted = excluded.deleted,
                    updated_wall_time_ms = excluded.updated_wall_time_ms,
                    updated_logical = excluded.updated_logical
                "#,
                params![
                    object.key.collection.as_str(),
                    object.key.object_id.as_str(),
                    mode,
                    codec,
                    table_name,
                    payload,
                    json_payload,
                    row_payload,
                    version_json,
                    if object.deleted { 1_i64 } else { 0_i64 },
                    i64::try_from(object.updated_at.wall_time_ms)
                        .map_err(|_| SyncholeError::Storage("timestamp too large".to_owned()))?,
                    i64::from(object.updated_at.logical),
                ],
            )
            .map_err(to_storage_error)?;

            self.record_change(
                conn,
                &object.key,
                &object.version,
                ChangeOperation::Put,
                object.updated_at,
            )
        })
    }

    async fn delete_object(
        &self,
        key: ObjectKey,
        version: VersionVector,
        updated_at: Timestamp,
    ) -> Result<()> {
        self.with_conn(|conn| {
            let version_json = serde_json::to_string(&version).map_err(to_storage_error)?;
            conn.execute(
                r#"
                INSERT INTO synchole_objects (
                    collection, object_id, mode, version_json, deleted,
                    updated_wall_time_ms, updated_logical
                ) VALUES (?1, ?2, 'tombstone', ?3, 1, ?4, ?5)
                ON CONFLICT(collection, object_id) DO UPDATE SET
                    version_json = excluded.version_json,
                    deleted = 1,
                    updated_wall_time_ms = excluded.updated_wall_time_ms,
                    updated_logical = excluded.updated_logical
                "#,
                params![
                    key.collection.as_str(),
                    key.object_id.as_str(),
                    version_json,
                    i64::try_from(updated_at.wall_time_ms)
                        .map_err(|_| SyncholeError::Storage("timestamp too large".to_owned()))?,
                    i64::from(updated_at.logical),
                ],
            )
            .map_err(to_storage_error)?;
            self.record_change(conn, &key, &version, ChangeOperation::Delete, updated_at)
        })
    }

    async fn scan_changes(&self, since: Option<ChangeCursor>, limit: usize) -> Result<ChangePage> {
        let limit = limit.max(1);
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    r#"
                    SELECT cursor, collection, object_id, version_json, operation,
                           updated_wall_time_ms, updated_logical
                    FROM synchole_changes
                    WHERE cursor > ?1
                    ORDER BY cursor ASC
                    LIMIT ?2
                    "#,
                )
                .map_err(to_storage_error)?;

            let rows = stmt
                .query_map(
                    params![
                        i64::try_from(since.unwrap_or_default().0).map_err(|_| {
                            SyncholeError::Storage("change cursor too large".to_owned())
                        })?,
                        i64::try_from(limit)
                            .map_err(|_| SyncholeError::Storage("limit too large".to_owned()))?,
                    ],
                    |row| {
                        let cursor: i64 = row.get(0)?;
                        let collection: String = row.get(1)?;
                        let object_id: String = row.get(2)?;
                        let version_json: String = row.get(3)?;
                        let operation: String = row.get(4)?;
                        let wall: i64 = row.get(5)?;
                        let logical: i64 = row.get(6)?;

                        Ok((
                            cursor,
                            collection,
                            object_id,
                            version_json,
                            operation,
                            wall,
                            logical,
                        ))
                    },
                )
                .map_err(to_storage_error)?;

            let mut records = Vec::new();
            for row in rows {
                let (cursor, collection, object_id, version_json, operation, wall, logical) =
                    row.map_err(to_storage_error)?;
                let version = serde_json::from_str(&version_json).map_err(to_storage_error)?;
                records.push(ChangeRecord {
                    cursor: ChangeCursor(u64::try_from(cursor).map_err(|_| {
                        SyncholeError::Storage("negative change cursor".to_owned())
                    })?),
                    key: ObjectKey::new(collection.into(), object_id.into()),
                    version,
                    operation: parse_operation(&operation)?,
                    updated_at: Timestamp {
                        wall_time_ms: u64::try_from(wall)
                            .map_err(|_| SyncholeError::Storage("negative timestamp".to_owned()))?,
                        logical: u32::try_from(logical).map_err(|_| {
                            SyncholeError::Storage("invalid logical timestamp".to_owned())
                        })?,
                    },
                });
            }

            let next_cursor = records.last().map(|record| record.cursor);
            Ok(ChangePage {
                records,
                next_cursor,
            })
        })
    }

    async fn compact(&self) -> Result<()> {
        self.with_conn(|conn| {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")
                .map_err(to_storage_error)
        })
    }
}

type PayloadColumns = (
    String,
    Option<String>,
    Option<String>,
    Option<Vec<u8>>,
    Option<String>,
    Option<String>,
);

fn payload_columns(payload: &StoragePayload) -> Result<PayloadColumns> {
    match payload {
        StoragePayload::Binary { codec, bytes } => Ok((
            "binary".to_owned(),
            Some(codec.clone()),
            None,
            Some(bytes.clone()),
            None,
            None,
        )),
        StoragePayload::SqlRow { table, row } => Ok((
            "sql".to_owned(),
            None,
            Some(table.clone()),
            None,
            None,
            Some(serde_json::to_string(row).map_err(to_storage_error)?),
        )),
        StoragePayload::Document(value) => Ok((
            "document".to_owned(),
            None,
            None,
            None,
            Some(serde_json::to_string(value).map_err(to_storage_error)?),
            None,
        )),
    }
}

type RawObjectRow = (
    String,
    Option<String>,
    Option<String>,
    Option<Vec<u8>>,
    Option<String>,
    Option<String>,
    String,
    i64,
    i64,
    i64,
);

fn row_to_object(key: ObjectKey, row: RawObjectRow) -> Result<StoredObject> {
    let (
        mode,
        codec,
        table_name,
        payload,
        json_payload,
        row_payload,
        version_json,
        deleted,
        wall,
        logical,
    ) = row;

    let payload = match mode.as_str() {
        "binary" => StoragePayload::Binary {
            codec: codec.unwrap_or_else(|| "custom".to_owned()),
            bytes: payload.unwrap_or_default(),
        },
        "sql" => StoragePayload::SqlRow {
            table: table_name.unwrap_or_else(|| "unknown".to_owned()),
            row: parse_relational_row(row_payload)?,
        },
        "document" => {
            let json = json_payload.unwrap_or_else(|| "null".to_owned());
            StoragePayload::Document(serde_json::from_str(&json).map_err(to_storage_error)?)
        }
        "tombstone" => StoragePayload::Document(serde_json::Value::Null),
        other => {
            return Err(SyncholeError::Storage(format!(
                "unknown sqlite object mode: {other}"
            )))
        }
    };

    Ok(StoredObject {
        key,
        payload,
        version: serde_json::from_str(&version_json).map_err(to_storage_error)?,
        updated_at: Timestamp {
            wall_time_ms: u64::try_from(wall)
                .map_err(|_| SyncholeError::Storage("negative timestamp".to_owned()))?,
            logical: u32::try_from(logical)
                .map_err(|_| SyncholeError::Storage("invalid logical timestamp".to_owned()))?,
        },
        deleted: deleted != 0,
    })
}

fn parse_relational_row(value: Option<String>) -> Result<RelationalRow> {
    match value {
        Some(json) => serde_json::from_str(&json).map_err(to_storage_error),
        None => Ok(RelationalRow::default()),
    }
}

fn operation_label(operation: &ChangeOperation) -> &'static str {
    match operation {
        ChangeOperation::Put => "put",
        ChangeOperation::Delete => "delete",
    }
}

fn parse_operation(value: &str) -> Result<ChangeOperation> {
    match value {
        "put" => Ok(ChangeOperation::Put),
        "delete" => Ok(ChangeOperation::Delete),
        other => Err(SyncholeError::Storage(format!(
            "unknown change operation: {other}"
        ))),
    }
}

fn to_storage_error(error: impl std::fmt::Display) -> SyncholeError {
    SyncholeError::Storage(error.to_string())
}
