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
}
