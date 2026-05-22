//! Platform binding surfaces.
//!
//! Each binding keeps ownership, async runtime, and platform storage concerns at
//! the edge while reusing the same Rust sync engine.

#[cfg(feature = "android")]
pub mod android;
#[cfg(feature = "apple")]
pub mod apple;
#[cfg(feature = "c-abi")]
pub mod cabi;
#[cfg(feature = "wasm")]
pub mod wasm;

use serde::{Deserialize, Serialize};
use synchole::StorageMode;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingStorageConfig {
    pub path_or_name: String,
    pub mode: StorageMode,
}

impl BindingStorageConfig {
    pub fn binary(path_or_name: impl Into<String>) -> Self {
        Self {
            path_or_name: path_or_name.into(),
            mode: StorageMode::Binary,
        }
    }
}
