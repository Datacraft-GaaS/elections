use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
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
    /// Gazette notices by id.
    pub notices: eel_memstore_rs::Collection<crate::domain::GazetteNotice>,
    /// Calendar events by id.
    pub events: eel_memstore_rs::Collection<crate::domain::CalendarEvent>,
    /// Subscriptions by id.
    pub subscriptions: eel_memstore_rs::Collection<crate::domain::Subscription>,
}

/// A problem+json response for a missing resource.
fn not_found(detail: String) -> Response {
    problem(StatusCode::NOT_FOUND, "Not Found".to_string(), detail)
}

/// A problem+json response for an operation this crate cannot honestly perform.
fn not_implemented(detail: String) -> Response {
    problem(StatusCode::NOT_IMPLEMENTED, "Not Implemented".to_string(), detail)
}

fn problem(status: StatusCode, title: String, detail: String) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": title,
            "status": status.as_u16(),
            "detail": detail,
        })),
    )
        .into_response()
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
    State(state): State<AppState>,
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
    tracing::info!(id = %event.id, "calendar event created");
    // Real. Previously answered 201 and dropped the event.
    state.events.insert(event.id.to_string(), event.clone());
    (StatusCode::CREATED, Json(event))
}

async fn list_calendar_events(
    State(state): State<AppState>,
    Query(q): Query<CalendarEventQuery>,
) -> impl IntoResponse {
    // Real, with the declared filters applied. Previously always an empty array.
    let events: Vec<CalendarEvent> = state
        .events
        .list()
        .into_iter()
        .filter(|e| q.election_id.is_none_or(|id| e.election_id == id))
        .filter(|e| q.from.is_none_or(|from| e.scheduled_at >= from))
        .filter(|e| q.to.is_none_or(|to| e.scheduled_at <= to))
        .collect();
    (StatusCode::OK, Json(EventsResp { events }))
}

async fn get_calendar_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    tracing::debug!(%id, "get calendar event");
    // Real. Previously answered 404 unconditionally.
    match state.events.get(&id.to_string()) {
        Some(e) => Ok(Json(e).into_response()),
        None => Err(not_found(format!("no calendar event with id {id}"))),
    }
}

async fn derive_notices_for_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    tracing::debug!(%id, "derive notices for calendar event");
    // Real, and it now calls the crate's own rules engine. The comment said
    // "load event from DB, call rules engine, persist notices" and the body did
    // none of the three: it returned an empty derived list for every event,
    // which is indistinguishable from an event that requires no notice.
    let event = state
        .events
        .get(&id.to_string())
        .ok_or_else(|| not_found(format!("no calendar event with id {id}")))?;
    let derived = crate::rules::derive_notices(&event);
    for notice in &derived {
        state
            .notices
            .insert(notice.id.to_string(), notice.clone());
    }
    tracing::info!(%id, derived = derived.len(), "derived gazette notices");
    Ok(Json(DerivedNoticesResp { derived }).into_response())
}

async fn create_gazette_notice(
    State(state): State<AppState>,
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
    // Real. Previously answered 201 and dropped the notice. `hsm_signature`,
    // `published_at` and `gov_printer_ref` all stay None: a notice that has not
    // been signed or published must not look otherwise.
    state.notices.insert(notice.id.to_string(), notice.clone());
    (StatusCode::CREATED, Json(notice))
}

async fn list_gazette_notices(
    State(state): State<AppState>,
    Query(q): Query<GazetteNoticeQuery>,
) -> impl IntoResponse {
    // Real. Previously always an empty list, so `?published=true` returned an
    // empty array rather than an error or a filter that did nothing visible.
    let notices = state
        .notices
        .list()
        .into_iter()
        .filter(|n| match q.published {
            Some(true) => n.published_at.is_some(),
            Some(false) => n.published_at.is_none(),
            None => true,
        })
        .collect();
    (StatusCode::OK, Json(NoticesResp { notices }))
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
) -> Response {
    // Refused, and this is the most serious refusal in the tree.
    //
    // This handler fabricated `gov_printer_ref = format!("GPR-{id}")`, emitted a
    // gazette_submitted_v1 event and returned 204. A gazette notice is a
    // *statutory publication*: the reference is the evidence that a notice was
    // published, and it was manufactured from the notice's own id with no printer
    // contacted. A caller holding `GPR-…` had a receipt for a publication that
    // never happened, and the emitted event said it did.
    not_implemented(format!(
        "no gazette printer integration exists in this crate, so no statutory \
         publication can be made; this previously manufactured gov_printer_ref \
         as GPR-{id} from the notice's own id and returned 204, which is a \
         receipt for a publication that never occurred"
    ))
}

async fn create_subscription(
    State(state): State<AppState>,
    Json(req): Json<CreateSubscriptionReq>,
) -> impl IntoResponse {
    let sub = Subscription {
        id: Uuid::now_v7(),
        subscriber_id: req.subscriber_id,
        channel: req.channel,
        event_types: req.event_types,
    };
    // Real. Previously answered 201 and dropped the subscription, so
    // delete_subscription could only ever answer 204 for nothing.
    state.subscriptions.insert(sub.id.to_string(), sub.clone());
    (StatusCode::CREATED, Json(sub))
}

async fn delete_subscription(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    tracing::info!(%id, "subscription deleted");
    // Real. Previously answered 204 for any id, deleting nothing.
    match state.subscriptions.remove(&id.to_string()) {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(not_found(format!("no subscription with id {id}"))),
    }
}
