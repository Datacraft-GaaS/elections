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

use serde_json::json;

// Import the public compute_diff function from the binary's lib surface.
// Since main.rs exposes it as pub fn, we reference it via the crate name.
use iam_seed::compute_diff;

#[test]
fn empty_diff_when_equal() {
    let v = json!({ "realm": "eel", "enabled": true });
    let changes = compute_diff(&v, &v);
    assert!(
        changes.is_empty(),
        "expected no changes, got: {:?}",
        changes
    );
}

#[test]
fn added_key_detected() {
    let current = json!({ "realm": "eel" });
    let desired = json!({ "realm": "eel", "displayName": "EEL" });
    let changes = compute_diff(&current, &desired);
    assert!(
        changes
            .iter()
            .any(|c| c.contains("displayName") && c.starts_with('+')),
        "expected added key in diff, got: {:?}",
        changes
    );
}

#[test]
fn removed_key_detected() {
    let current = json!({ "realm": "eel", "displayName": "EEL" });
    let desired = json!({ "realm": "eel" });
    let changes = compute_diff(&current, &desired);
    assert!(
        changes
            .iter()
            .any(|c| c.contains("displayName") && c.starts_with('-')),
        "expected removed key in diff, got: {:?}",
        changes
    );
}

#[test]
fn changed_value_detected() {
    let current = json!({ "realm": "eel", "enabled": false });
    let desired = json!({ "realm": "eel", "enabled": true });
    let changes = compute_diff(&current, &desired);
    assert!(
        changes
            .iter()
            .any(|c| c.contains("enabled") && c.starts_with('~')),
        "expected changed value in diff, got: {:?}",
        changes
    );
}

#[test]
fn nested_object_diff() {
    let current = json!({ "smtp": { "host": "mail.old.internal", "port": 25 } });
    let desired = json!({ "smtp": { "host": "mail.eel.internal", "port": 25 } });
    let changes = compute_diff(&current, &desired);
    assert!(
        changes
            .iter()
            .any(|c| c.contains("smtp.host") && c.starts_with('~')),
        "expected nested change in diff, got: {:?}",
        changes
    );
    assert!(
        !changes.iter().any(|c| c.contains("smtp.port")),
        "unchanged port should not appear in diff, got: {:?}",
        changes
    );
}

#[test]
fn array_length_change_detected() {
    let current = json!({ "roles": ["admin", "viewer"] });
    let desired = json!({ "roles": ["admin", "viewer", "editor"] });
    let changes = compute_diff(&current, &desired);
    assert!(
        changes.iter().any(|c| c.contains("roles")),
        "expected array change in diff, got: {:?}",
        changes
    );
}
