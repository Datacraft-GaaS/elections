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

//! POST /api/v1/bvi/anomalies/evaluate.
//!
//! Stateless anomaly-detection endpoint: runs every bvi-anomaly detector
//! over the submitted payload. No store access, so the handler stays
//! compatible with the generic `BviStore` handler bounds.

use axum::{
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use bvi_anomaly::{
    burst_zscore, digit_distribution_chi_square, velocity_impossible, DeviceRead, TurnoutAnomaly,
};
use serde::Deserialize;

use crate::{error::ApiError, idem};

const INSTANCE: &str = "/api/v1/bvi/anomalies/evaluate";

/// Default minimum travel time between stations for one device (10 minutes).
const DEFAULT_MIN_TRAVEL_SECONDS: i64 = 600;

#[derive(Debug, Deserialize)]
pub struct StationTotals {
    pub station_code: String,
    pub identified: i64,
    pub registered: i64,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateRequest {
    #[serde(default)]
    pub reads: Vec<DeviceRead>,
    #[serde(default)]
    pub station_totals: Vec<StationTotals>,
    #[serde(default)]
    pub series: Vec<f64>,
    pub window_index: Option<usize>,
    pub window_len: Option<usize>,
    pub min_travel_seconds: Option<i64>,
    /// Optional observed leading-digit counts for the chi-square detector.
    pub digit_counts: Option<Vec<u64>>,
    /// Optional expected shares aligned with `digit_counts`.
    pub expected_shares: Option<Vec<f64>>,
}

#[derive(Debug, serde::Serialize)]
pub struct EvaluateResponse {
    pub anomalies: Vec<TurnoutAnomaly>,
    pub digit_stats: Option<bvi_anomaly::DigitStats>,
}

/// POST /api/v1/bvi/anomalies/evaluate.
///
/// Runs all detectors (turnover-over-registered per station, impossible
/// device velocity, turnout-burst z-score) plus the digit-distribution
/// chi-square when the payload carries leading-digit counts under the
/// optional `digit_counts` / `expected_shares` fields.
pub async fn evaluate(
    headers: HeaderMap,
    Json(req): Json<EvaluateRequest>,
) -> Result<Response, ApiError> {
    idem::require_bearer(&headers, INSTANCE)?;

    let min_travel_seconds = req.min_travel_seconds.unwrap_or(DEFAULT_MIN_TRAVEL_SECONDS);

    let mut anomalies: Vec<TurnoutAnomaly> = Vec::new();

    for st in &req.station_totals {
        if let Some(a) = bvi_anomaly::turnover_over_registered(
            st.station_code.clone(),
            st.identified,
            st.registered,
        ) {
            anomalies.push(a);
        }
    }

    anomalies.extend(velocity_impossible(&req.reads, min_travel_seconds));

    if let (Some(window_index), Some(window_len), Some(series)) = (
        req.window_index,
        req.window_len,
        (!req.series.is_empty()).then_some(&req.series),
    ) {
        if let Some(a) = burst_zscore(series, window_index, window_len, 3.0) {
            anomalies.push(a);
        }
    }

    let digit_stats = match (&req.digit_counts, &req.expected_shares) {
        (Some(counts), Some(shares)) => match digit_distribution_chi_square(counts, shares) {
            Ok(stats) => Some(stats),
            Err(e) => return Err(ApiError::bad_request(INSTANCE, &e)),
        },
        _ => None,
    };

    Ok((
        axum::http::StatusCode::OK,
        Json(EvaluateResponse {
            anomalies,
            digit_stats,
        }),
    )
        .into_response())
}
