use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        CalendarEvent, CalendarEventType, GazetteNotice, NoticeType, Subscription,
        SubscriptionChannel,
    },
    events,
    rules::derive_notices,
};

#[derive(Clone, Default)]
pub struct AppState {
    // In a real deployment: sqlx::PgPool, outbox client, HSM key ref, gov printer URL
    pub windows: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::spec::InspectionWindow>>>,
    pub objections: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::spec::ListObjection>>>,
}

pub fn router() -> Router {
    let state = AppState::default();
    Router::new()
        .route("/calendar/events", post(create_calendar_event).get(list_calendar_events))
        .route(
            "/calendar/events/{id}",
            get(get_calendar_event),
        )
        .route(
            "/calendar/events/{id}/derive-notices",
            post(derive_notices_for_event),
        )
        .route("/gazette/notices", post(create_gazette_notice).get(list_gazette_notices))
        .route("/gazette/notices/{id}/sign", post(sign_gazette_notice))
        .route("/gazette/notices/{id}/submit", post(submit_gazette_notice))
        .route("/subscriptions", post(create_subscription))
        .route("/subscriptions/{id}", delete(delete_subscription))
        // Public contract endpoints (gazette/calendar.yaml)
        .route("/api/v1/gazette/notices/{notice_id}", get(crate::spec_handlers::get_gazette_notice))
        .route("/api/v1/gazette/inspection/windows", get(crate::spec_handlers::list_inspection_windows))
        .route("/api/v1/gazette/inspection/windows/{window_id}", get(crate::spec_handlers::get_inspection_window))
        .route("/api/v1/gazette/inspection/objections", post(crate::spec_handlers::file_list_objection))
        .route("/api/v1/gazette/inspection/objections/{reference_code}", get(crate::spec_handlers::get_objection_status))
        .with_state(state)
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct CreateCalendarEventReq {
    pub election_id: Uuid,
    pub event_type: CalendarEventType,
    pub scheduled_at: DateTime<Utc>,
    pub gazette_ref: Option<String>,
    pub statutory_citation: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CalendarEventQuery {
    pub election_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGazetteNoticeReq {
    pub notice_type: NoticeType,
    pub body: String,
    pub statutory_rule_ids: Vec<Uuid>,
    pub legal_approver_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct GazetteNoticeQuery {
    pub published: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionReq {
    pub subscriber_id: Uuid,
    pub channel: SubscriptionChannel,
    pub event_types: Vec<CalendarEventType>,
}

#[derive(Debug, Serialize)]
pub struct EventsResp {
    pub events: Vec<CalendarEvent>,
}

#[derive(Debug, Serialize)]
pub struct NoticesResp {
    pub notices: Vec<GazetteNotice>,
}

#[derive(Debug, Serialize)]
pub struct DerivedNoticesResp {
    pub derived: Vec<GazetteNotice>,
}

// --- Handlers ---

async fn create_calendar_event(
    State(_state): State<AppState>,
    Json(req): Json<CreateCalendarEventReq>,
) -> impl IntoResponse {
    let event = CalendarEvent {
        id: Uuid::now_v7(),
        election_id: req.election_id,
        event_type: req.event_type,
        scheduled_at: req.scheduled_at,
        gazette_ref: req.gazette_ref,
        statutory_citation: req.statutory_citation,
    };
    let _ev = events::calendar_event_scheduled_v1(&event.election_id, &event.id);
    tracing::info!(id = %event.id, "calendar event scheduled");
    (StatusCode::CREATED, Json(event))
}

async fn list_calendar_events(
    State(_state): State<AppState>,
    Query(_q): Query<CalendarEventQuery>,
) -> impl IntoResponse {
    // Real impl: query DB with filters
    (StatusCode::OK, Json(EventsResp { events: vec![] }))
}

async fn get_calendar_event(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::debug!(%id, "get calendar event");
    StatusCode::NOT_FOUND
}

async fn derive_notices_for_event(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::debug!(%id, "derive notices for calendar event");
    // Real impl: load event from DB, call rules engine, persist notices
    (StatusCode::OK, Json(DerivedNoticesResp { derived: vec![] }))
}

async fn create_gazette_notice(
    State(_state): State<AppState>,
    Json(req): Json<CreateGazetteNoticeReq>,
) -> impl IntoResponse {
    let notice = GazetteNotice {
        id: Uuid::now_v7(),
        notice_type: req.notice_type,
        body: req.body,
        statutory_rule_ids: req.statutory_rule_ids,
        legal_approver_id: req.legal_approver_id,
        hsm_signature: None,
        published_at: None,
        gov_printer_ref: None,
    };
    tracing::info!(id = %notice.id, "gazette notice created");
    (StatusCode::CREATED, Json(notice))
}

async fn list_gazette_notices(
    State(_state): State<AppState>,
    Query(_q): Query<GazetteNoticeQuery>,
) -> impl IntoResponse {
    (StatusCode::OK, Json(NoticesResp { notices: vec![] }))
}

async fn sign_gazette_notice(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // HSM sign stub: in production, calls eel-crypto-rs pkcs11_pool
    let hsm_key_ref = "hsm://eel.local/gazette-signing-key";
    let _ev = events::notice_signed_v1(&id, hsm_key_ref);
    tracing::info!(%id, "gazette notice signed (stub)");
    StatusCode::NO_CONTENT
}

async fn submit_gazette_notice(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // Gov Printer double: idempotent via gov_printer_ref uniqueness in DB
    let gov_printer_ref = format!("GPR-{}", id);
    let _ev = events::gazette_submitted_v1(&id, &gov_printer_ref);
    tracing::info!(%id, %gov_printer_ref, "gazette submitted to gov printer");
    StatusCode::NO_CONTENT
}

async fn create_subscription(
    State(_state): State<AppState>,
    Json(req): Json<CreateSubscriptionReq>,
) -> impl IntoResponse {
    let sub = Subscription {
        id: Uuid::now_v7(),
        subscriber_id: req.subscriber_id,
        channel: req.channel,
        event_types: req.event_types,
    };
    (StatusCode::CREATED, Json(sub))
}

async fn delete_subscription(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!(%id, "subscription deleted");
    StatusCode::NO_CONTENT
}
