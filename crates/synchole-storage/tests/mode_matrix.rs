use serde_json::json;
use synchole_core::{CollectionId, ObjectId, StorageMode, Timestamp, VersionVector};
use synchole_storage::{
    ObjectKey, RelationalRow, SQLiteStore, SqlValue, StorageConfig, StoragePayload, StoredObject,
    SyncStore,
};

#[tokio::test]
async fn binary_sql_and_document_modes_share_sync_store_contract() {
    let cases = [StorageMode::Binary, StorageMode::Sql, StorageMode::Document];

    for mode in cases {
        let store = SQLiteStore::open_in_memory(StorageConfig::new(mode.clone()))
            .await
            .expect("open sqlite store");

        let mut version = VersionVector::new();
        version.increment("device-a".into());

        let payload = match mode {
            StorageMode::Binary => StoragePayload::Binary {
                codec: "postcard".to_owned(),
                bytes: vec![1, 2, 3, 4],
            },
            StorageMode::Sql => {
                let mut row = RelationalRow::new();
                row.insert("title".to_owned(), SqlValue::Text("hello".to_owned()));
                StoragePayload::SqlRow {
                    table: "notes".to_owned(),
                    row,
                }
            }
            StorageMode::Document => StoragePayload::Document(json!({"title": "hello"})),
        };

        let key = ObjectKey::new(CollectionId::from("notes"), ObjectId::from("1"));
        let object = StoredObject::new(
            key.collection.clone(),
            key.object_id.clone(),
            payload,
            version,
            Timestamp {
                wall_time_ms: 100,
                logical: 0,
            },
        );

        store.put_object(object).await.expect("put object");
        assert!(store.get_object(&key).await.expect("get object").is_some());

        let changes = store.scan_changes(None, 10).await.expect("scan changes");
        assert_eq!(changes.records.len(), 1);
    }
}
