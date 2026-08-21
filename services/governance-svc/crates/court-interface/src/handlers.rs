use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};
use tracing::info;

use crate::models::*;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_petitions() -> impl IntoResponse {
    info!("list_petitions");
    let petitions: Vec<Petition> = vec![];
    Json(serde_json::json!({ "petitions": petitions }))
}

pub async fn file_petition() -> impl IntoResponse {
    info!("file_petition");
    Json(Petition::default())
}

pub async fn get_petition(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "get_petition");
    Json(Petition { petition_id, ..Default::default() })
}

pub async fn request_evidence(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "request_evidence");
    StatusCode::ACCEPTED
}

pub async fn get_evidence_bundle(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "get_evidence_bundle");
    Json(EvidenceBundle { petition_id, ..Default::default() })
}

pub async fn register_witness(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "register_witness");
    Json(TechnicalWitness { petition_id, ..Default::default() })
}

pub async fn list_witnesses(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "list_witnesses");
    let witnesses: Vec<TechnicalWitness> = vec![];
    Json(serde_json::json!({ "witnesses": witnesses }))
}

pub async fn add_supplementary_request(Path(petition_id): Path<String>) -> impl IntoResponse {
    info!(petition_id, "add_supplementary_request");
    Json(SupplementaryRequest { petition_id, ..Default::default() })
}
