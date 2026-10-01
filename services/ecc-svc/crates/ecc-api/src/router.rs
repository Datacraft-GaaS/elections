use crate::handlers::*;
use crate::speech;
use crate::store::Store;
use axum::{
    routing::{get, post},
    Extension, Router,
};

pub fn build_router(store: Store) -> Router {
    // Constructed per call so each router -- including every test's -- gets its
    // own records. A shared store would let one test's complaint appear in
    // another's list and make the round-trip pass for the wrong reason.
    let records = crate::records::Records::default();
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
        // Section 44A dual-officer verification
        .route("/ecc/s44a/officer1", post(create_s44a_officer1))
        .route("/ecc/s44a/officer2", post(complete_s44a_officer2))
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
        // Speech integrity (M17) public contract
        .route("/speech/media-items", get(speech::list_media_items).post(speech::submit_media_item))
        .route("/speech/media-items/{item_id}", get(speech::get_media_item))
        .route("/speech/media-items/{item_id}/adjudications", post(speech::adjudicate_media_item))
        .route("/speech/reports/summary", get(speech::get_speech_summary_report))
        .with_state(store)
        .layer(Extension(records))
}
