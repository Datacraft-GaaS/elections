//! Prometheus counters for RFID cover custody.

use prometheus::{register_counter_vec, CounterVec, Opts};
use std::sync::OnceLock;

static COVER_READS_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static COVER_TAMPER_ALERTS_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static COVER_STATE_TRANSITIONS_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static COVER_HMAC_FAILURES_TOTAL: OnceLock<CounterVec> = OnceLock::new();

/// Initialise all metrics counters. Call once at service startup.
pub fn init() {
    COVER_READS_TOTAL.get_or_init(|| {
        register_counter_vec!(
            Opts::new(
                "eel_cover_reads_total",
                "Total RFID cover reads, labelled by reader_type"
            ),
            &["reader_type"]
        )
        .expect("eel_cover_reads_total registration failed")
    });

    COVER_TAMPER_ALERTS_TOTAL.get_or_init(|| {
        register_counter_vec!(
            Opts::new(
                "eel_cover_tamper_alerts_total",
                "Total tamper alerts, labelled by severity"
            ),
            &["severity"]
        )
        .expect("eel_cover_tamper_alerts_total registration failed")
    });

    COVER_STATE_TRANSITIONS_TOTAL.get_or_init(|| {
        register_counter_vec!(
            Opts::new(
                "eel_cover_state_transitions_total",
                "Total cover state transitions, labelled by from_to"
            ),
            &["from_to"]
        )
        .expect("eel_cover_state_transitions_total registration failed")
    });

    COVER_HMAC_FAILURES_TOTAL.get_or_init(|| {
        register_counter_vec!(
            Opts::new(
                "eel_cover_hmac_failures_total",
                "Total HMAC challenge failures, labelled by reader_type"
            ),
            &["reader_type"]
        )
        .expect("eel_cover_hmac_failures_total registration failed")
    });
}

pub fn inc_cover_reads(reader_type: &str) {
    if let Some(c) = COVER_READS_TOTAL.get() {
        c.with_label_values(&[reader_type]).inc();
    }
}

pub fn inc_tamper_alerts(severity: &str) {
    if let Some(c) = COVER_TAMPER_ALERTS_TOTAL.get() {
        c.with_label_values(&[severity]).inc();
    }
}

pub fn inc_state_transitions(from_state: &str, to_state: &str) {
    if let Some(c) = COVER_STATE_TRANSITIONS_TOTAL.get() {
        let label = format!("{from_state}_to_{to_state}");
        c.with_label_values(&[&label]).inc();
    }
}

pub fn inc_hmac_failures(reader_type: &str) {
    if let Some(c) = COVER_HMAC_FAILURES_TOTAL.get() {
        c.with_label_values(&[reader_type]).inc();
    }
}
