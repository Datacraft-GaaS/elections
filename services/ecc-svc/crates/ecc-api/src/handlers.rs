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
