//! Round-trip tests for the gazette and calendar endpoints.
//!
//! The one that matters most: `submit_gazette_notice` fabricated
//! `gov_printer_ref = format!("GPR-{id}")` from the notice's own id, emitted a
//! `gazette_submitted_v1` event and returned 204. A gazette notice is a
//! *statutory publication* — that reference is the evidence it was published, and
//! it was manufactured with no printer contacted. It now refuses.
//!
//! `derive_notices_for_event` carried the comment "load event from DB, call rules
//! engine, persist notices" and did none of the three, returning an empty list
//! for every event — indistinguishable from an event that requires no notice. It
//! now calls the crate's own `rules::derive_notices`.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_portal_gazette::api::{router, AppState};

fn app() -> axum::Router {
    router()
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn send(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> axum::response::Response {
    let mut b = Request::builder().method(method).uri(uri);
    let payload = match body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    app.clone().oneshot(b.body(payload).unwrap()).await.unwrap()
}

fn event_body() -> Value {
    json!({
        "election_id": "00000000-0000-0000-0000-0000000000e1",
        "event_type": "polling_day",
        "scheduled_at": "2026-11-01T08:00:00Z",
        "gazette_ref": null,
        "statutory_citation": "Art. 168(3)"
    })
}

async fn create_event(app: &axum::Router) -> String {
    let resp = send(&app, "POST", "/calendar/events", Some(event_body())).await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "create_calendar_event must work for the rest to be reachable"
    );
    body_json(resp).await["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn submitting_a_gazette_notice_refuses_rather_than_manufacturing_a_printer_ref() {
    // The most serious fabrication fixed in this sweep: a statutory publication
    // receipt generated from the notice's own id, with no printer contacted.
    let app = app();
    let notice = body_json(
        send(
            &app,
            "POST",
            "/gazette/notices",
            Some(json!({
                "notice_type": "campaign_start",
                "body": "Notice of campaign start",
                "statutory_rule_ids": [],
                "legal_approver_id": "00000000-0000-0000-0000-0000000000a1"
            })),
        )
        .await,
    )
    .await;
    let id = notice["id"].as_str().unwrap().to_string();

    let resp = send(&app, "POST", &format!("/gazette/notices/{id}/submit"), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "submitting previously manufactured GPR-<id> and returned 204"
    );
    let problem = body_json(resp).await;
    assert!(
        problem["detail"].as_str().unwrap_or("").contains("no gazette printer"),
        "the refusal should name the missing integration"
    );
}

#[tokio::test]
async fn a_created_notice_is_listed_and_unpublished_by_default() {
    let app = app();
    send(
        &app,
        "POST",
        "/gazette/notices",
        Some(json!({
            "notice_type": "campaign_start",
            "body": "Notice of campaign start",
            "statutory_rule_ids": [],
            "legal_approver_id": "00000000-0000-0000-0000-0000000000a1"
        })),
    )
    .await;

    let all = body_json(send(&app, "GET", "/gazette/notices", None).await).await;
    assert_eq!(all["notices"].as_array().unwrap().len(), 1);

    // A notice that was never published must not appear as published.
    let unpublished = body_json(send(&app, "GET", "/gazette/notices?published=false", None).await).await;
    assert_eq!(unpublished["notices"].as_array().unwrap().len(), 1);
    let published = body_json(send(&app, "GET", "/gazette/notices?published=true", None).await).await;
    assert!(
        published["notices"].as_array().unwrap().is_empty(),
        "?published=true must filter, not return everything"
    );
}

#[tokio::test]
async fn calendar_events_round_trip() {
    let app = app();
    let id = create_event(&app).await;

    let fetched = body_json(send(&app, "GET", &format!("/calendar/events/{id}"), None).await).await;
    assert_eq!(fetched["id"], id.as_str());
    assert_eq!(fetched["statutory_citation"], "Art. 168(3)");

    let listed = body_json(send(&app, "GET", "/calendar/events", None).await).await;
    assert_eq!(listed["events"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn an_unknown_calendar_event_is_404() {
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/calendar/events/00000000-0000-0000-0000-0000000000bb",
        None,
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "get_calendar_event previously answered 404 unconditionally, including for \
         events that had just been created"
    );
}

#[tokio::test]
async fn derive_notices_uses_the_rules_engine() {
    // The comment promised all three steps and the body did none. An empty derived
    // list for a polling-day event is indistinguishable from "no notice required",
    // which is the answer that would let a statutory notice go unpublished.
    let app = app();
    let id = create_event(&app).await;

    let derived = body_json(
        send(&app, "POST", &format!("/calendar/events/{id}/derive-notices"), None).await,
    )
    .await;
    let notices = derived["derived"].as_array().unwrap();
    assert!(
        !notices.is_empty(),
        "a polling_day event must derive at least the campaign_start notice from \
         rules::derive_notices"
    );
    assert_eq!(notices[0]["notice_type"], "campaign_start");

    // Deriving persists: the notices appear in the list afterwards.
    let listed = body_json(send(&app, "GET", "/gazette/notices", None).await).await;
    assert!(
        listed["notices"].as_array().unwrap().len() >= 1,
        "derived notices must be persisted, as the handler comment promised"
    );
}

#[tokio::test]
async fn deriving_for_an_unknown_event_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/calendar/events/00000000-0000-0000-0000-0000000000cc/derive-notices",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn subscriptions_round_trip_and_delete() {
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/subscriptions",
            Some(json!({
                "subscriber_id": "00000000-0000-0000-0000-0000000000f1",
                "channel": "email",
                "event_types": ["polling_day"]
            })),
        )
        .await,
    )
    .await;
    let id = created["id"].as_str().unwrap().to_string();

    assert_eq!(
        send(&app, "DELETE", &format!("/subscriptions/{id}"), None).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        send(&app, "DELETE", &format!("/subscriptions/{id}"), None).await.status(),
        StatusCode::NOT_FOUND,
        "delete_subscription previously answered 204 for any id, deleting nothing"
    );
}

#[tokio::test]
async fn deleting_an_unknown_subscription_is_404() {
    let app = app();
    let resp = send(
        &app,
        "DELETE",
        "/subscriptions/00000000-0000-0000-0000-0000000000dd",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn routers_do_not_share_records() {
    let a = app();
    let b = app();
    create_event(&a).await;
    let b_listed = body_json(send(&b, "GET", "/calendar/events", None).await).await;
    assert!(
        b_listed["events"].as_array().unwrap().is_empty(),
        "records leaked between routers"
    );
    let _ = AppState::default();
}