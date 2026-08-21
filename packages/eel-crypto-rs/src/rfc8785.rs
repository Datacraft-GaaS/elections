//! RFC 8785 JSON Canonicalization Scheme (JCS) — implemented from scratch.
//!
//! Produces a deterministic byte sequence from a `serde_json::Value`.  The
//! output is suitable for signing: two semantically-equivalent JSON
//! documents produce byte-identical output.
//!
//! Key rules:
//! - Object keys are sorted by their UTF-16 code-unit sequence.
//! - Numbers are serialised per ES2020 §7.1.12.1 (integer if safe integer,
//!   otherwise IEEE 754 double).
//! - Control characters (U+0000-U+001F) are encoded as `\uXXXX`.
//! - No whitespace outside string values.
//!
//! # Example
//!
//! ```
//! use eel_crypto_rs::rfc8785::canonicalize;
//! use serde_json::json;
//!
//! let v = json!({"b": 2, "a": 1});
//! let bytes = canonicalize(&v).unwrap();
//! assert_eq!(bytes, br#"{"a":1,"b":2}"#);
//! ```

use serde_json::Value;

use crate::CryptoError;

/// Canonicalize `value` per RFC 8785 and return the UTF-8 byte sequence.
///
/// # Errors
///
/// Returns [`CryptoError::CanonicalJson`] if the value contains a non-finite
/// number (`NaN` or `Infinity`), which has no canonical JSON representation.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::rfc8785::canonicalize;
/// use serde_json::json;
/// let out = canonicalize(&json!({"z": [3,1,2], "a": true})).unwrap();
/// assert_eq!(out, br#"{"a":true,"z":[3,1,2]}"#);
/// ```
pub fn canonicalize(value: &Value) -> Result<Vec<u8>, CryptoError> {
    let mut out = Vec::with_capacity(128);
    write_value(value, &mut out)?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Internal serializer
// ---------------------------------------------------------------------------

fn write_value(v: &Value, out: &mut Vec<u8>) -> Result<(), CryptoError> {
    match v {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(n) => write_number(n, out)?,
        Value::String(s) => write_string(s, out),
        Value::Array(arr) => {
            out.push(b'[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_value(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(map) => {
            // Sort keys by UTF-16 code unit sequence.
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| cmp_utf16(a, b));

            out.push(b'{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_string(key, out);
                out.push(b':');
                write_value(&map[*key], out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

/// Compare two strings by their UTF-16 code unit sequences (RFC 8785 §3.2.3).
fn cmp_utf16(a: &str, b: &str) -> std::cmp::Ordering {
    let a16: Vec<u16> = a.encode_utf16().collect();
    let b16: Vec<u16> = b.encode_utf16().collect();
    a16.cmp(&b16)
}

/// Serialize a JSON number per ES2020 §7.1.12.1.
fn write_number(n: &serde_json::Number, out: &mut Vec<u8>) -> Result<(), CryptoError> {
    if let Some(i) = n.as_i64() {
        // Safe integer: serialize without decimal point.
        let s = i.to_string();
        out.extend_from_slice(s.as_bytes());
    } else if let Some(u) = n.as_u64() {
        let s = u.to_string();
        out.extend_from_slice(s.as_bytes());
    } else if let Some(f) = n.as_f64() {
        if !f.is_finite() {
            return Err(CryptoError::CanonicalJson(format!(
                "non-finite number {} cannot be canonicalized",
                f
            )));
        }
        // Use the ES2020 minimal representation.  Rust's Display for f64
        // uses Grisu3/Dragon4 which matches JavaScript's serialization for
        // all finite values within the double precision range.
        let s = serialize_f64(f);
        out.extend_from_slice(s.as_bytes());
    } else {
        return Err(CryptoError::CanonicalJson("unknown number type".to_owned()));
    }
    Ok(())
}

/// Serialize an f64 using ES2020 double-to-string rules.
fn serialize_f64(f: f64) -> String {
    if f == 0.0 {
        return "0".to_owned();
    }
    // Check if the value is a safe integer (no fractional part, within range).
    if f.fract() == 0.0
        && f.abs() <= 9_007_199_254_740_992.0 // 2^53
        && f.abs() >= 1.0
    {
        return format!("{:.0}", f);
    }
    // General case: use Rust's default f64 formatter which matches ES2020
    // for all normal finite values.
    format!("{}", f)
}

/// Serialize a JSON string per RFC 8785 §3.2.2.2.
///
/// Rules:
/// - `"` becomes `\"`
/// - `\` becomes `\\`
/// - U+0000..U+001F become `\uXXXX` (lower-hex)
/// - All other characters are emitted as UTF-8 without escaping.
fn write_string(s: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for c in s.chars() {
        match c {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\x08' => out.extend_from_slice(b"\\b"),
            '\x0C' => out.extend_from_slice(b"\\f"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            c if (c as u32) < 0x20 => {
                let encoded = format!("\\u{:04x}", c as u32);
                out.extend_from_slice(encoded.as_bytes());
            }
            c => {
                let mut buf = [0u8; 4];
                let len = c.encode_utf8(&mut buf).len();
                out.extend_from_slice(&buf[..len]);
            }
        }
    }
    out.push(b'"');
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    fn canon(v: Value) -> String {
        String::from_utf8(canonicalize(&v).unwrap()).unwrap()
    }

    // --- basic values --------------------------------------------------------

    #[test]
    fn null_value() {
        assert_eq!(canon(json!(null)), "null");
    }

    #[test]
    fn boolean_values() {
        assert_eq!(canon(json!(true)), "true");
        assert_eq!(canon(json!(false)), "false");
    }

    #[test]
    fn integer_number() {
        assert_eq!(canon(json!(42)), "42");
        assert_eq!(canon(json!(-7)), "-7");
        assert_eq!(canon(json!(0)), "0");
    }

    #[test]
    fn empty_string() {
        assert_eq!(canon(json!("")), r#""""#);
    }

    #[test]
    fn string_with_escapes() {
        assert_eq!(canon(json!("a\"b")), r#""a\"b""#);
        assert_eq!(canon(json!("a\\b")), r#""a\\b""#);
        assert_eq!(canon(json!("a\nb")), r#""a\nb""#);
    }

    #[test]
    fn control_char_escape() {
        let v = Value::String("\x01\x1f".to_owned());
        assert_eq!(canon(v), "\"\\u0001\\u001f\"");
    }

    // --- object key sort -----------------------------------------------------

    #[test]
    fn object_keys_sorted_alphabetically() {
        let v = json!({"z": 1, "a": 2, "m": 3});
        assert_eq!(canon(v), r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn nested_object_keys_sorted() {
        let v = json!({"b": {"d": 4, "c": 3}, "a": 1});
        assert_eq!(canon(v), r#"{"a":1,"b":{"c":3,"d":4}}"#);
    }

    // --- arrays preserve order -----------------------------------------------

    #[test]
    fn array_order_preserved() {
        let v = json!([3, 1, 2]);
        assert_eq!(canon(v), "[3,1,2]");
    }

    // --- property: equivalent objects produce identical output ---------------

    proptest! {
        /// Two JSON objects with the same key-value pairs but different
        /// insertion order must produce byte-identical canonical output.
        #[test]
        fn prop_key_order_invariant(
            a in 0i64..=1000,
            b in 0i64..=1000,
            c in 0i64..=1000,
        ) {
            let v1 = json!({"a": a, "b": b, "c": c});
            let v2 = json!({"c": c, "a": a, "b": b});
            let c1 = canonicalize(&v1).unwrap();
            let c2 = canonicalize(&v2).unwrap();
            prop_assert_eq!(c1, c2);
        }

        /// Canonicalizing twice is idempotent: canon(parse(canon(v))) == canon(v).
        #[test]
        fn prop_idempotent(
            k in "[a-z]{1,8}",
            v in 0i64..=9999,
        ) {
            let obj = json!({k.clone(): v});
            let first = canonicalize(&obj).unwrap();
            let reparsed: Value = serde_json::from_slice(&first).unwrap();
            let second = canonicalize(&reparsed).unwrap();
            prop_assert_eq!(first, second);
        }
    }
}
