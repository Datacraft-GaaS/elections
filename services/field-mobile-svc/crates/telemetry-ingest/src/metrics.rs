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

//! Prometheus counters for telemetry ingest.

use prometheus::{register_counter, register_counter_vec, Counter, CounterVec, Opts};
use std::sync::OnceLock;

static SIGNALS_INGESTED_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static ALERTS_RAISED_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static HMAC_FAILURES_TOTAL: OnceLock<Counter> = OnceLock::new();
static REPLAY_REJECTIONS_TOTAL: OnceLock<Counter> = OnceLock::new();
static MERKLE_SNAPSHOTS_TOTAL: OnceLock<Counter> = OnceLock::new();

/// Initialise all metric counters. Call once at service startup.
pub fn init() {
    SIGNALS_INGESTED_TOTAL.get_or_init(|| {
        register_counter_vec!(
            Opts::new(
                "eel_telemetry_signals_ingested_total",
                "Total telemetry signals ingested, labelled by class"
            ),
            &["class"]
        )
        .expect("eel_telemetry_signals_ingested_total registration failed")
    });

    ALERTS_RAISED_TOTAL.get_or_init(|| {
        register_counter_vec!(
            Opts::new(
                "eel_telemetry_alerts_raised_total",
                "Total alerts raised, labelled by class and severity"
            ),
            &["class", "severity"]
        )
        .expect("eel_telemetry_alerts_raised_total registration failed")
    });

    HMAC_FAILURES_TOTAL.get_or_init(|| {
        register_counter!(Opts::new(
            "eel_telemetry_hmac_failures_total",
            "Total HMAC verification failures"
        ))
        .expect("eel_telemetry_hmac_failures_total registration failed")
    });

    REPLAY_REJECTIONS_TOTAL.get_or_init(|| {
        register_counter!(Opts::new(
            "eel_telemetry_replay_rejections_total",
            "Total signals rejected as replays"
        ))
        .expect("eel_telemetry_replay_rejections_total registration failed")
    });

    MERKLE_SNAPSHOTS_TOTAL.get_or_init(|| {
        register_counter!(Opts::new(
            "eel_telemetry_merkle_snapshots_total",
            "Total daily Merkle snapshots completed"
        ))
        .expect("eel_telemetry_merkle_snapshots_total registration failed")
    });
}

pub fn inc_signals_ingested(class: &str) {
    if let Some(c) = SIGNALS_INGESTED_TOTAL.get() {
        c.with_label_values(&[class]).inc();
    }
}

pub fn inc_alerts_raised(class: &str, severity: &str) {
    if let Some(c) = ALERTS_RAISED_TOTAL.get() {
        c.with_label_values(&[class, severity]).inc();
    }
}

pub fn inc_hmac_failures() {
    if let Some(c) = HMAC_FAILURES_TOTAL.get() {
        c.inc();
    }
}

pub fn inc_replay_rejections() {
    if let Some(c) = REPLAY_REJECTIONS_TOTAL.get() {
        c.inc();
    }
}

pub fn inc_merkle_snapshots() {
    if let Some(c) = MERKLE_SNAPSHOTS_TOTAL.get() {
        c.inc();
    }
}
