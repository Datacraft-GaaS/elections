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
