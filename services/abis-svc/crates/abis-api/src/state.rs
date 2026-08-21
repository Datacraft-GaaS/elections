//! Application state shared across handlers.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Idempotency key store (in-memory, keyed by Idempotency-Key header value).
#[derive(Default)]
pub struct IdempotencyStore {
    inner: Mutex<HashMap<String, serde_json::Value>>,
}

impl IdempotencyStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the key has been seen; return the cached response if so.
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.inner
            .lock()
            .expect("idempotency lock")
            .get(key)
            .cloned()
    }

    /// Store a response for the given key.
    pub fn set(&self, key: String, value: serde_json::Value) {
        self.inner
            .lock()
            .expect("idempotency lock")
            .insert(key, value);
    }
}

pub struct AppState {
    pub mock_mode: bool,
    pub idempotency: IdempotencyStore,
}

impl AppState {
    pub fn new() -> Self {
        let mock_mode = std::env::var("EEL_ABIS_MOCK")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(true);
        Self {
            mock_mode,
            idempotency: IdempotencyStore::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
