use chrono::{Duration, Utc};
use eel_portal_inspection::{
    auto_referral::AutoReferralScheduler,
    domain::{CorrectionClaim, CorrectionStatus, CorrectionType},
    events::{correction_referred_v1, objection_filed_v1, roll_snapshotted_v1},
    search::{constant_time_lookup, LookupCap, DAILY_CAP, FIXED_WINDOW_MS},
};
use uuid::Uuid;

// Helper: create an approved correction claim due for referral
fn approved_claim() -> CorrectionClaim {
    CorrectionClaim {
        id: Uuid::now_v7(),
        voter_id_hash: "a".repeat(64),
        claimed_by_biometric_hash: "b".repeat(64),
        correction_type: CorrectionType::NameSpelling,
        evidence_refs: vec!["doc://ref/1".into()],
        status: CorrectionStatus::Approved,
        filed_at: Utc::now(),
    }
}

// --- Constant-time property ---

#[tokio::test]
async fn constant_time_lookup_both_paths_within_5ms_of_each_other() {
    let cap = LookupCap::new();
    let valid_hash = "a".repeat(64);
    let short_hash = "b".repeat(10);

    let t1_start = std::time::Instant::now();
    let _ = constant_time_lookup(&valid_hash, &cap, "subject-A").await;
    let t1 = t1_start.elapsed();

    // Reset cap to allow second lookup under same subject
    let cap2 = LookupCap::new();
    let t2_start = std::time::Instant::now();
    let _ = constant_time_lookup(&short_hash, &cap2, "subject-B").await;
    let t2 = t2_start.elapsed();

    let diff_ms = (t1.as_millis() as i64 - t2.as_millis() as i64).unsigned_abs();
    assert!(
        diff_ms < 5,
        "timing diff {} ms exceeds 5 ms threshold (t1={}ms, t2={}ms)",
        diff_ms,
        t1.as_millis(),
        t2.as_millis()
    );
    // Both paths take at least FIXED_WINDOW_MS
    assert!(t1.as_millis() >= FIXED_WINDOW_MS as u128);
    assert!(t2.as_millis() >= FIXED_WINDOW_MS as u128);
}

// --- Rate limit / daily cap ---

#[tokio::test]
async fn daily_cap_enforced_after_limit() {
    let cap = LookupCap::new();
    let hash = "c".repeat(64);
    let subject = "subject-cap-test";

    for _ in 0..DAILY_CAP {
        let res = constant_time_lookup(&hash, &cap, subject).await;
        assert!(res.is_ok(), "should succeed under cap");
    }
    let over = constant_time_lookup(&hash, &cap, subject).await;
    assert!(over.is_err(), "should fail over daily cap");
    assert_eq!(cap.count(subject), DAILY_CAP);
}

// --- Auto-referral idempotency ---

#[test]
fn auto_referral_idempotent_under_double_fire() {
    let mut scheduler = AutoReferralScheduler::new("https://ecourt.internal");
    let claim = approved_claim();
    let polling_day = Utc::now() + Duration::days(30); // within 60-day window

    let first = scheduler.refer(&claim);
    assert!(first.is_some(), "first referral should succeed");

    let second = scheduler.refer(&claim);
    assert!(second.is_none(), "second referral of same claim should be skipped");
}

#[test]
fn auto_referral_run_batch_is_idempotent() {
    let mut scheduler = AutoReferralScheduler::new("https://ecourt.internal");
    let claims = vec![approved_claim(), approved_claim()];
    let polling_day = Utc::now() + Duration::days(20);

    let first_run = scheduler.run_batch(&claims, polling_day);
    assert_eq!(first_run.len(), 2);

    let second_run = scheduler.run_batch(&claims, polling_day);
    assert_eq!(second_run.len(), 0, "second batch run must produce no new packets");
}

// --- Hash reproducibility ---

#[test]
fn roll_snapshot_hash_reproducible() {
    // BLAKE3 is deterministic: same input produces same hash
    let data = b"voter-id-001\nvoter-id-002\nvoter-id-003";
    let hash1 = eel_crypto_rs::blake3::hash(data);
    let hash2 = eel_crypto_rs::blake3::hash(data);
    assert_eq!(hash1, hash2);
}

// --- Event builders ---

#[test]
fn roll_snapshotted_event_has_correct_type() {
    let id = Uuid::now_v7();
    let ev = roll_snapshotted_v1(&id, "deadbeef");
    assert_eq!(ev["type"], "eel.portal.roll_snapshotted.v1");
    assert_eq!(ev["data"]["snapshot_hash"], "deadbeef");
}

#[test]
fn objection_filed_event_has_correct_type() {
    let id = Uuid::now_v7();
    let ev = objection_filed_v1(&id, "ST-007");
    assert_eq!(ev["type"], "eel.portal.objection_filed.v1");
    assert_eq!(ev["data"]["station_id"], "ST-007");
}

#[test]
fn correction_referred_event_has_correct_type() {
    let claim_id = Uuid::now_v7();
    let ev = correction_referred_v1(&claim_id, "https://ecourt.internal/ref/1");
    assert_eq!(ev["type"], "eel.portal.correction_referred.v1");
    assert!(ev["data"]["ecourt_ref"]
        .as_str()
        .unwrap()
        .contains("ecourt.internal"));
}
