//! Power-cut crash-safety property test (EC-OPS-05, M09-PROP-POWER-001).
//!
//! Simulates:
//!   1. RO begins a Form 34B compile — action enqueued in SQLite WAL, status = 'pending'.
//!   2. Power cut before backend confirms — connection dropped without commit.
//!   3. App restarts, reopens WAL database.
//!   4. Pending actions are visible for resume (no silent loss).
//!   5. Resume completes the action — status transitions to 'committed'.
//!   6. A second compile for the same constituency raises Duplicate34B (no double-emit).

use ro_desktop_lib::db::{commit_action, enqueue_action, open_db, pending_actions, record_form34b};
use ro_desktop_lib::error::AppError;
use uuid::Uuid;

async fn tmp_db() -> (sqlx::SqlitePool, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("test.db");
    let pool = open_db(path.to_str().unwrap()).await.expect("open_db");
    (pool, dir)
}

#[tokio::test]
async fn pending_action_survives_simulated_power_cut() {
    let (pool, _dir) = tmp_db().await;
    let idem = Uuid::new_v4();

    // Step 1: enqueue (journal-before-act).
    let row_id = enqueue_action(
        &pool,
        "compile_form34b",
        &serde_json::json!({ "constituency_code": "NAI-001", "election_id": "GE-2027" }),
        idem,
    )
    .await
    .expect("enqueue");

    // Step 2: simulate power cut — do NOT call commit_action.
    // Drop connection and reopen (WAL guarantees durability of the INSERT).
    drop(pool);

    // Reopen same DB file (WAL replay on open).
    let dir_path = _dir.path().join("test.db");
    let pool2 = open_db(dir_path.to_str().unwrap())
        .await
        .expect("reopen after power cut");

    // Step 3: pending action must be present for resume.
    let pending = pending_actions(&pool2).await.expect("pending_actions");
    assert_eq!(pending.len(), 1, "one action must survive power cut");
    assert_eq!(pending[0].id, row_id);
    assert_eq!(pending[0].action_type, "compile_form34b");

    // Step 4: resume — commit the action.
    commit_action(&pool2, &row_id).await.expect("commit");

    // Step 5: no more pending actions.
    let after_commit = pending_actions(&pool2).await.expect("pending after commit");
    assert!(after_commit.is_empty(), "no pending actions after commit");
}

#[tokio::test]
async fn no_duplicate_34b_on_resume() {
    let (pool, _dir) = tmp_db().await;

    // First compile: succeeds and records in form34b_log.
    let result1 = record_form34b(&pool, "NAI-001", "GE-2027", "PRES").await;
    assert!(result1.is_ok(), "first 34B compile must succeed: {:?}", result1);

    // Second compile for same constituency+election+position must be rejected.
    let result2 = record_form34b(&pool, "NAI-001", "GE-2027", "PRES").await;
    match result2 {
        Err(AppError::Duplicate34B(code)) => {
            assert_eq!(code, "NAI-001", "duplicate error must name the constituency");
        }
        other => panic!("expected Duplicate34B, got {:?}", other),
    }
}

#[tokio::test]
async fn different_positions_not_duplicate() {
    let (pool, _dir) = tmp_db().await;

    // President and Governor 34Bs for the same constituency are independent.
    let r1 = record_form34b(&pool, "NAI-001", "GE-2027", "PRES").await;
    let r2 = record_form34b(&pool, "NAI-001", "GE-2027", "GOV").await;
    assert!(r1.is_ok(), "PRES 34B: {:?}", r1);
    assert!(r2.is_ok(), "GOV 34B must not conflict with PRES: {:?}", r2);
}

#[tokio::test]
async fn idempotency_key_uniqueness_enforced() {
    let (pool, _dir) = tmp_db().await;
    let idem = Uuid::new_v4();

    enqueue_action(
        &pool,
        "verify_form34a",
        &serde_json::json!({"form_id": "F-001"}),
        idem,
    )
    .await
    .expect("first enqueue");

    // Same idempotency key must be rejected (UNIQUE constraint).
    let result = enqueue_action(
        &pool,
        "verify_form34a",
        &serde_json::json!({"form_id": "F-001"}),
        idem,
    )
    .await;
    assert!(result.is_err(), "duplicate idempotency key must fail");
}
