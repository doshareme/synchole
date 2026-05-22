use serde::{Deserialize, Serialize};

/// Runtime platform used for capability checks and binding defaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Android,
    Ios,
    MacOs,
    Windows,
    Linux,
    Browser,
    WebWorker,
    Unknown(String),
}

/// SQLite-backed storage mode selected by the application.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageMode {
    /// Compact opaque replication records encoded by a binary codec.
    Binary,
    /// Relational tables with migrations and indexed local queries.
    Sql,
    /// Document/key-value/CRDT objects represented with JSON and BLOB columns.
    Document,
}

/// Preference for how aggressively the SDK should search for direct paths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionPreference {
    DirectOnly,
    DirectWithRelayFallback,
    RelayOnly,
}

/// Battery-sensitive scheduling policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryPolicy {
    Interactive,
    Balanced,
    Saver,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff_ms: 250,
            max_backoff_ms: 30_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportPolicy {
    pub preference: ConnectionPreference,
    pub keepalive_interval_ms: u64,
    pub idle_timeout_ms: u64,
    pub max_frame_bytes: usize,
}

impl Default for TransportPolicy {
    fn default() -> Self {
        Self {
            preference: ConnectionPreference::DirectWithRelayFallback,
            keepalive_interval_ms: 15_000,
            idle_timeout_ms: 120_000,
            max_frame_bytes: 1_048_576,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncholeConfig {
    pub platform: Platform,
    pub storage_mode: StorageMode,
    pub transport: TransportPolicy,
    pub retry: RetryPolicy,
    pub battery: BatteryPolicy,
    pub sync_batch_size: usize,
}

impl Default for SyncholeConfig {
    fn default() -> Self {
        Self {
            platform: Platform::Unknown("runtime-detected".to_owned()),
            storage_mode: StorageMode::Binary,
            transport: TransportPolicy::default(),
            retry: RetryPolicy::default(),
            battery: BatteryPolicy::Balanced,
            sync_batch_size: 512,
        }
    }
}
