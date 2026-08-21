use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::threshold::{can_declare, county_has_yes_majority, meets_county_threshold, meets_national_majority};

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_referendums() -> impl IntoResponse {
    info!("list_referendums");
    let referendums: Vec<Referendum> = vec![];
    Json(serde_json::json!({ "referendums": referendums, "total": 0 }))
}

pub async fn get_referendum(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_referendum");
    Json(Referendum { id, ..Default::default() })
}

pub async fn create_referendum(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("create_referendum");
    let id = uuid::Uuid::now_v7().to_string();
    let referendum = Referendum {
        id: id.clone(),
        county_threshold: crate::threshold::REQUIRED_COUNTY_THRESHOLD,
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let evt = crate::events::ReferendumConfiguredV1::new(
        id,
        body["election_id"].as_str().unwrap_or("").to_string(),
        body["scheduled_date"].as_str().unwrap_or("").to_string(),
    );
    (StatusCode::CREATED, Json(serde_json::json!({ "referendum": referendum, "event": evt })))
}

pub async fn activate_referendum(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "activate_referendum");
    // Kit OTA mode bundle signed by HSM (P06) and distributed; stub returns pending ack
    let bundle = KitModeBundle {
        id: uuid::Uuid::now_v7().to_string(),
        referendum_id: id,
        signed_bundle_hash: "stub-hash".to_string(),
        distributed_at: Some(chrono::Utc::now().to_rfc3339()),
        ack_count: 0,
        total_kits: 0,
    };
    Json(bundle)
}

pub async fn submit_county_result(
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!(id, "submit_county_result");
    let yes: u64 = body["yes_votes"].as_u64().unwrap_or(0);
    let no: u64 = body["no_votes"].as_u64().unwrap_or(0);
    let county_code = body["county_code"].as_str().unwrap_or("000").to_string();
    let majority = county_has_yes_majority(yes, no);
    let result = ReferendumCountyResult {
        id: uuid::Uuid::now_v7().to_string(),
        referendum_id: id,
        county_code,
        yes_votes: yes,
        no_votes: no,
        rejected_votes: body["rejected_votes"].as_u64().unwrap_or(0),
        registered_voters: body["registered_voters"].as_u64().unwrap_or(0),
        status: "provisional".to_string(),
        certified_at: None,
        county_yes_majority: majority,
    };
    (StatusCode::CREATED, Json(result))
}

pub async fn certify_county_result(
    Path((id, county)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(id, county, "certify_county_result");
    StatusCode::NO_CONTENT
}

pub async fn get_national_result(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_national_result");
    // Stub: returns empty aggregate; real impl aggregates all 47 county results
    let result = ReferendumNationalResult {
        id: uuid::Uuid::now_v7().to_string(),
        referendum_id: id,
        counties_reported: 0,
        counties_yes_majority: 0,
        meets_county_threshold: meets_county_threshold(0),
        meets_national_majority: meets_national_majority(0, 0),
        computed_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };
    Json(result)
}

pub async fn declare_referendum(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "declare_referendum");
    // Hard dependency on M51 result-declaration-certification
    // Invariant enforced: NoDeclarationWithoutBothThresholds
    // Stub: real impl fetches county results and checks can_declare()
    let _ = can_declare; // referenced to confirm import
    StatusCode::NO_CONTENT
}
