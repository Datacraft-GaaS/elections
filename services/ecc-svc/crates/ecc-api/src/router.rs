use crate::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        // ECC
        .route("/ecc/situational-awareness", get(get_situational_awareness))
        .route("/ecc/stations", get(list_stations))
        .route("/ecc/stations/{station_code}", get(get_station))
        .route(
            "/ecc/escalations",
            get(list_escalations).post(create_escalation),
        )
        .route(
            "/ecc/escalations/{escalation_id}",
            get(get_escalation).patch(patch_escalation),
        )
        .route("/ecc/turnout-heatmap", get(get_turnout_heatmap))
        .route("/ecc/audit-log", get(get_audit_log))
        // IAB
        .route(
            "/iab/audit-reports",
            get(list_audit_reports).post(create_audit_report),
        )
        .route("/iab/audit-reports/{report_id}", get(get_audit_report))
        .route("/iab/dashboards/summary", get(get_iab_summary))
        .route("/iab/audit-trails/{module_code}", get(get_audit_trails))
        .route("/iab/flags", get(list_iab_flags).post(create_iab_flag))
        // Grievance
        .route(
            "/grievance/complaints",
            get(list_complaints).post(create_complaint),
        )
        .route("/grievance/complaints/{reference_code}", get(get_complaint))
        .route(
            "/grievance/complaints/{complaint_id}/updates",
            get(list_complaint_updates).post(add_complaint_update),
        )
        .route("/grievance/ussd/session", post(ussd_session))
        .route("/grievance/whatsapp/webhook", post(whatsapp_webhook))
        // SSE portals
        .route("/ecc/events/party", get(crate::sse::party_stream))
        .route("/ecc/events/observer", get(crate::sse::observer_stream))
        .route("/ecc/events/journalist", get(crate::sse::journalist_stream))
}
