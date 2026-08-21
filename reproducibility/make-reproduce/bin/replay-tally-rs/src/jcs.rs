// jcs.rs -- JSON Canonicalization Scheme (RFC 8785) implemented inline.
//
// Rules:
//   - Object keys sorted lexicographically (Unicode code-point order).
//   - No insignificant whitespace.
//   - Numbers serialized without exponent for integers; floats use shortest
//     representation that round-trips.
//   - Strings: only mandatory JSON escapes (\", \\, \n, \r, \t, \uXXXX for
//     control chars < U+0020).

use std::fmt::Write;

pub type JcsError = String;

pub fn canonicalize(value: &serde_json::Value) -> Result<String, JcsError> {
    let mut out = String::new();
    write_value(&mut out, value)?;
    Ok(out)
}

fn write_value(out: &mut String, value: &serde_json::Value) -> Result<(), JcsError> {
    match value {
        serde_json::Value::Null => out.push_str("null"),
        serde_json::Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                write!(out, "{i}").map_err(|e| e.to_string())?;
            } else if let Some(u) = n.as_u64() {
                write!(out, "{u}").map_err(|e| e.to_string())?;
            } else if let Some(f) = n.as_f64() {
                if f.is_infinite() || f.is_nan() {
                    return Err(format!("non-finite float in JCS: {f}"));
                }
                // Use Rust's default float formatting which gives shortest round-trip.
                let s = format!("{f}");
                out.push_str(&s);
            } else {
                return Err(format!("unknown number: {n}"));
            }
        }
        serde_json::Value::String(s) => {
            write_string(out, s)?;
        }
        serde_json::Value::Array(arr) => {
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item)?;
            }
            out.push(']');
        }
        serde_json::Value::Object(map) => {
            out.push('{');
            // Sort keys by Unicode code-point order (String's default Ord).
            let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
            keys.sort_unstable();
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(out, key)?;
                out.push(':');
                write_value(out, &map[*key])?;
            }
            out.push('}');
        }
    }
    Ok(())
}

fn write_string(out: &mut String, s: &str) -> Result<(), JcsError> {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                write!(out, "\\u{:04x}", c as u32).map_err(|e| e.to_string())?;
            }
            c => out.push(c),
        }
    }
    out.push('"');
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_object_keys() {
        let v: serde_json::Value = serde_json::from_str(r#"{"z":1,"a":2,"m":3}"#).unwrap();
        assert_eq!(canonicalize(&v).unwrap(), r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn null_bool() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"x":null,"y":true,"z":false}"#).unwrap();
        let c = canonicalize(&v).unwrap();
        assert!(c.contains("null"));
        assert!(c.contains("true"));
        assert!(c.contains("false"));
    }
}
