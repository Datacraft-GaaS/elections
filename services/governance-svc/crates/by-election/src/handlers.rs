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

use crate::cost_report::build_cost_report;
use crate::kit_plan::sign_kit_plan;
use crate::models::*;
use crate::spec;
use crate::state::SharedState;

/// Parse an enum variant by name, falling back only when the field is absent.
///
/// An unrecognised value falls back to `default` rather than erroring, which
/// keeps this handler total; the response echoes what was stored so a caller
/// can see which variant was chosen.
fn parse_enum<T>(value: &serde_json::Value, default: T) -> T
where
    T: serde::de::DeserializeOwned + Default,
{
    value
        .as_str()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_string())).ok())
        .unwrap_or(default)
}

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

/// A polling station to build a kit assignment from.
///
/// Coordinates are a required input rather than a defaulted one. `generate_kit_plan`
/// used to assign every station `station_gps_lat: 0.0, station_gps_lng: 0.0`,
/// which is 0N 0E in the Gulf of Guinea, so a 201 described a delivery plan
/// routed to open ocean. There is no station gazetteer in this workspace to look
/// a code up in, so the caller supplies the coordinates.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StationInput {
    pub code: String,
    pub lat: f64,
    pub lng: f64,
}

/// The body `create_kit_plan` accepts.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct KitPlanRequest {
    pub stations: Vec<StationInput>,
    #[serde(default)]
    pub expected_delivery_date: Option<String>,
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn create_by_election(
    State(state): State<SharedState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    info!("create_by_election");
    let id = uuid::Uuid::now_v7().to_string();
    let gazette_notice_id = body["gazette_notice_id"].as_str().map(str::to_string);
    let by_election = ByElection {
        id: id.clone(),
        constituency_id: body["constituency_id"].as_str().unwrap_or("").to_string(),
        // These are enums, so the strings must name real variants rather than
        // being defaulted into a seat type nobody asked for.
        seat_type: parse_enum(&body["seat_type"], SeatType::Mp),
        trigger: parse_enum(&body["trigger"], ByElectionTrigger::Vacancy),
        // A by-election with a gazette notice starts GAZETTED; one without is a
        // draft. Previously status was left at Default::default() and nothing
        // was stored, so no by-election could ever be activated because none
        // existed.
        status: if gazette_notice_id.is_some() {
            ByElectionStatus::Gazetted
        } else {
            ByElectionStatus::Draft
        },
        notice_date: body["notice_date"]
            .as_str()
            .unwrap_or("2025-01-01")
            .to_string(),
        election_date: body["election_date"]
            .as_str()
            .unwrap_or("2025-02-01")
            .to_string(),
        cost_ceiling_kes: body["cost_ceiling_kes"].as_u64().unwrap_or(0),
        gazette_notice_id,
        recall_petition_id: body["recall_petition_id"].as_str().map(str::to_string),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    let short_notice = by_election.is_short_notice();
    state.by_elections.insert(id.clone(), by_election.clone());
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "by_election": by_election,
            "short_notice_alert": short_notice,
        })),
    )
}

pub async fn generate_register_subset(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response, Response> {
    info!(by_election_id = %id, "generate_register_subset");
    if state.by_elections.get(&id).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no by-election with id {id}"),
        ));
    }

    // The counts are the M02 snapshot extraction's output. There is no M02
    // adapter in this workspace, so they arrive as a required input instead of
    // being invented: generate_subset used to return station_count 0 and
    // voter_count 0 and this answered 201 for a subset covering no voters.
    let station_count = match body["station_count"].as_u64() {
        Some(n) => n,
        None => {
            return Err(problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "station_count is required: it is the M02 extraction result and \
                 cannot be assumed"
                    .to_string(),
            ))
        }
    };
    let voter_count = match body["voter_count"].as_u64() {
        Some(n) => n,
        None => {
            return Err(problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "voter_count is required: it is the M02 extraction result and \
                 cannot be assumed"
                    .to_string(),
            ))
        }
    };

    let subset = RegisterSubset {
        id: uuid::Uuid::now_v7().to_string(),
        by_election_id: id.clone(),
        snapshot_id: body["snapshot_id"].as_str().unwrap_or("").to_string(),
        constituency_id: body["constituency_id"].as_str().unwrap_or("").to_string(),
        station_count: station_count as u32,
        voter_count,
        signed_blob_hash: None,
        generated_at: chrono::Utc::now().to_rfc3339(),
        frozen_at: None,
    };
    state
        .register_subsets
        .insert(subset.id.clone(), subset.clone());
    Ok((StatusCode::CREATED, Json(subset)).into_response())
}

pub async fn create_kit_plan(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<KitPlanRequest>,
) -> Result<Response, Response> {
    info!(by_election_id = %id, stations = req.stations.len(), "create_kit_plan");
    if state.by_elections.get(&id).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no by-election with id {id}"),
        ));
    }
    if req.stations.is_empty() {
        return Err(problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "a kit plan needs at least one station with real coordinates".to_string(),
        ));
    }

    let delivery = req
        .expected_delivery_date
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let assignments: Vec<KitAssignment> = req
        .stations
        .iter()
        .map(|s| KitAssignment {
            kit_id: uuid::Uuid::new_v4().to_string(),
            station_code: s.code.clone(),
            station_gps_lat: s.lat,
            station_gps_lng: s.lng,
            expected_delivery_date: delivery.clone(),
            acknowledged: false,
        })
        .collect();

    let plan = sign_kit_plan(KitPlan {
        id: uuid::Uuid::now_v7().to_string(),
        by_election_id: id.clone(),
        kit_ids: assignments.iter().map(|a| a.kit_id.clone()).collect(),
        station_assignments: assignments,
        signed_blob_hash: None,
        signed_at: None,
    });

    // The hash is a plain SHA-256 digest over the canonical plan, stored in a
    // field named signed_blob_hash. It is not a signature: there is no key, so
    // it attests only that the plan is self-consistent, not who produced it.
    state.kit_plans.insert(plan.id.clone(), plan.clone());
    Ok((StatusCode::CREATED, Json(plan)).into_response())
}

pub async fn activate_by_election(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(id, "activate_by_election");
    // Real. Previously answered 204 for any id without checking the GAZETTED
    // transition its own comment named as a prerequisite.
    let outcome = match state.by_elections.get(&id) {
        Some(e) if e.status == ByElectionStatus::Gazetted => {
            state.by_elections.update(&id, |mut e| {
                // Activation opens nominations. ByElectionStatus has no `Active`
                // variant and adding one would misrepresent the state machine.
                e.status = ByElectionStatus::NominationsOpen;
                e
            });
            "ok"
        }
        Some(_) => "not_gazetted",
        None => "missing",
    };
    match outcome {
        "ok" => Ok(StatusCode::NO_CONTENT.into_response()),
        "not_gazetted" => Err(problem(
            StatusCode::CONFLICT,
            format!("by-election {id} is not GAZETTED; activation requires it"),
        )),
        _ => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no by-election with id {id}"),
        )),
    }
}

pub async fn submit_cost_report(
    State(state): State<SharedState>,
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
        Ok(report) => {
            // Store it so get_cost_report reads the submitted figures rather
            // than a default-constructed report full of zeros.
            let by_election_id = report.by_election_id.clone();
            state.cost_reports.insert(report.id.clone(), report.clone());
            info!(
                by_election_id,
                total_kes = report.total_kes,
                "cost report stored"
            );
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(report).unwrap()),
            )
        }
        Err(msg) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": msg })),
        ),
    }
}

pub async fn get_cost_report(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Response, Response> {
    info!(by_election_id = %id, "get_cost_report");
    // Real. Previously returned a default-constructed CostReport with every line
    // zero, bypassing the crate's own build_cost_report arithmetic entirely.
    let report = state
        .cost_reports
        .list_where(|r| r.by_election_id == id)
        .into_iter()
        .max_by_key(|r| r.submitted_at.clone().unwrap_or_default());
    match report {
        Some(r) => Ok(Json(r).into_response()),
        // A by-election with no submitted cost report has no report, which is
        // different from one whose report is all zeros.
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no cost report submitted for by-election {id}"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Public contract endpoints (governance-by-election/operations.yaml)
// ---------------------------------------------------------------------------

/// POST /api/v1/governance/by-election/elections — declare a by-election.
pub async fn declare_by_election(
    State(state): State<SharedState>,
    Json(req): Json<spec::ByElectionDeclaration>,
) -> impl IntoResponse {
    info!(constituency = %req.constituency_code, "declare_by_election");
    let now = chrono::Utc::now().to_rfc3339();
    let election = spec::ByElection {
        by_election_id: uuid::Uuid::now_v7().to_string(),
        trigger_reason: req.trigger_reason,
        constituency_code: req.constituency_code,
        gazette_ref: req.gazette_ref,
        election_date: req.election_date,
        status: "declared".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    let mut guard = state.elections.lock().expect("lock poisoned");
    guard.insert(election.by_election_id.clone(), election.clone());
    drop(guard);
    (StatusCode::CREATED, Json(election)).into_response()
}

/// GET /api/v1/governance/by-election/elections — list by-elections.
pub async fn list_by_elections(State(state): State<SharedState>) -> impl IntoResponse {
    info!("list_by_elections");
    let guard = state.elections.lock().expect("lock poisoned");
    let mut items: Vec<spec::ByElection> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Json(
        serde_json::json!({ "items": items, "meta": { "page": 1, "page_size": items.len(), "total": items.len(), "total_pages": 1 } }),
    )
}

/// GET /api/v1/governance/by-election/elections/{by_election_id} — fetch one.
pub async fn get_by_election(
    State(state): State<SharedState>,
    Path(by_election_id): Path<String>,
) -> impl IntoResponse {
    info!(by_election_id, "get_by_election");
    let guard = state.elections.lock().expect("lock poisoned");
    match guard.get(&by_election_id) {
        Some(e) => Json(e.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "by-election not found"})),
        )
            .into_response(),
    }
}

/// PATCH /api/v1/governance/by-election/elections/{by_election_id}/status —
/// transition the by-election status.
pub async fn transition_by_election_status(
    State(state): State<SharedState>,
    Path(by_election_id): Path<String>,
    Json(req): Json<spec::StatusTransition>,
) -> impl IntoResponse {
    info!(by_election_id, new_status = %req.new_status, "transition_by_election_status");
    let mut guard = state.elections.lock().expect("lock poisoned");
    match guard.get_mut(&by_election_id) {
        Some(e) => {
            e.status = req.new_status;
            e.updated_at = chrono::Utc::now().to_rfc3339();
            Json(e.clone()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"detail": "by-election not found"})),
        )
            .into_response(),
    }
}
