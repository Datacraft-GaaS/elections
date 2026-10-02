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

use iam_seed::compute_diff;
use serde_json::json;

/// Schema diff: identical schemas produce no changes.
#[test]
fn spicedb_no_change_when_equal() {
    let schema = json!({
        "schema": "definition voter_registration { relation owner: voter }"
    });
    let changes = compute_diff(&schema, &schema);
    assert!(
        changes.is_empty(),
        "identical schemas should produce no diff, got: {:?}",
        changes
    );
}

/// Schema diff: added definition is detected.
#[test]
fn spicedb_addition_detected() {
    let current = json!({ "schema": "definition voter_registration { relation owner: voter }" });
    let desired = json!({
        "schema": "definition voter_registration { relation owner: voter }\ndefinition ballot { relation cast_by: voter }"
    });
    let changes = compute_diff(&current, &desired);
    assert!(!changes.is_empty(), "schema addition should be detected");
}

/// Schema diff: removal of a definition is detected.
#[test]
fn spicedb_removal_detected() {
    let current = json!({
        "schema": "definition voter_registration { relation owner: voter }\ndefinition audit_log { relation reader: auditor }"
    });
    let desired = json!({
        "schema": "definition voter_registration { relation owner: voter }"
    });
    let changes = compute_diff(&current, &desired);
    assert!(!changes.is_empty(), "schema removal should be detected");
    assert!(
        changes
            .iter()
            .any(|c| c.contains("schema") && c.starts_with('~')),
        "diff should note schema field changed, got: {:?}",
        changes
    );
}
