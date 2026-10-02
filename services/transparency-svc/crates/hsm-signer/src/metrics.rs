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
        .expect("eel_hsm_sign_total registration failed"),
        sign_error_total: register_int_counter_vec!(
            "eel_hsm_sign_error_total",
            "Total sign errors",
            &["role"]
        )
        .expect("eel_hsm_sign_error_total registration failed"),
        ceremony_started_total: register_int_counter_vec!(
            "eel_hsm_ceremony_started_total",
            "Total ceremonies started",
            &["role"]
        )
        .expect("eel_hsm_ceremony_started_total registration failed"),
        ceremony_complete_total: register_int_counter_vec!(
            "eel_hsm_ceremony_complete_total",
            "Total ceremonies completed successfully",
            &["role"]
        )
        .expect("eel_hsm_ceremony_complete_total registration failed"),
        ceremony_aborted_total: register_int_counter_vec!(
            "eel_hsm_ceremony_aborted_total",
            "Total ceremonies aborted",
            &["role"]
        )
        .expect("eel_hsm_ceremony_aborted_total registration failed"),
        rate_limited_total: register_int_counter_vec!(
            "eel_hsm_rate_limited_total",
            "Total requests rejected by rate limiter",
            &["role"]
        )
        .expect("eel_hsm_rate_limited_total registration failed"),
        active_ceremonies: register_int_gauge!(
            "eel_hsm_active_ceremonies",
            "Currently active ceremonies"
        )
        .expect("eel_hsm_active_ceremonies registration failed"),
    })
}
