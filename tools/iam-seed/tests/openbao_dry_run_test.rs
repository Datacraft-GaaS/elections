use iam_seed::compute_diff;
use serde_json::json;

/// Policy diff: identical content produces no changes.
#[test]
fn openbao_policy_no_change_when_equal() {
    let policy = json!({
        "policy": "path \"kv/data/eel/+/portal-svc/*\" { capabilities = [\"read\"] }"
    });
    let changes = compute_diff(&policy, &policy);
    assert!(
        changes.is_empty(),
        "identical policies should produce no diff, got: {:?}",
        changes
    );
}

/// Policy diff: a capability added to the desired policy is detected.
#[test]
fn openbao_policy_change_detected() {
    let current = json!({
        "policy": "path \"kv/data/eel/+/portal-svc/*\" { capabilities = [\"read\"] }"
    });
    let desired = json!({
        "policy": "path \"kv/data/eel/+/portal-svc/*\" { capabilities = [\"read\", \"list\"] }"
    });
    let changes = compute_diff(&current, &desired);
    assert!(!changes.is_empty(), "policy change should be detected");
    assert!(
        changes
            .iter()
            .any(|c| c.contains("policy") && c.starts_with('~')),
        "diff should show policy field changed, got: {:?}",
        changes
    );
}

/// Mount diff: new mount path detected as addition.
#[test]
fn openbao_mount_addition_detected() {
    let current = json!({});
    let desired = json!({
        "kv/eel/dev/": { "type": "kv", "options": { "version": "2" } }
    });
    let changes = compute_diff(&current, &desired);
    assert!(
        changes.iter().any(|c| c.starts_with('+')),
        "new mount should appear as addition, got: {:?}",
        changes
    );
}
