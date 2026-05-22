use std::sync::Arc;

use synchole::{
    CollectionId, ObjectId, SQLiteStore, StorageConfig, StorageMode, Synchole, SyncholeConfig,
};

#[tokio::main]
async fn main() -> synchole::Result<()> {
    let store =
        SQLiteStore::open("synchole-example.db", StorageConfig::new(StorageMode::Document))
            .await?;

    let sdk = Synchole::builder()
        .with_config(SyncholeConfig {
            storage_mode: StorageMode::Document,
            ..SyncholeConfig::default()
        })
        .with_storage(Arc::new(store))
        .build()
        .await?;

    sdk.put_document(
        CollectionId::from("notes"),
        ObjectId::from("welcome"),
        serde_json::json!({"title": "Hello from Synchole"}),
    )
    .await?;

    let report = sdk.sync_once().await?;
    println!("peers contacted: {}", report.peers_contacted);
    Ok(())
}
