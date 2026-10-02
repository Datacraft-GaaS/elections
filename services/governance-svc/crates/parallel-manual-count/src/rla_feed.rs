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

use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::divergence::DivergenceReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlaAuditInput {
    pub input_id: String,
    pub source: String,
    pub report_id: String,
    pub generated_at: String,
    pub mean_abs_divergence: f64,
    pub stations_with_divergence: usize,
    pub total_stations: usize,
    pub raw_report_json: String,
}

pub fn build_rla_input(report: &DivergenceReport) -> RlaAuditInput {
    let raw_report_json =
        serde_json::to_string(report).unwrap_or_else(|_| "{}".to_string());

    RlaAuditInput {
        input_id: Uuid::new_v4().to_string(),
        source: "parallel-manual-count".to_string(),
        report_id: report.report_id.clone(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        mean_abs_divergence: report.mean_abs_divergence,
        stations_with_divergence: report.stations_with_divergence,
        total_stations: report.total_stations_sampled,
        raw_report_json,
    }
}

pub async fn submit_to_rla(
    input: &RlaAuditInput,
    rla_endpoint: &str,
) -> Result<String, anyhow::Error> {
    let url = format!("{}/v1/audit-inputs", rla_endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(input)
        .send()
        .await
        .with_context(|| format!("POST to RLA endpoint {}", url))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "<empty>".to_string());

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "RLA endpoint returned {}: {}",
            status,
            body
        ));
    }

    Ok(body)
}
