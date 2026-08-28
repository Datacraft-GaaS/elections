use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::state::SharedState;
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

// ---------------------------------------------------------------------------
// Public transparency endpoints (M49 spec: governance-referendum/mechanics.yaml)
// ---------------------------------------------------------------------------

/// Return the most recently registered question id, if any.
fn latest_question_id(state: &SharedState) -> Option<String> {
    let guard = state.questions.lock().expect("lock poisoned");
    guard
        .values()
        .max_by(|a, b| a.created_at.cmp(&b.created_at))
        .map(|q| q.question_id.clone())
}

/// GET /api/v1/governance/referendum/questions — list all registered questions.
pub async fn list_referendum_questions(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    info!("list_referendum_questions");
    let guard = state.questions.lock().expect("lock poisoned");
    let mut questions: Vec<ReferendumQuestion> = guard.values().cloned().collect();
    drop(guard);
    questions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(serde_json::json!({ "questions": questions }))
}

/// POST /api/v1/governance/referendum/questions — register a question (idempotent
/// on `question_id`; a duplicate `question_id` returns 409).
pub async fn register_referendum_question(
    State(state): State<SharedState>,
    Json(req): Json<ReferendumQuestionRequest>,
) -> impl IntoResponse {
    info!(question_id = %req.question_id, "register_referendum_question");
    if req.text_en.trim().is_empty() || req.text_sw.trim().is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "text_en and text_sw are required"})),
        )
            .into_response();
    }
    if req.options.len() < 2 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "at least two options are required"})),
        )
            .into_response();
    }
    let question = ReferendumQuestion {
        question_id: req.question_id.clone(),
        text_en: req.text_en,
        text_sw: req.text_sw,
        gazette_ref: req.gazette_ref,
        options: req.options,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let mut guard = state.questions.lock().expect("lock poisoned");
    if guard.contains_key(&question.question_id) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"detail": "question already registered"})),
        )
            .into_response();
    }
    guard.insert(question.question_id.clone(), question.clone());
    drop(guard);
    (StatusCode::CREATED, Json(question)).into_response()
}

/// GET /api/v1/governance/referendum/results — national + per-county results
/// for the most recently registered question.
pub async fn get_referendum_results(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    info!("get_referendum_results");
    let Some(question_id) = latest_question_id(&state) else {
        return Json(ReferendumResults {
            question_id: String::new(),
            national_yes: 0,
            national_no: 0,
            national_total: 0,
            county_results: vec![],
            snapshot_at: chrono::Utc::now().to_rfc3339(),
        })
        .into_response();
    };

    let guard = state.county_results.lock().expect("lock poisoned");
    let rows = guard.get(&question_id).cloned().unwrap_or_default();
    drop(guard);

    let mut national_yes = 0u64;
    let mut national_no = 0u64;
    let mut county_results = Vec::with_capacity(rows.len());
    for r in rows {
        national_yes = national_yes.saturating_add(r.yes_votes);
        national_no = national_no.saturating_add(r.no_votes);
        county_results.push(CountyResult {
            county_code: r.county_code.clone(),
            county_name: crate::counties::county_name(&r.county_code).to_string(),
            yes_votes: r.yes_votes,
            no_votes: r.no_votes,
            total_votes: r.yes_votes.saturating_add(r.no_votes),
            approved: r.county_yes_majority,
        });
    }
    county_results.sort_by(|a, b| a.county_code.cmp(&b.county_code));

    Json(ReferendumResults {
        question_id,
        national_yes,
        national_no,
        national_total: national_yes.saturating_add(national_no),
        county_results,
        snapshot_at: chrono::Utc::now().to_rfc3339(),
    })
    .into_response()
}

/// GET /api/v1/governance/referendum/county-threshold — 24-of-47 compliance
/// status for the most recently registered question.
pub async fn get_county_threshold_status(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    info!("get_county_threshold_status");
    let Some(question_id) = latest_question_id(&state) else {
        return Json(CountyThresholdStatus {
            counties_required: crate::threshold::REQUIRED_COUNTY_THRESHOLD,
            counties_approved: 0,
            threshold_met: false,
            county_breakdown: vec![],
            snapshot_at: chrono::Utc::now().to_rfc3339(),
        })
        .into_response();
    };

    let guard = state.county_results.lock().expect("lock poisoned");
    let rows = guard.get(&question_id).cloned().unwrap_or_default();
    drop(guard);

    let counties_approved = rows
        .iter()
        .filter(|r| r.county_yes_majority)
        .count() as u8;
    let mut breakdown: Vec<CountyBreakdown> = rows
        .iter()
        .map(|r| CountyBreakdown {
            county_code: r.county_code.clone(),
            approved: r.county_yes_majority,
        })
        .collect();
    breakdown.sort_by(|a, b| a.county_code.cmp(&b.county_code));

    Json(CountyThresholdStatus {
        counties_required: crate::threshold::REQUIRED_COUNTY_THRESHOLD,
        counties_approved,
        threshold_met: meets_county_threshold(counties_approved),
        county_breakdown: breakdown,
        snapshot_at: chrono::Utc::now().to_rfc3339(),
    })
    .into_response()
}
