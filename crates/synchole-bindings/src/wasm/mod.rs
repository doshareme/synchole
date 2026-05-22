//! Browser and Web Worker binding surface.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmSyncholeConfig {
    storage_name: String,
    storage_mode: String,
}

#[wasm_bindgen]
impl WasmSyncholeConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(storage_name: String, storage_mode: String) -> Self {
        Self {
            storage_name,
            storage_mode,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn storage_name(&self) -> String {
        self.storage_name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn storage_mode(&self) -> String {
        self.storage_mode.clone()
    }
}
