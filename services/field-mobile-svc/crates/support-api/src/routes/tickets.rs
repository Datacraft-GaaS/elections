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
    Json,
};
use chrono::{Duration, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::models::{CreateTicketRequest, SupportTicket, TicketSeverity, TicketStatus};
use crate::state::AppState;

pub async fn list_tickets(state: State<AppState>) -> (StatusCode, Json<Value>) {
    let Ok(store) = state.tickets.lock() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "store unavailable" })),
        );
    };
    (
        StatusCode::OK,
        Json(serde_json::to_value(&*store).unwrap_or_default()),
    )
}

pub async fn create_ticket(
    state: State<AppState>,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    let req: CreateTicketRequest = match serde_json::from_value(body) {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid ticket request" })),
            )
        }
    };

    // SLA target scales with severity: Sev1 within 1h, Sev2 within 4h, Sev3 by next business day.
    let sla_hours = match req.severity {
        TicketSeverity::Sev1 => 1,
        TicketSeverity::Sev2 => 4,
        TicketSeverity::Sev3 => 24,
    };
    let ticket = SupportTicket {
        id: Uuid::new_v4(),
        station_code: req.station_code,
        kit_serial: req.kit_serial,
        reporter_role: String::from("FIELD_OFFICER"),
        severity: req.severity,
        title: req.title,
        description: req.description,
        status: TicketStatus::Open,
        assigned_engineer_id: None,
        sla_target_at: Utc::now() + Duration::hours(sla_hours),
        resolved_at: None,
        escalation_level: 1,
    };

    let Ok(mut store) = state.tickets.lock() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "store unavailable" })),
        );
    };
    store.push(ticket.clone());

    (
        StatusCode::CREATED,
        Json(serde_json::to_value(ticket).unwrap_or_default()),
    )
}

pub async fn get_ticket(state: State<AppState>, Path(id): Path<Uuid>) -> (StatusCode, Json<Value>) {
    let Ok(store) = state.tickets.lock() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "store unavailable" })),
        );
    };

    match store.iter().find(|t| t.id == id) {
        Some(ticket) => (
            StatusCode::OK,
            Json(serde_json::to_value(ticket).unwrap_or_default()),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "ticket not found" })),
        ),
    }
}
