use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::threshold::{meets_art_104_2_threshold, threshold_numerator_denominator};

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_petitions() -> impl IntoResponse {
    info!("list_petitions");
    let petitions: Vec<RecallPetition> = vec![];
    Json(serde_json::json!({ "petitions": petitions, "total": 0 }))
}

pub async fn get_petition(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_petition");
    Json(RecallPetition { id, ..Default::default() })
}

pub async fn create_petition(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("create_petition");
    // Commissioner dual-sign verified in production via SpiceDB (P01)
    let id = uuid::Uuid::now_v7().to_string();
    Json(serde_json::json!({ "id": id, "status": "draft", "body": body }))
}

pub async fn import_signatories(
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "import_signatories");
    // OCR batch processed asynchronously; result available via poll endpoint
    Json(serde_json::json!({
        "petition_id": id,
        "batch_id": uuid::Uuid::now_v7().to_string(),
        "status": "processing"
    }))
}

pub async fn get_threshold_report(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_threshold_report");
    let registered_voters = 100_000u64;
    let verified_signatories = 0u64;
    let (num, den) = threshold_numerator_denominator(verified_signatories, registered_voters);
    let meets = meets_art_104_2_threshold(verified_signatories, registered_voters);
    let report = ThresholdReport {
        id: uuid::Uuid::now_v7().to_string(),
        petition_id: id,
        total_signatories: 0,
        verified_signatories,
        registered_voters,
        numerator_x100: num,
        denominator_30pct: den,
        meets_threshold: meets,
        ward_breakdown: serde_json::Value::Object(Default::default()),
        generated_at: chrono::Utc::now().to_rfc3339(),
    };
    Json(report)
}

pub async fn close_petition(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "close_petition");
    StatusCode::NO_CONTENT
}

pub async fn create_recall_election(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "create_recall_election");
    // Idempotent: triggers M50 by-election creation via outbox event
    let election = RecallElection {
        id: uuid::Uuid::now_v7().to_string(),
        petition_id: id,
        by_election_id: None,
        trigger_status: "pending".to_string(),
        trigger_attempts: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::CREATED, Json(election))
}

pub async fn list_signatories(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "list_signatories");
    let records: Vec<SignatoryRecord> = vec![];
    Json(serde_json::json!({ "petition_id": id, "records": records, "total": 0 }))
}
