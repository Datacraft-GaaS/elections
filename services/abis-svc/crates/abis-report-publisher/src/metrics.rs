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

//! Prometheus counters and histograms for the delivery pipeline.

use prometheus::{
    exponential_buckets, register_counter_vec, register_histogram_vec, CounterVec, HistogramVec,
};
use std::sync::OnceLock;

static DELIVERIES_TOTAL: OnceLock<CounterVec> = OnceLock::new();
static DELIVERY_LATENCY_MS: OnceLock<HistogramVec> = OnceLock::new();
static ABANDON_TOTAL: OnceLock<CounterVec> = OnceLock::new();

/// deliveries_by_channel_status_total{channel, status}
pub fn deliveries_total() -> &'static CounterVec {
    DELIVERIES_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "eel_report_deliveries_by_channel_status_total",
            "Total delivery attempts by channel and outcome",
            &["channel", "status"],
        )
        .expect("metric registration")
    })
}

/// delivery_latency_ms{channel}
pub fn delivery_latency_ms() -> &'static HistogramVec {
    DELIVERY_LATENCY_MS.get_or_init(|| {
        register_histogram_vec!(
            "eel_report_delivery_latency_ms",
            "Delivery round-trip latency in milliseconds",
            &["channel"],
            exponential_buckets(10.0, 2.0, 14).expect("bucket spec"),
        )
        .expect("metric registration")
    })
}

/// abandon_total{channel}
pub fn abandon_total() -> &'static CounterVec {
    ABANDON_TOTAL.get_or_init(|| {
        register_counter_vec!(
            "eel_report_delivery_abandon_total",
            "Total deliveries abandoned after max attempts",
            &["channel"],
        )
        .expect("metric registration")
    })
}
