use thiserror::Error;

/// Result type used across Synchole crates.
pub type Result<T> = std::result::Result<T, SyncholeError>;

/// Error taxonomy shared by the SDK.
#[derive(Debug, Error)]
pub enum SyncholeError {
    #[error("identity error: {0}")]
    Identity(String),
    #[error("peer discovery error: {0}")]
    Discovery(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("nat traversal error: {0}")]
    NatTraversal(String),
    #[error("relay error: {0}")]
    Relay(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("sync protocol error: {0}")]
    Sync(String),
    #[error("conflict resolution error: {0}")]
    Conflict(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("feature is unavailable on this platform: {0}")]
    UnsupportedPlatform(String),
    #[error("operation timed out: {0}")]
    Timeout(String),
}
