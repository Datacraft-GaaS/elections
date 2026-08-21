use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::cost_report::build_cost_report;
use crate::kit_plan::{generate_kit_plan, sign_kit_plan};
use crate::models::*;
use crate::register_subset::generate_subset;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_by_elections() -> impl IntoResponse {
    info!("list_by_elections");
    let elections: Vec<ByElection> = vec![];
    Json(serde_json::json!({ "by_elections": elections, "total": 0 }))
}

pub async fn get_by_election(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_by_election");
    Json(ByElection { id, ..Default::default() })
}

pub async fn create_by_election(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("create_by_election");
    // Commissioner dual-sign verified in production via SpiceDB (P01) -- FR-M50-002
    // Concurrency guard: reject if 10 active by-elections exist (FR-M50-008) -- stub
    let id = uuid::Uuid::now_v7().to_string();
    let notice_date = body["notice_date"].as_str().unwrap_or("2025-01-01").to_string();
    let election_date = body["election_date"].as_str().unwrap_or("2025-02-01").to_string();
    let by_election = ByElection {
        id: id.clone(),
        constituency_id: body["constituency_id"].as_str().unwrap_or("").to_string(),
        notice_date: notice_date.clone(),
        election_date: election_date.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let short_notice = by_election.is_short_notice();
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "by_election": by_election,
            "short_notice_alert": short_notice,
        })),
    )
}

pub async fn generate_register_subset(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "generate_register_subset");
    let snapshot_id = body["snapshot_id"].as_str().unwrap_or("").to_string();
    let constituency_id = body["constituency_id"].as_str().unwrap_or("").to_string();
    let subset = generate_subset(id, constituency_id, snapshot_id);
    (StatusCode::CREATED, Json(subset))
}

pub async fn create_kit_plan(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "create_kit_plan");
    let station_codes: Vec<String> = body["station_codes"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let plan = sign_kit_plan(generate_kit_plan(id, station_codes));
    (StatusCode::CREATED, Json(plan))
}

pub async fn activate_by_election(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "activate_by_election");
    // Triggers M04 register subset notice; GAZETTED transition required first
    StatusCode::NO_CONTENT
}

pub async fn submit_cost_report(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "submit_cost_report");
    let parse_u64 = |key: &str| body[key].as_u64().unwrap_or(0);
    match build_cost_report(
        id,
        parse_u64("kit_days_kes"),
        parse_u64("staff_days_kes"),
        parse_u64("transport_kes"),
        parse_u64("printing_kes"),
        parse_u64("venue_kes"),
        parse_u64("other_kes"),
    ) {
        Ok(report) => (StatusCode::CREATED, Json(serde_json::to_value(report).unwrap())),
        Err(msg) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": msg })),
        ),
    }
}

pub async fn get_cost_report(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_cost_report");
    Json(CostReport {
        id: uuid::Uuid::now_v7().to_string(),
        by_election_id: id,
        ..Default::default()
    })
}
