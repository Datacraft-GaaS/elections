//! Tests for model canary rollout logic.

use eel_abis_model_canary::canary::{next_step, start_canary};
use eel_abis_model_canary::dispatcher::{route_to_canary, select_model};
use eel_abis_model_canary::domain::{CanaryMetrics, CanaryState, RollbackReason};
use eel_abis_model_canary::monitor::check_sla;
use eel_abis_model_canary::promote::promote;
use eel_abis_model_canary::rollback::rollback;
use uuid::Uuid;

fn default_canary() -> eel_abis_model_canary::domain::CanaryRecord {
    let from = Uuid::now_v7();
    let to = Uuid::now_v7();
    start_canary(from, to, 10, 0.01, 0.05, 500.0, 0.01).unwrap()
}

#[test]
fn start_canary_creates_in_progress_record() {
    let c = default_canary();
    assert_eq!(c.state, CanaryState::InProgress);
    assert_eq!(c.traffic_pct, 10);
}

#[test]
fn start_canary_invalid_pct_errors() {
    let from = Uuid::now_v7();
    let to = Uuid::now_v7();
    assert!(start_canary(from, to, 101, 0.01, 0.05, 500.0, 0.01).is_err());
}

#[test]
fn next_step_correct_progression() {
    assert_eq!(next_step(0), Some(10));
    assert_eq!(next_step(10), Some(25));
    assert_eq!(next_step(25), Some(50));
    assert_eq!(next_step(50), Some(100));
    assert_eq!(next_step(100), None);
}

#[test]
fn dispatch_0_pct_always_baseline() {
    let from = Uuid::now_v7();
    let to = Uuid::now_v7();
    let mut c = start_canary(from, to, 0, 0.01, 0.05, 500.0, 0.01).unwrap();
    c.traffic_pct = 0;
    for _ in 0..100 {
        assert!(!route_to_canary(Uuid::now_v7(), &c));
    }
}

#[test]
fn dispatch_100_pct_always_canary() {
    let from = Uuid::now_v7();
    let to = Uuid::now_v7();
    let mut c = start_canary(from, to, 10, 0.01, 0.05, 500.0, 0.01).unwrap();
    c.traffic_pct = 100;
    for _ in 0..100 {
        assert!(route_to_canary(Uuid::now_v7(), &c));
    }
}

#[test]
fn dispatch_deterministic_for_same_subject() {
    let c = default_canary();
    let subject = Uuid::now_v7();
    let first = route_to_canary(subject, &c);
    for _ in 0..10 {
        assert_eq!(route_to_canary(subject, &c), first);
    }
}

#[test]
fn dispatch_traffic_roughly_proportional() {
    let from = Uuid::now_v7();
    let to = Uuid::now_v7();
    let mut c = start_canary(from, to, 50, 0.01, 0.05, 500.0, 0.01).unwrap();
    c.traffic_pct = 50;
    let routed: usize = (0..1000)
        .filter(|_| route_to_canary(Uuid::now_v7(), &c))
        .count();
    assert!(routed > 350 && routed < 650, "got {routed}");
}

#[test]
fn select_model_returns_baseline_when_no_canary() {
    let baseline = Uuid::now_v7();
    let subject = Uuid::now_v7();
    assert_eq!(select_model(subject, baseline, None), baseline);
}

#[test]
fn promote_sets_state_and_pct() {
    let mut c = default_canary();
    promote(&mut c).unwrap();
    assert_eq!(c.state, CanaryState::Promoted);
    assert_eq!(c.traffic_pct, 100);
    assert!(c.promoted_at.is_some());
}

#[test]
fn promote_fails_on_non_in_progress() {
    let mut c = default_canary();
    promote(&mut c).unwrap();
    assert!(promote(&mut c).is_err());
}

#[test]
fn rollback_sets_state_to_rolled_back() {
    let mut c = default_canary();
    rollback(
        &mut c,
        RollbackReason::Manual {
            operator: "ops".to_string(),
            reason: "test".to_string(),
        },
    )
    .unwrap();
    assert_eq!(c.state, CanaryState::RolledBack);
    assert_eq!(c.traffic_pct, 0);
    assert!(c.rolled_back_at.is_some());
    assert!(c.rollback_reason.is_some());
}

#[test]
fn rollback_idempotent_when_already_rolled_back() {
    let mut c = default_canary();
    rollback(
        &mut c,
        RollbackReason::Manual {
            operator: "a".to_string(),
            reason: "b".to_string(),
        },
    )
    .unwrap();
    assert!(rollback(
        &mut c,
        RollbackReason::Manual {
            operator: "c".to_string(),
            reason: "d".to_string()
        }
    )
    .is_ok());
}

#[test]
fn monitor_no_breach_healthy_metrics() {
    let c = default_canary();
    let m = CanaryMetrics {
        fmr: 0.001,
        fnmr: 0.01,
        latency_p99_ms: 100.0,
        error_rate: 0.001,
        ..Default::default()
    };
    assert!(check_sla(&c, &m).is_none());
}

#[test]
fn monitor_fmr_breach_detected() {
    let c = default_canary();
    let m = CanaryMetrics {
        fmr: 0.5,
        ..Default::default()
    };
    let reason = check_sla(&c, &m);
    assert!(matches!(reason, Some(RollbackReason::FmrBreach { .. })));
}

#[test]
fn monitor_latency_breach_detected() {
    let c = default_canary();
    let m = CanaryMetrics {
        latency_p99_ms: 9999.0,
        ..Default::default()
    };
    let reason = check_sla(&c, &m);
    assert!(matches!(reason, Some(RollbackReason::LatencyBreach { .. })));
}

#[test]
fn rollback_preserves_history() {
    let mut c = default_canary();
    let started_at = c.started_at;
    let from_id = c.from_model_id;
    rollback(
        &mut c,
        RollbackReason::Manual {
            operator: "ops".to_string(),
            reason: "test".to_string(),
        },
    )
    .unwrap();
    assert_eq!(c.started_at, started_at);
    assert_eq!(c.from_model_id, from_id);
}
