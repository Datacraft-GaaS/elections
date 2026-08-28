use axum::{extract::Path, http::StatusCode, response::{IntoResponse, Json}};
use tracing::info;
use uuid::Uuid;
use crate::models::*;

// Health

pub async fn health() -> impl IntoResponse { StatusCode::OK }

// ECC handlers

pub async fn get_situational_awareness() -> impl IntoResponse {
    info!("get_situational_awareness");
    Json(SituationalAwareness {
        total_stations: 0,
        reporting_stations: 0,
        pending_escalations: 0,
        active_alerts: 0,
        turnout_pct: 0.0,
        as_of: "2027-08-09T06:00:00Z".to_string(),
    })
}

pub async fn list_stations() -> impl IntoResponse {
    Json(serde_json::json!({ "stations": [] }))
}

pub async fn get_station(Path(station_code): Path<String>) -> impl IntoResponse {
    info!(station_code, "get_station");
    Json(Station {
        station_code,
        name: "stub".to_string(),
        constituency_code: "stub".to_string(),
        county_code: "stub".to_string(),
        status: "unknown".to_string(),
        last_updated: None,
    })
}

pub async fn list_escalations() -> impl IntoResponse {
    Json(serde_json::json!({ "escalations": [] }))
}

pub async fn create_escalation(Json(req): Json<EscalationRequest>) -> impl IntoResponse {
    info!(station_code=%req.station_code, "create_escalation");
    (StatusCode::CREATED, Json(EscalationCreated {
        escalation_id: Uuid::new_v4(),
        status: "open".to_string(),
    }))
}

pub async fn get_escalation(Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(Escalation {
        escalation_id: id,
        station_code: "stub".to_string(),
        category: "stub".to_string(),
        severity: "low".to_string(),
        status: "open".to_string(),
        description: "stub".to_string(),
        raised_at: "2027-08-09T06:00:00Z".to_string(),
        resolved_at: None,
    })
}

pub async fn patch_escalation(Path(id): Path<Uuid>, Json(req): Json<EscalationPatch>) -> impl IntoResponse {
    info!(escalation_id=%id, status=?req.status, "patch_escalation");
    Json(serde_json::json!({ "escalation_id": id, "status": req.status.unwrap_or_else(|| "open".to_string()) }))
}

pub async fn get_turnout_heatmap() -> impl IntoResponse {
    Json(serde_json::json!({ "entries": [] }))
}

pub async fn get_audit_log() -> impl IntoResponse {
    Json(serde_json::json!({ "entries": [] }))
}

// IAB handlers

pub async fn list_audit_reports() -> impl IntoResponse {
    Json(serde_json::json!({ "reports": [] }))
}

pub async fn create_audit_report(Json(req): Json<AuditReportRequest>) -> impl IntoResponse {
    info!(module_code=%req.module_code, "create_audit_report");
    (StatusCode::CREATED, Json(AuditReportCreated {
        report_id: Uuid::new_v4(),
        status: "submitted".to_string(),
    }))
}

pub async fn get_audit_report(Path(report_id): Path<Uuid>) -> impl IntoResponse {
    Json(AuditReport {
        report_id,
        module_code: "stub".to_string(),
        title: "stub".to_string(),
        status: "pending".to_string(),
        submitted_at: "2027-08-09T06:00:00Z".to_string(),
    })
}

pub async fn get_iab_summary() -> impl IntoResponse {
    Json(IabSummary {
        open_reports: 0,
        flagged_items: 0,
        pending_trails: 0,
        as_of: "2027-08-09T06:00:00Z".to_string(),
    })
}

pub async fn get_audit_trails(Path(module_code): Path<String>) -> impl IntoResponse {
    info!(module_code, "get_audit_trails");
    Json(serde_json::json!({ "module_code": module_code, "entries": [] }))
}

pub async fn list_iab_flags() -> impl IntoResponse {
    Json(serde_json::json!({ "flags": [] }))
}

pub async fn create_iab_flag(Json(req): Json<IabFlagRequest>) -> impl IntoResponse {
    info!(module_code=%req.module_code, "create_iab_flag");
    (StatusCode::CREATED, Json(IabFlagCreated {
        flag_id: Uuid::new_v4(),
        status: "open".to_string(),
    }))
}

// Grievance handlers

pub async fn list_complaints() -> impl IntoResponse {
    Json(serde_json::json!({ "complaints": [] }))
}

pub async fn create_complaint(Json(req): Json<GrievanceComplaintRequest>) -> impl IntoResponse {
    info!(complainant=%req.complainant_name, "create_complaint");
    let ref_code = format!("GRV-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
    (StatusCode::CREATED, Json(GrievanceComplaintCreated {
        complaint_id: Uuid::new_v4(),
        reference_code: ref_code,
        status: "received".to_string(),
    }))
}

pub async fn get_complaint(Path(reference_code): Path<String>) -> impl IntoResponse {
    info!(reference_code, "get_complaint");
    Json(GrievanceComplaint {
        complaint_id: Uuid::new_v4(),
        reference_code,
        complainant_name: "stub".to_string(),
        category: "stub".to_string(),
        status: "received".to_string(),
        submitted_at: "2027-08-09T06:00:00Z".to_string(),
    })
}

pub async fn list_complaint_updates(Path(complaint_id): Path<Uuid>) -> impl IntoResponse {
    Json(serde_json::json!({ "complaint_id": complaint_id, "updates": [] }))
}

pub async fn add_complaint_update(Path(complaint_id): Path<Uuid>, Json(req): Json<ComplaintUpdateRequest>) -> impl IntoResponse {
    info!(complaint_id=%complaint_id, "add_complaint_update");
    (StatusCode::CREATED, Json(ComplaintUpdate {
        update_id: Uuid::new_v4(),
        complaint_id,
        note: req.note,
        updated_by: req.updated_by,
        updated_at: "2027-08-09T06:00:00Z".to_string(),
    }))
}

pub async fn ussd_session(Json(req): Json<UssdSessionRequest>) -> impl IntoResponse {
    info!(session_id=%req.session_id, msisdn=%req.msisdn, "ussd_session");
    Json(UssdSessionResponse {
        session_id: req.session_id,
        response: "Welcome to IEBC Grievance. Press 1 to lodge a complaint.".to_string(),
        end_session: false,
    })
}

pub async fn whatsapp_webhook(Json(_payload): Json<WhatsappWebhookPayload>) -> impl IntoResponse {
    StatusCode::OK
}

// Section 44A handlers

use axum::extract::State;
use axum::response::Response;
use chrono::Utc;
use crate::store::{S44aRow, Store, StoreError};

fn store_error_response(err: StoreError) -> Response {
    match err {
        StoreError::S44aConfirmationExists => (
            StatusCode::CONFLICT,
            "s44a confirmation already exists for station",
        )
            .into_response(),
        StoreError::S44aNotAwaitingOfficer2 => (
            StatusCode::CONFLICT,
            "no s44a confirmation awaiting officer2 for station",
        )
            .into_response(),
        other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()).into_response(),
    }
}

/// POST /ecc/s44a/officer1 -- record officer 1 confirmation for a station.
///
/// Returns 409 if the station already has a confirmation row.
pub async fn create_s44a_officer1(
    State(store): State<Store>,
    Json(req): Json<S44aOfficer1Request>,
) -> Response {
    info!(station_code=%req.station_code, officer_id=%req.officer_id, "s44a officer1 confirm");
    let row = S44aRow {
        confirmation_id: Uuid::new_v4(),
        county_code: req.county_code.clone(),
        station_code: req.station_code.clone(),
        officer1_id: req.officer_id,
        officer1_confirmed_at: Utc::now(),
        officer2_id: None,
        officer2_confirmed_at: None,
        status: "officer1_confirmed".to_string(),
    };
    let confirmation_id = row.confirmation_id;
    match store.insert_s44a_officer1(&row).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(S44aConfirmation {
                confirmation_id,
                station_code: req.station_code,
                status: "officer1_confirmed".to_string(),
            }),
        )
            .into_response(),
        Err(e) => store_error_response(e),
    }
}

/// POST /ecc/s44a/officer2 -- complete dual-officer verification.
///
/// Returns 409 if no confirmation is in the `officer1_confirmed` state.
pub async fn complete_s44a_officer2(
    State(store): State<Store>,
    Json(req): Json<S44aOfficer2Request>,
) -> Response {
    info!(station_code=%req.station_code, officer_id=%req.officer_id, "s44a officer2 confirm");
    match store
        .complete_s44a_officer2(&req.station_code, &req.officer_id, Utc::now())
        .await
    {
        Ok(Some(row)) => Json(S44aConfirmation {
            confirmation_id: row.confirmation_id,
            station_code: row.station_code,
            status: row.status,
        })
        .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            format!("no s44a confirmation for station {}", req.station_code),
        )
            .into_response(),
        Err(e) => store_error_response(e),
    }
}
