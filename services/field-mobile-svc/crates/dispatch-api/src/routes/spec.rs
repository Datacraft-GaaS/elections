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

//! Public contract handlers (field-mobile-dispatch/dispatch.yaml).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::collections::HashMap;

use crate::spec;
use crate::state::AppState;

fn page(items: &serde_json::Value) -> serde_json::Value {
    eel_spec_helpers_rs::page_cursor_value(items)
}

/// POST /api/v1/mobile/dispatch/assignments — create an assignment.
pub async fn create_dispatch_assignment(
    State(state): State<AppState>,
    Json(req): Json<spec::AssignmentCreate>,
) -> impl IntoResponse {
    let now = chrono::Utc::now().to_rfc3339();
    let assignment = spec::DispatchAssignment {
        assignment_id: uuid::Uuid::now_v7().to_string(),
        personnel_id_hash: req.personnel_id_hash,
        station_code: req.station_code,
        role: req.role,
        report_by: req.report_by,
        election_day: req.election_day,
        status: "assigned".to_string(),
        actual_arrival: None,
        notes: req.notes,
        created_at: now.clone(),
        updated_at: now,
    };
    state
        .spec_assignments
        .lock()
        .expect("lock poisoned")
        .insert(assignment.assignment_id.clone(), assignment.clone());
    (StatusCode::CREATED, Json(assignment))
}

/// GET /api/v1/mobile/dispatch/assignments — list assignments.
pub async fn list_dispatch_assignments(State(state): State<AppState>) -> impl IntoResponse {
    let guard = state.spec_assignments.lock().expect("lock poisoned");
    let mut items: Vec<spec::DispatchAssignment> = guard.values().cloned().collect();
    drop(guard);
    items.sort_by(|a, b| a.assignment_id.cmp(&b.assignment_id));
    Json(page(&serde_json::to_value(items).unwrap()))
}

/// GET /api/v1/mobile/dispatch/assignments/{assignment_id} — fetch one.
pub async fn get_dispatch_assignment(
    State(state): State<AppState>,
    Path(assignment_id): Path<String>,
) -> impl IntoResponse {
    let guard = state.spec_assignments.lock().expect("lock poisoned");
    match guard.get(&assignment_id) {
        Some(a) => Json(a.clone()).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "assignment not found" })),
        )
            .into_response(),
    }
}

/// PATCH /api/v1/mobile/dispatch/assignments/{assignment_id} — update.
pub async fn update_dispatch_assignment(
    State(state): State<AppState>,
    Path(assignment_id): Path<String>,
    Json(req): Json<spec::AssignmentPatch>,
) -> impl IntoResponse {
    let mut guard = state.spec_assignments.lock().expect("lock poisoned");
    match guard.get_mut(&assignment_id) {
        Some(a) => {
            if let Some(v) = req.station_code {
                a.station_code = v;
            }
            if let Some(v) = req.personnel_id_hash {
                a.personnel_id_hash = v;
            }
            if let Some(v) = req.report_by {
                a.report_by = v;
            }
            if let Some(v) = req.status {
                a.status = v;
            }
            if let Some(v) = req.notes {
                a.notes = v;
            }
            a.updated_at = chrono::Utc::now().to_rfc3339();
            Json(a.clone()).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "detail": "assignment not found" })),
        )
            .into_response(),
    }
}

/// POST /api/v1/mobile/dispatch/bulk-assignments — bulk assignment.
pub async fn create_bulk_assignments(
    State(_state): State<AppState>,
    Json(req): Json<spec::BulkAssignRequest>,
) -> impl IntoResponse {
    let _ = req;
    (
        StatusCode::ACCEPTED,
        Json(spec::BulkAssignJob {
            job_id: uuid::Uuid::now_v7().to_string(),
            status: "processing".to_string(),
            dry_run: req.dry_run,
            submitted_at: chrono::Utc::now().to_rfc3339(),
            created_count: 0,
            error_count: 0,
            error_report_ref: None,
        }),
    )
}

/// GET /api/v1/mobile/dispatch/coverage/{constituency_code} — coverage.
pub async fn get_constituency_coverage(
    State(state): State<AppState>,
    Path(constituency_code): Path<String>,
) -> impl IntoResponse {
    let guard = state.spec_assignments.lock().expect("lock poisoned");
    let stations: HashMap<String, u32> = {
        let mut m = HashMap::new();
        for a in guard.values() {
            if a.station_code.starts_with(&constituency_code) {
                *m.entry(a.station_code.clone()).or_insert(0) += 1;
            }
        }
        m
    };
    drop(guard);
    let mut station_rows: Vec<spec::StationCoverage> = stations
        .into_iter()
        .map(|(station_code, assigned_count)| spec::StationCoverage {
            coverage_status: if assigned_count > 0 {
                "staffed"
            } else {
                "unstaffed"
            }
            .to_string(),
            station_code,
            required_count: 0,
            assigned_count,
        })
        .collect();
    station_rows.sort_by(|a, b| a.station_code.cmp(&b.station_code));
    let total = station_rows.len() as u32;
    let staffed = station_rows.iter().filter(|s| s.assigned_count > 0).count() as u32;
    Json(spec::ConstituencyCoverage {
        constituency_code,
        election_day: "".to_string(),
        total_stations: total,
        fully_staffed_count: staffed,
        understaffed_count: 0,
        unstaffed_count: total - staffed,
        coverage_pct: if total > 0 {
            staffed as f64 * 100.0 / total as f64
        } else {
            0.0
        },
        stations: station_rows,
    })
}
