/// Compute a diff between two JSON values.
/// Returns human-readable change descriptions:
///   `+ key: value`    -- key present in desired but not in current
///   `- key: (removed)` -- key present in current but not in desired
///   `~ key: old -> new` -- value changed
pub fn compute_diff(current: &serde_json::Value, desired: &serde_json::Value) -> Vec<String> {
    let mut changes = Vec::new();
    diff_values("", current, desired, &mut changes);
    changes
}

fn diff_values(
    path: &str,
    current: &serde_json::Value,
    desired: &serde_json::Value,
    out: &mut Vec<String>,
) {
    use serde_json::Value;

    match (current, desired) {
        (Value::Object(cur_map), Value::Object(des_map)) => {
            for (key, des_val) in des_map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                match cur_map.get(key) {
                    None => out.push(format!("+ {}: {}", child_path, des_val)),
                    Some(cur_val) => diff_values(&child_path, cur_val, des_val, out),
                }
            }
            for key in cur_map.keys() {
                if !des_map.contains_key(key) {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    out.push(format!("- {}: (removed)", child_path));
                }
            }
        }
        (Value::Array(cur_arr), Value::Array(des_arr)) => {
            if cur_arr.len() != des_arr.len() {
                out.push(format!(
                    "~ {}: array length {} -> {}",
                    path,
                    cur_arr.len(),
                    des_arr.len()
                ));
            }
        }
        (cur, des) => {
            if cur != des {
                out.push(format!("~ {}: {} -> {}", path, cur, des));
            }
        }
    }
}
