//! Tests for bulk operations workers.

use eel_abis_bulk_ops::dedup::{find_duplicates, process_dedup_batch, DuplicatePair};
use eel_abis_bulk_ops::delete::process_delete_batch;
use eel_abis_bulk_ops::dispatcher::{cancel, mark_running, pause, validate_runnable};
use eel_abis_bulk_ops::domain::{
    BulkOpState, BulkOpType, BulkOperation, FailureRecord, ProgressSnapshot,
};
use eel_abis_bulk_ops::enrol::{process_enrol_batch, SubjectRecord};
use eel_abis_bulk_ops::export::export_to_csv;
use eel_abis_bulk_ops::import::parse_watchlist_csv;
use eel_abis_bulk_ops::progress::{mark_completed, record_progress, write_manifest};
use eel_abis_bulk_ops::retire::process_retire_batch;
use eel_abis_bulk_ops::rewrap::process_rewrap_batch;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

fn test_op(op_type: BulkOpType) -> BulkOperation {
    BulkOperation::new(op_type, 100, Uuid::now_v7(), serde_json::json!({}))
}

#[test]
fn bulk_op_starts_pending() {
    let op = test_op(BulkOpType::Enrol);
    assert_eq!(op.state, BulkOpState::Pending);
    assert_eq!(op.succeeded, 0);
}

#[test]
fn mark_running_sets_state() {
    let mut op = test_op(BulkOpType::Enrol);
    mark_running(&mut op);
    assert_eq!(op.state, BulkOpState::Running);
    assert!(op.started_at.is_some());
}

#[test]
fn pause_running_op() {
    let mut op = test_op(BulkOpType::Dedup);
    mark_running(&mut op);
    pause(&mut op).unwrap();
    assert_eq!(op.state, BulkOpState::Paused);
}

#[test]
fn pause_non_running_fails() {
    let mut op = test_op(BulkOpType::Delete);
    assert!(pause(&mut op).is_err());
}

#[test]
fn cancel_pending_op() {
    let mut op = test_op(BulkOpType::Export);
    cancel(&mut op).unwrap();
    assert_eq!(op.state, BulkOpState::Cancelled);
}

#[test]
fn cancel_completed_fails() {
    let mut op = test_op(BulkOpType::Retire);
    mark_running(&mut op);
    mark_completed(&mut op);
    assert!(cancel(&mut op).is_err());
}

#[test]
fn enrol_batch_success_count() {
    let op = test_op(BulkOpType::Enrol);
    let subjects: Vec<SubjectRecord> = (0..100)
        .map(|i| SubjectRecord {
            id: format!("s{}", i),
            payload: vec![],
        })
        .collect();
    let fail_ids = HashSet::new();
    let (ok, failures) = process_enrol_batch(&op, &subjects, 0, &fail_ids);
    assert_eq!(ok, 100);
    assert!(failures.is_empty());
}

#[test]
fn enrol_batch_with_failures() {
    let op = test_op(BulkOpType::Enrol);
    let subjects: Vec<SubjectRecord> = (0..20)
        .map(|i| SubjectRecord {
            id: format!("s{}", i),
            payload: vec![],
        })
        .collect();
    let fail_ids: HashSet<String> = vec!["s0".to_string()].into_iter().collect();
    let (ok, failures) = process_enrol_batch(&op, &subjects, 0, &fail_ids);
    assert_eq!(ok, 19);
    assert_eq!(failures.len(), 1);
}

#[test]
fn dedup_identifies_above_threshold() {
    let pairs = vec![
        ("a".to_string(), "b".to_string(), 0.95),
        ("c".to_string(), "d".to_string(), 0.50),
        ("e".to_string(), "f".to_string(), 0.85),
    ];
    let dups = find_duplicates(&pairs, 0.80);
    assert_eq!(dups.len(), 2);
}

#[test]
fn dedup_no_dups_below_threshold() {
    let pairs = vec![("a".to_string(), "b".to_string(), 0.3)];
    let dups = find_duplicates(&pairs, 0.80);
    assert!(dups.is_empty());
}

#[test]
fn delete_honours_retention() {
    let ids = vec!["s1".to_string(), "s2".to_string(), "s3".to_string()];
    let mut ages = HashMap::new();
    ages.insert("s1".to_string(), 100u32);
    ages.insert("s2".to_string(), 10u32);
    ages.insert("s3".to_string(), 200u32);
    let (ok, failures) = process_delete_batch(&ids, 30, &ages);
    assert_eq!(ok, 2);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].item_id, "s2");
}

#[test]
fn export_produces_valid_csv() {
    let records = vec![
        vec!["id1".to_string(), "Alice".to_string()],
        vec!["id2".to_string(), "Bob".to_string()],
    ];
    let csv = export_to_csv(&records, &["id", "name"]).unwrap();
    let text = String::from_utf8(csv).unwrap();
    assert!(text.contains("id,name"));
    assert!(text.contains("Alice"));
}

#[test]
fn rewrap_batch_success() {
    let ids: Vec<String> = (0..10).map(|i| format!("t{}", i)).collect();
    let fail_ids = HashSet::new();
    let (ok, failures) = process_rewrap_batch(&ids, "kek_old", "kek_new", &fail_ids);
    assert_eq!(ok, 10);
    assert!(failures.is_empty());
}

#[test]
fn import_watchlist_csv_parses_correctly() {
    let csv = b"subject_id,name,national_id,risk_level\ns1,Alice,ID001,high\ns2,Bob,ID002,low\n";
    let entries = parse_watchlist_csv(csv).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name, "Alice");
    assert_eq!(entries[1].risk_level, "low");
}

#[test]
fn retire_soft_deletes() {
    let ids: Vec<String> = (0..5).map(|i| format!("s{}", i)).collect();
    let fail_ids = HashSet::new();
    let (retired, failures) = process_retire_batch(&ids, "election_over", &fail_ids);
    assert_eq!(retired.len(), 5);
    assert!(failures.is_empty());
}

#[test]
fn manifest_csv_serialises() {
    let records = vec![FailureRecord {
        item_id: "s1".to_string(),
        error: "err".to_string(),
        offset: 0,
    }];
    let csv = write_manifest(&records).unwrap();
    let text = String::from_utf8(csv).unwrap();
    assert!(text.contains("item_id"));
    assert!(text.contains("s1"));
}

#[test]
fn validate_runnable_allows_pending_and_paused() {
    let pending = test_op(BulkOpType::Enrol);
    assert!(validate_runnable(&pending).is_ok());
    let mut paused = test_op(BulkOpType::Enrol);
    mark_running(&mut paused);
    pause(&mut paused).unwrap();
    assert!(validate_runnable(&paused).is_ok());
}

#[test]
fn validate_runnable_rejects_completed() {
    let mut op = test_op(BulkOpType::Enrol);
    mark_running(&mut op);
    mark_completed(&mut op);
    assert!(validate_runnable(&op).is_err());
}

#[test]
fn progress_record_updates_counters() {
    let mut op = test_op(BulkOpType::Enrol);
    mark_running(&mut op);
    let op_id = op.id;
    record_progress(
        &mut op,
        ProgressSnapshot {
            op_id,
            offset: 50,
            succeeded: 45,
            failed: 3,
            skipped: 2,
        },
    );
    assert_eq!(op.checkpoint_offset, 50);
    assert_eq!(op.succeeded, 45);
}

#[test]
fn retire_with_failures() {
    let ids: Vec<String> = (0..5).map(|i| format!("s{}", i)).collect();
    let fail_ids: HashSet<String> = vec!["s2".to_string()].into_iter().collect();
    let (retired, failures) = process_retire_batch(&ids, "reason", &fail_ids);
    assert_eq!(retired.len(), 4);
    assert_eq!(failures.len(), 1);
}

#[test]
fn dedup_failure_when_subject_in_fail_set() {
    let pairs = vec![DuplicatePair {
        subject_a: "a".to_string(),
        subject_b: "b".to_string(),
        score: 0.95,
    }];
    let fail_ids: HashSet<String> = vec!["a".to_string()].into_iter().collect();
    let failures = process_dedup_batch(&pairs, &fail_ids);
    assert_eq!(failures.len(), 1);
}
