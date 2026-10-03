// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Shared helpers for spec contract handlers.
//! Extracted from duplicated boilerplate across 31 `spec.rs` files (TD-08).
#![warn(missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use serde::Serialize;
use serde_json::{json, Value};

/// Character predicates mirroring the anchored patterns on the BVR contract's
/// `Voter` schema (`packages/openapi-contracts/bvr/openapi.yaml`):
/// - `national_id`: `^[0-9]{8}$`
/// - `phone`: `^\+254[0-9]{9}$`
/// - `polling_station_code`: `^[A-Z0-9]{6,12}$`
///
/// The workspace dependency set has no regex crate, so these are hand-rolled and
/// must be kept in step with the contract by hand. `tests/bvr_validation.rs`
/// asserts each against the literals in the spec.
///
/// Named so the signature reads as intent; clippy's `type_complexity` fires on
/// the tuple-of-tuple-and-fn-pointer otherwise.
type BvrPattern = (&'static str, fn(&str) -> bool);

static BVR_PATTERNS: LazyLock<[BvrPattern; 3]> = LazyLock::new(|| {
    [
        ("national_id", |s: &str| {
            s.len() == 8 && s.bytes().all(|b| b.is_ascii_digit())
        }),
        ("phone", |s: &str| {
            // "+254" is 4 bytes, leaving 9 digits for the 13-character E.164 form.
            s.len() == 13
                && s.as_bytes().starts_with(b"+254")
                && s.as_bytes()[4..].iter().all(u8::is_ascii_digit)
        }),
        ("polling_station_code", |s: &str| {
            (6..=12).contains(&s.len())
                && s.bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        }),
    ]
});

/// Returns `true` when `value` satisfies the BVR contract pattern for `field`.
///
/// Unknown fields return `true`, so a caller validating only the fields it cares
/// about need not enumerate the whole contract. This validates the three anchored
/// patterns on `Voter` and nothing else; it is not a schema engine.
pub fn matches_bvr_pattern(field: &str, value: &str) -> bool {
    BVR_PATTERNS
        .iter()
        .find(|(name, _)| *name == field)
        .is_none_or(|(_, check)| check(value))
}

/// Build a paginated response envelope `{"items": [...], "meta": {...}}`.
/// Mirrors the `page` helper duplicated across spec handlers.
pub fn page<T: Serialize>(items: Vec<T>) -> Value {
    let n = items.len();
    let v = serde_json::to_value(items).unwrap_or(Value::Null);
    json!({ "items": v, "meta": { "page": 1, "page_size": n, "total": n, "total_pages": 1 } })
}

/// Build a paginated response from a `serde_json::Value` array.
/// Use when items are already `Value` (e.g. from `json!`).
pub fn page_value(items: &Value) -> Value {
    let n = items.as_array().map_or(0, |a| a.len());
    json!({ "items": items, "meta": { "page": 1, "page_size": n, "total": n, "total_pages": 1 } })
}

/// Cursor-style pagination: `{"items": [...], "next_cursor": null, "total_count": n}`.
pub fn page_cursor_value(items: &Value) -> Value {
    let n = items.as_array().map_or(0, |a| a.len());
    json!({ "items": items, "next_cursor": Value::Null, "total_count": n })
}

/// Flat pagination: `{"items": [...], "total": n, "page": 1, "page_size": n, "next_page": null}`.
pub fn page_flat_value(items: &Value) -> Value {
    let n = items.as_array().map_or(0, |a| a.len());
    json!({ "items": items, "total": n, "page": 1, "page_size": n, "next_page": Value::Null })
}

/// Custody-style: `{"items": [...], "total": n, "next_cursor": null}`.
pub fn page_total_cursor_value(items: &Value) -> Value {
    let n = items.as_array().map_or(0, |a| a.len());
    json!({ "items": items, "total": n, "next_cursor": Value::Null })
}

// ---------------------------------------------------------------------------
// Shared in-memory store helpers (TD-03 / TD-08)
// ---------------------------------------------------------------------------

/// Thread-safe in-memory map mirroring the `Arc<Mutex<HashMap>>` pattern
/// duplicated across `AppState` / `spec_state.rs` files.
/// Swap boundary: replace inner `Mutex<HashMap>` with `sqlx::PgPool` + `Store` trait
/// when persistence lands (see `rla` and `custody/src/state.rs`).
#[derive(Debug, Clone)]
pub struct SharedMap<K, V> {
    inner: Arc<Mutex<HashMap<K, V>>>,
}

impl<K, V> Default for SharedMap<K, V> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<K, V> SharedMap<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Insert a value.
    pub fn insert(&self, key: K, value: V) {
        self.inner.lock().expect("lock poisoned").insert(key, value);
    }

    /// Get a cloned value.
    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.lock().expect("lock poisoned").get(key).cloned()
    }

    /// List all values.
    pub fn values(&self) -> Vec<V> {
        self.inner
            .lock()
            .expect("lock poisoned")
            .values()
            .cloned()
            .collect()
    }

    /// Current length.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("lock poisoned").len()
    }

    /// Whether empty.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().expect("lock poisoned").is_empty()
    }

    /// Run a closure with exclusive access to the underlying map.
    /// Use for `entry` / `get_mut` patterns that need mutation.
    pub fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<K, V>) -> R,
    {
        let mut g = self.inner.lock().expect("lock poisoned");
        f(&mut g)
    }
}

/// Example of the legacy `OnceLock<Mutex<HashMap>>` static pattern:
/// ```ignore
/// fn store() -> &'static Mutex<HashMap<String, Value>> {
///     static STORE: OnceLock<Mutex<HashMap<String, Value>>> = OnceLock::new();
///     STORE.get_or_init(|| Mutex::new(HashMap::new()))
/// }
/// ```
/// Prefer `SharedMap` in `AppState`; keep statics only for truly global singletons.

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn page_empty() {
        let v: Vec<i32> = vec![];
        let p = page(v);
        assert_eq!(p["meta"]["total"], 0);
    }

    #[test]
    fn page_with_items() {
        let p = page(vec![1, 2, 3]);
        assert_eq!(p["meta"]["total"], 3);
    }

    #[test]
    fn page_cursor_value_shape() {
        let v = json!([1, 2]);
        let p = page_cursor_value(&v);
        assert_eq!(p["total_count"], 2);
        assert!(p["next_cursor"].is_null());
    }

    #[test]
    fn page_flat_value_shape() {
        let v = json!([1]);
        let p = page_flat_value(&v);
        assert_eq!(p["total"], 1);
        assert_eq!(p["page"], 1);
    }

    #[test]
    fn page_total_cursor_value_shape() {
        let v = json!([]);
        let p = page_total_cursor_value(&v);
        assert_eq!(p["total"], 0);
    }

    #[test]
    fn shared_map_insert_get() {
        let m: SharedMap<String, i32> = SharedMap::new();
        m.insert("a".to_string(), 1);
        assert_eq!(m.get(&"a".to_string()), Some(1));
        assert_eq!(m.len(), 1);
        assert!(!m.is_empty());
    }

    #[test]
    fn shared_map_values() {
        let m: SharedMap<String, i32> = SharedMap::new();
        m.insert("a".to_string(), 1);
        m.insert("b".to_string(), 2);
        let mut v = m.values();
        v.sort();
        assert_eq!(v, vec![1, 2]);
    }
}
