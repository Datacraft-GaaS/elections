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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::info;

use crate::models::*;
use crate::state::SharedState;
use crate::threshold::{
    can_declare, county_has_yes_majority, meets_county_threshold, meets_national_majority,
    REQUIRED_COUNTY_THRESHOLD,
};


/// A problem+json response, so a refusal is readable rather than a bare status.
fn problem(status: StatusCode, detail: String) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail,
        })),
    )
        .into_response()
}


pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn list_referendums(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_referendums");
    // Real. Previously always an empty list with total 0, which asserted that no
    // referendum exists rather than that nothing was stored.
    let mut referendums: Vec<Referendum> = state
        .referendums
        .list();
    referendums.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    let total = referendums.len();
    Json(serde_json::json!({ "referendums": referendums, "total": total }))
}

pub async fn get_referendum(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(id, "get_referendum");
    // Real. Previously echoed the requested id into a default-constructed
    // Referendum and answered 200 for any id, so a lookup for a referendum that
    // does not exist returned a plausible referendum instead of a 404.
    match state.referendums.get(&id) {
        Some(r) => Ok(Json(r).into_response()),
        None => Err(problem(StatusCode::NOT_FOUND, format!("no referendum with id {id}"))),
    }
}

pub async fn create_referendum(
    State(state): State<SharedState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("create_referendum");
    let id = uuid::Uuid::now_v7().to_string();
    let referendum = Referendum {
        id: id.clone(),
        election_id: body["election_id"].as_str().unwrap_or("").to_string(),
        question_text: body["question_text"].clone(),
        status: ReferendumStatus::Draft,
        // Always 24 for constitutional referenda, stored explicitly for audit.
        county_threshold: REQUIRED_COUNTY_THRESHOLD,
        national_majority_rule: "yes > (yes + no) / 2".to_string(),
        scheduled_date: body["scheduled_date"].as_str().unwrap_or("").to_string(),
        gazette_notice_id: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let evt = crate::events::ReferendumConfiguredV1::new(
        id.clone(),
        referendum.election_id.clone(),
        referendum.scheduled_date.clone(),
    );
    // Real. Previously minted an id, synthesised this event and answered 201
    // while storing nothing and emitting nothing, so every creation succeeded
    // and none of them existed.
    state.referendums.insert(id.clone(), referendum.clone());
    (StatusCode::CREATED, Json(serde_json::json!({
        "referendum": referendum,
        "event": evt,
    })))
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
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, Response> {
    info!(id, "submit_county_result");
    if state.referendums.get(&id).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no referendum with id {id}"),
        ));
    }
    let yes: u64 = body["yes_votes"].as_u64().unwrap_or(0);
    let no: u64 = body["no_votes"].as_u64().unwrap_or(0);
    let result = ReferendumCountyResult {
        id: uuid::Uuid::now_v7().to_string(),
        referendum_id: id.clone(),
        county_code: body["county_code"].as_str().unwrap_or("000").to_string(),
        yes_votes: yes,
        no_votes: no,
        rejected_votes: body["rejected_votes"].as_u64().unwrap_or(0),
        registered_voters: body["registered_voters"].as_u64().unwrap_or(0),
        status: "provisional".to_string(),
        certified_at: None,
        county_yes_majority: county_has_yes_majority(yes, no),
    };
    // Real. Previously answered 201 with a provisional result it never recorded.
    // This was the bottom of the chain that certify_county_result and
    // get_national_result claimed to read from.
    state
        .county_results
        .lock()
        .expect("referendum county results lock poisoned")
        .entry(id)
        .or_default()
        .push(result.clone());
    Ok((StatusCode::CREATED, Json(result)).into_response())
}

pub async fn certify_county_result(
    State(state): State<SharedState>,
    Path((id, county)): Path<(String, String)>,
) -> Result<Response, Response> {
    info!(id, county, "certify_county_result");
    let certified_at = chrono::Utc::now().to_rfc3339();
    // Real. Previously logged and returned 204, which is a successful
    // certification of a result that existed nowhere.
    let updated = {
        let mut guard = state
            .county_results
            .lock()
            .expect("referendum county results lock poisoned");
        match guard.get_mut(&id) {
            Some(results) => match results.iter_mut().find(|r| r.county_code == county) {
                Some(r) => {
                    r.status = "certified".to_string();
                    r.certified_at = Some(certified_at.clone());
                    Some(r.clone())
                }
                None => None,
            },
            None => None,
        }
    };
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no county result for referendum {id} county {county}"),
        )),
    }
}

pub async fn get_national_result(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(id, "get_national_result");
    if state.referendums.get(&id).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no referendum with id {id}"),
        ));
    }
    // Real. Previously synthesised this aggregate from constants --
    // counties_reported 0, counties_yes_majority 0, both thresholds evaluated at
    // zero -- and minted a fresh id. A national tally computed from no county
    // results is not a tally of zero, it is a tally never taken.
    let results = state
        .county_results
        .lock()
        .expect("referendum county results lock poisoned")
        .get(&id)
        .cloned()
        .unwrap_or_default();

    let total_yes: u64 = results.iter().map(|r| r.yes_votes).sum();
    let total_no: u64 = results.iter().map(|r| r.no_votes).sum();
    let total_rejected: u64 = results.iter().map(|r| r.rejected_votes).sum();
    let counties_yes_majority = results.iter().filter(|r| r.county_yes_majority).count() as u8;

    Ok((
        StatusCode::OK,
        Json(ReferendumNationalResult {
            id: uuid::Uuid::now_v7().to_string(),
            referendum_id: id,
            total_yes,
            total_no,
            total_rejected,
            counties_reported: results.len() as u8,
            counties_yes_majority,
            meets_county_threshold: meets_county_threshold(counties_yes_majority),
            meets_national_majority: meets_national_majority(total_yes, total_no),
            computed_at: chrono::Utc::now().to_rfc3339(),
        }),
    )
        .into_response())
}

pub async fn declare_referendum(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(id, "declare_referendum");
    if state.referendums.get(&id).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no referendum with id {id}"),
        ));
    }
    let results = state
        .county_results
        .lock()
        .expect("referendum county results lock poisoned")
        .get(&id)
        .cloned()
        .unwrap_or_default();

    // The invariant the old comment claimed to enforce without ever calling it:
    // NoDeclarationWithoutBothThresholds. It is now actually evaluated, using
    // the crate's unit-tested can_declare.
    let counties_yes_majority = results.iter().filter(|r| r.county_yes_majority).count() as u8;
    let total_yes: u64 = results.iter().map(|r| r.yes_votes).sum();
    let total_no: u64 = results.iter().map(|r| r.no_votes).sum();
    if !can_declare(counties_yes_majority, total_yes, total_no) {
        return Err(problem(
            StatusCode::CONFLICT,
            format!(
                "cannot declare: {counties_yes_majority} counties with YES majority \
                 (need {REQUIRED_COUNTY_THRESHOLD}), totals yes={total_yes} \
                 no={total_no}"
            ),
        ));
    }

    let updated = state.referendums.update(&id, |mut r| {
        r.status = ReferendumStatus::Declared;
        r
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no referendum with id {id}"),
        )),
    }
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
