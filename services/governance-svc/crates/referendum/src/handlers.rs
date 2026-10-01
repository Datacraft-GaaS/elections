use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::info;

use crate::models::*;
use crate::state::SharedState;
use crate::threshold::{can_declare, meets_county_threshold};

/// A declared operation with no implementation behind it.
///
/// Returned instead of a fabricated success. Certification and declaration are
/// the two operations in this file that a caller cannot undo, so an endpoint
/// that answers 204 for work it never did is the most dangerous shape available
/// here: it is indistinguishable from a real certification.
fn not_implemented(operation: &str, detail: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": "Not Implemented",
            "status": 501,
            "detail": format!(
                "{operation} is declared by the contract but has no \
                 implementation behind it: {detail}"
            ),
        })),
    )
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_referendums() -> impl IntoResponse {
    info!("list_referendums");
    // Previously answered 200 with an empty list and total: 0, which reads as
    // "no referendums exist" rather than "nothing is stored".
    not_implemented(
        "referendum listing",
        "no referendum store is wired; it previously returned an empty list \
         with total 0",
    )
}

pub async fn get_referendum(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_referendum");
    // Echoed the requested id back inside a default-constructed Referendum and
    // answered 200 for any id at all, so a lookup for a referendum that does
    // not exist returned a plausible referendum rather than a 404.
    let _ = id;
    not_implemented(
        "referendum lookup",
        "no referendum store is wired; it previously echoed the requested id \
         into a default record and answered 200 for any id",
    )
}

pub async fn create_referendum(Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("create_referendum");
    // Minted a UUIDv7, built a Referendum from defaults, synthesised a
    // ReferendumConfiguredV1 event and answered 201 -- storing nothing and
    // emitting nothing. Every referendum creation therefore succeeded and none
    // of them existed.
    not_implemented(
        "referendum creation",
        "no referendum store or event bus is wired; it previously returned \
         201 with a minted id and a synthesised ReferendumConfiguredV1",
    )
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
    Path(_id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("submit_county_result");
    // Parsed the tally, computed county_has_yes_majority, built a provisional
    // result and answered 201 -- persisting nothing. County results are what
    // certify_county_result and get_national_result later claim to be working
    // from, so this was the bottom of a chain that had no storage at any link.
    not_implemented(
        "county result submission",
        "no results store is wired; it previously returned 201 with a \
         provisional result it never recorded",
    )
}

pub async fn certify_county_result(
    Path((id, county)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(id, county, "certify_county_result");
    // Previously logged and returned 204, which is a successful
    // certification: a county result was declared certified and recorded
    // nowhere. There is no store here, so the honest answer is that the
    // certification does not exist.
    not_implemented(
        "county result certification",
        "no results store is wired; it previously returned 204 for a \
         certification it never recorded",
    )
}

pub async fn get_national_result(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "get_national_result");
    // Previously synthesised an aggregate from constants: counties_reported 0,
    // counties_yes_majority 0, both thresholds evaluated at zero, and a
    // freshly minted id. A national tally computed from no county results is
    // not a tally of zero, it is a tally that was never taken, and the two
    // thresholds made the fabricated figures look authoritative.
    let _ = id;
    not_implemented(
        "national result aggregate",
        "no county results are stored; it previously reported zero counties \
         and evaluated the county and national thresholds at zero",
    )
}

pub async fn declare_referendum(Path(id): Path<String>) -> impl IntoResponse {
    info!(id, "declare_referendum");
    // Hard dependency on M51 result-declaration-certification.
    // Invariant NoDeclarationWithoutBothThresholds is *named* here and not
    // enforced: the handler returned 204 without ever calling can_declare(),
    // so the invariant the comment claims to hold was only ever a comment.
    let _ = can_declare; // referenced to confirm import
    let _ = id;
    not_implemented(
        "referendum declaration",
        "the NoDeclarationWithoutBothThresholds invariant was named in a \
         comment but never checked; this previously returned 204 for a \
         declaration that was never made",
    )
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
