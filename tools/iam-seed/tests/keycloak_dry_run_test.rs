use serde_json::json;

use iam_seed::compute_diff;

/// Verify that when current and desired realm configs match, the diff is empty
/// (dry-run would print nothing and take no action).
#[test]
fn keycloak_dry_run_no_changes_when_equal() {
    let realm = json!({
        "realm": "eel",
        "enabled": true,
        "displayName": "Electronic Electoral Ledger",
        "ssoSessionMaxLifespan": 36000,
        "accessTokenLifespan": 300
    });

    let changes = compute_diff(&realm, &realm);
    assert!(
        changes.is_empty(),
        "dry-run should report no changes when configs match, got: {:?}",
        changes
    );
}

/// Verify that a new client added in the desired config is detected.
#[test]
fn keycloak_dry_run_detects_added_client() {
    let current = json!({
        "realm": "eel",
        "enabled": true,
        "clients": []
    });
    let desired = json!({
        "realm": "eel",
        "enabled": true,
        "clients": [
            { "clientId": "portal-svc", "protocol": "openid-connect" }
        ]
    });

    let changes = compute_diff(&current, &desired);
    assert!(
        !changes.is_empty(),
        "should detect change when a client is added"
    );
    assert!(
        changes.iter().any(|c| c.contains("clients")),
        "diff should mention 'clients', got: {:?}",
        changes
    );
}

/// Verify that changing a realm display name is detected as a modification.
#[test]
fn keycloak_dry_run_detects_display_name_change() {
    let current = json!({ "realm": "eel", "displayName": "Old Name" });
    let desired = json!({ "realm": "eel", "displayName": "Electronic Electoral Ledger" });

    let changes = compute_diff(&current, &desired);
    assert!(
        changes
            .iter()
            .any(|c| c.contains("displayName") && c.starts_with('~')),
        "should detect display name change, got: {:?}",
        changes
    );
}
