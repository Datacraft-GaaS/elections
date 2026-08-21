//! Prometheus metrics for the HSM signer service.

use prometheus::{register_int_counter_vec, register_int_gauge, IntCounterVec, IntGauge};
use std::sync::OnceLock;

pub struct HsmMetrics {
    pub sign_total: IntCounterVec,
    pub sign_error_total: IntCounterVec,
    pub ceremony_started_total: IntCounterVec,
    pub ceremony_complete_total: IntCounterVec,
    pub ceremony_aborted_total: IntCounterVec,
    pub rate_limited_total: IntCounterVec,
    pub active_ceremonies: IntGauge,
}

static METRICS: OnceLock<HsmMetrics> = OnceLock::new();

pub fn metrics() -> &'static HsmMetrics {
    METRICS.get_or_init(|| HsmMetrics {
        sign_total: register_int_counter_vec!(
            "eel_hsm_sign_total",
            "Total sign operations",
            &["role"]
        )
        .unwrap(),
        sign_error_total: register_int_counter_vec!(
            "eel_hsm_sign_error_total",
            "Total sign errors",
            &["role"]
        )
        .unwrap(),
        ceremony_started_total: register_int_counter_vec!(
            "eel_hsm_ceremony_started_total",
            "Total ceremonies started",
            &["role"]
        )
        .unwrap(),
        ceremony_complete_total: register_int_counter_vec!(
            "eel_hsm_ceremony_complete_total",
            "Total ceremonies completed successfully",
            &["role"]
        )
        .unwrap(),
        ceremony_aborted_total: register_int_counter_vec!(
            "eel_hsm_ceremony_aborted_total",
            "Total ceremonies aborted",
            &["role"]
        )
        .unwrap(),
        rate_limited_total: register_int_counter_vec!(
            "eel_hsm_rate_limited_total",
            "Total requests rejected by rate limiter",
            &["role"]
        )
        .unwrap(),
        active_ceremonies: register_int_gauge!(
            "eel_hsm_active_ceremonies",
            "Currently active ceremonies"
        )
        .unwrap(),
    })
}
