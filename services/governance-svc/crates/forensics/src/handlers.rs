use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;
use crate::models::*;

pub async fn health() -> impl IntoResponse { StatusCode::OK }

pub async fn list_cycles() -> impl IntoResponse {
    info!("list_cycles");
    let cycles: Vec<AnalysisCycle> = vec![];
    Json(serde_json::json!({ "cycles": cycles }))
}

pub async fn create_cycle(Json(req): Json<CreateCycleRequest>) -> impl IntoResponse {
    info!(election_id = %req.election_id, method = ?req.method, "create_cycle");
    let cycle = AnalysisCycle {
        id: uuid::Uuid::new_v4().to_string(),
        election_id: req.election_id,
        method: req.method,
        status: CycleStatus::Draft,
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    (StatusCode::CREATED, Json(cycle))
}

pub async fn trigger_run(Path(id): Path<String>) -> impl IntoResponse {
    info!(cycle_id = %id, "trigger_run");
    StatusCode::ACCEPTED
}

pub async fn list_estimates(Path(id): Path<String>) -> impl IntoResponse {
    info!(cycle_id = %id, "list_estimates");
    let estimates: Vec<EstimateRecord> = vec![];
    Json(serde_json::json!({ "estimates": estimates }))
}

pub async fn create_referral(
    Path(cycle_id): Path<String>,
    Json(req): Json<CreateReferralRequest>,
) -> impl IntoResponse {
    info!(cycle_id, referred_to = ?req.referred_to, "create_referral");
    let packet = ReferralPacket {
        id: uuid::Uuid::new_v4().to_string(),
        cycle_id,
        referred_to: req.referred_to,
        referral_basis: req.referral_basis,
        content_url: req.content_url,
        referred_at: chrono::Utc::now().to_rfc3339(),
        acknowledged_at: None,
    };
    (StatusCode::CREATED, Json(packet))
}

pub async fn publish_replication(
    Path(cycle_id): Path<String>,
    Json(req): Json<PublishReplicationRequest>,
) -> impl IntoResponse {
    info!(cycle_id, "publish_replication");
    let pkg = ReplicationPackage {
        id: uuid::Uuid::new_v4().to_string(),
        cycle_id,
        code_url: req.code_url,
        data_hash: req.data_hash,
        environment_spec: req.environment_spec,
        doi: req.doi,
        published_at: chrono::Utc::now().to_rfc3339(),
    };
    (StatusCode::CREATED, Json(pkg))
}

pub async fn list_public_reports() -> impl IntoResponse {
    info!("list_public_reports");
    let reports: Vec<PublicReport> = vec![];
    Json(serde_json::json!({ "reports": reports }))
}

pub async fn get_public_report(Path(id): Path<String>) -> impl IntoResponse {
    info!(report_id = %id, "get_public_report");
    Json(PublicReport {
        id,
        ..Default::default()
    })
}
