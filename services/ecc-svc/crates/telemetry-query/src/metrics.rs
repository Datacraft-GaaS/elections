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

//! Prometheus metrics for the telemetry query service.

use prometheus::{register_counter_vec, register_histogram_vec, CounterVec, HistogramVec};
use std::sync::OnceLock;

static QUERIES_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static EVIDENCE_BUNDLES_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static NL_TRANSLATIONS_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static QUERY_DURATION_SECONDS: OnceLock<HistogramVec> = OnceLock::new();

pub fn queries_total() -> &'static CounterVec {
    QUERIES_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "eel_telemetry_queries_total",
            "Total telemetry queries executed",
            &["preset", "status"]
        )
        .expect("register eel_telemetry_queries_total")
    })
}

pub fn evidence_bundles_total() -> &'static CounterVec {
    EVIDENCE_BUNDLES_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "eel_telemetry_evidence_bundles_total",
            "Total evidence bundles generated",
            &["status"]
        )
        .expect("register eel_telemetry_evidence_bundles_total")
    })
}

pub fn nl_translations_total() -> &'static CounterVec {
    NL_TRANSLATIONS_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "eel_telemetry_nl_translations_total",
            "Total NL-to-DSL translations",
            &["bridge", "status"]
        )
        .expect("register eel_telemetry_nl_translations_total")
    })
}

pub fn query_duration_seconds() -> &'static HistogramVec {
    QUERY_DURATION_SECONDS.get_or_init(|| {
        register_histogram_vec!(
            "eel_telemetry_query_duration_seconds",
            "Telemetry query execution duration",
            &["preset"],
            vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0]
        )
        .expect("register eel_telemetry_query_duration_seconds")
    })
}

/// Increment the queries_total counter.
pub fn inc_query(preset: &str, ok: bool) {
    queries_total()
        .with_label_values(&[preset, if ok { "ok" } else { "error" }])
        .inc();
}

/// Increment the evidence_bundles_total counter.
pub fn inc_evidence(ok: bool) {
    evidence_bundles_total()
        .with_label_values(&[if ok { "ok" } else { "error" }])
        .inc();
}

/// Increment the nl_translations_total counter.
pub fn inc_nl(bridge: &str, ok: bool) {
    nl_translations_total()
        .with_label_values(&[bridge, if ok { "ok" } else { "error" }])
        .inc();
}
