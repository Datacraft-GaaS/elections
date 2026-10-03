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

use crate::handlers::*;
use crate::speech;
use crate::store::Store;
use axum::{
    routing::{get, post},
    Extension, Router,
};

/// The router, with Meta's app secret read from configuration.
///
/// The secret is issued by Meta and must not live in a repository, so it is
/// configuration and not code. With none set, `whatsapp_webhook` says it cannot
/// verify the payload -- which is the honest answer, and not the 200 it used to
/// return for any body at all.
pub fn build_router(store: Store) -> Router {
    let secret = std::env::var("WHATSAPP_APP_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty());
    build_router_with(store, secret)
}

/// The router, with an explicitly supplied Meta app secret.
pub fn build_router_with(store: Store, whatsapp_app_secret: Option<String>) -> Router {
    build_router_inner(store, whatsapp_app_secret)
}

/// The original route chain, plus the Meta app secret as an `Extension`.
///
/// The layer goes last: in axum a `layer` applies only to the routes added before
/// it, so putting it first would leave every route without the secret.
fn build_router_inner(store: Store, whatsapp_app_secret: Option<String>) -> Router {
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
        .route(
            "/speech/media-items",
            get(speech::list_media_items).post(speech::submit_media_item),
        )
        .route("/speech/media-items/{item_id}", get(speech::get_media_item))
        .route(
            "/speech/media-items/{item_id}/adjudications",
            post(speech::adjudicate_media_item),
        )
        .route(
            "/speech/reports/summary",
            get(speech::get_speech_summary_report),
        )
        .with_state(store)
        .layer(Extension(records))
        .layer(axum::Extension(whatsapp_app_secret))
}
