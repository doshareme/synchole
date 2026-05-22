//! Shared primitives for the Synchole SDK.

pub mod clock;
pub mod config;
pub mod error;
pub mod ids;
pub mod version;

pub use clock::{HybridLogicalClock, Timestamp};
pub use config::{
    BatteryPolicy, ConnectionPreference, Platform, RetryPolicy, StorageMode, SyncholeConfig,
    TransportPolicy,
};
pub use error::{Result, SyncholeError};
pub use ids::{CollectionId, DeviceId, ObjectId, PeerId, SessionId, UserId};
pub use version::{Causality, VersionVector};
