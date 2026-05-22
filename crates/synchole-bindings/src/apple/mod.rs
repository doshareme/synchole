//! Swift binding strategy.
//!
//! UniFFI should generate Swift interfaces for high-level SDK calls. Low-level
//! transport callbacks remain Rust traits and are adapted through Swift classes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleRuntimeConfig {
    pub app_group_identifier: Option<String>,
    pub background_sync_enabled: bool,
}

impl Default for AppleRuntimeConfig {
    fn default() -> Self {
        Self {
            app_group_identifier: None,
            background_sync_enabled: true,
        }
    }
}
