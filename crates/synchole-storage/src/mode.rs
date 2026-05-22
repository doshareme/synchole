use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use synchole_core::StorageMode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageConfig {
    pub mode: StorageMode,
    pub namespace: String,
    pub sqlite_pragmas: BTreeMap<String, String>,
    pub migrations: MigrationPlan,
}

impl StorageConfig {
    pub fn new(mode: StorageMode) -> Self {
        let mut sqlite_pragmas = BTreeMap::new();
        sqlite_pragmas.insert("journal_mode".to_owned(), "WAL".to_owned());
        sqlite_pragmas.insert("synchronous".to_owned(), "NORMAL".to_owned());
        sqlite_pragmas.insert("foreign_keys".to_owned(), "ON".to_owned());

        Self {
            mode,
            namespace: "default".to_owned(),
            sqlite_pragmas,
            migrations: MigrationPlan::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub schema_version: u32,
    pub migrations: Vec<Migration>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub sql: String,
}
