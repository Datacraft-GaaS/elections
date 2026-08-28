//! Shared helpers for spec contract handlers.
//! Extracted from duplicated boilerplate across 31 `spec.rs` files (TD-08).

use serde::Serialize;
use serde_json::{json, Value};

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
}
