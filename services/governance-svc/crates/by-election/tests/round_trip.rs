//! Round-trip tests for by-election: real kit coordinates, real cost figures,
//! and an activation state machine.
//!
//! `create_kit_plan` used to assign every polling station
//! `station_gps_lat: 0.0, station_gps_lng: 0.0` — 0N 0E, in the Gulf of Guinea —
//! so a 201 described a delivery plan for kits that were never reserved, routed
//! to open ocean. `get_cost_report` bypassed the crate's own `build_cost_report`
//! arithmetic and returned a default report with every line zero.
//! `activate_by_election` answered 204 for any id without checking the GAZETTED
//! transition its own comment named as a prerequisite.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_by_election::{router::build_router, state::AppState};

fn app() -> axum::Router {
    build_router(AppState::new())
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

#[tokio::test]
async fn kit_plan_carries_the_supplied_coordinates_not_zeros() {
    let app = app();
    let id = seed_and_get_id(&app).await;

    let resp = send(
        &app,
        "POST",
        &format!("/v1/by-elections/{id}/kit-plan"),
        Some(json!({
            "stations": [
                {"code": "STN-001", "lat": -1.2921, "lng": 36.8219},
                {"code": "STN-002", "lat": -0.0917, "lng": 37.9060}
            ],
            "expected_delivery_date": "2026-11-01T06:00:00Z"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let plan = body_json(resp).await;

    let assignments = plan["station_assignments"].as_array().unwrap();
    assert_eq!(assignments.len(), 2);
    assert_eq!(assignments[0]["station_code"], "STN-001");
    // The bug: these were both 0.0, which is the Gulf of Guinea.
    assert!(
        (assignments[0]["station_gps_lat"].as_f64().unwrap() - (-1.2921)).abs() < 1e-6,
        "latitude must be the real one, got {}",
        assignments[0]["station_gps_lat"]
    );
    assert!((assignments[1]["station_gps_lng"].as_f64().unwrap() - 37.9060).abs() < 1e-6);
    assert!(
        assignments.iter().all(|a| a["acknowledged"] == false),
        "a new assignment is unacknowledged"
    );
    assert_eq!(plan["kit_ids"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn kit_plan_hash_is_present_and_is_not_a_signature() {
    let app = app();
    let id = seed_and_get_id(&app).await;
    let plan = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/by-elections/{id}/kit-plan"),
            Some(json!({"stations": [{"code": "STN-001", "lat": -1.29, "lng": 36.82}]})),
        )
        .await,
    )
    .await;
    let hash = plan["signed_blob_hash"].as_str().expect("a hash is recorded");
    assert!(hash.starts_with("sha256:"), "got {hash}");
    assert!(!plan["signed_at"].is_null());
}

#[tokio::test]
async fn a_kit_plan_needs_at_least_one_station() {
    // Otherwise the plan is an empty delivery to nowhere, which is what the
    // zeroed version effectively was.
    let app = app();
    let id = seed_and_get_id(&app).await;
    let resp = send(
        &app,
        "POST",
        &format!("/v1/by-elections/{id}/kit-plan"),
        Some(json!({"stations": []})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn operations_on_an_unknown_by_election_are_404() {
    let app = app();
    for (method, uri, body) in [
        ("POST", "/v1/by-elections/nope/kit-plan", Some(json!({"stations": [{"code":"S","lat":1.0,"lng":2.0}]}))),
        ("POST", "/v1/by-elections/nope/register-subset", Some(json!({"station_count": 1, "voter_count": 10}))),
        ("PUT", "/v1/by-elections/nope/activate", None),
        ("GET", "/v1/by-elections/nope/cost-report", None),
    ] {
        let resp = send(&app, method, uri, body).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{method} {uri}");
    }
}

#[tokio::test]
async fn register_subset_requires_the_m02_counts_rather_than_inventing_them() {
    // generate_subset used to return station_count 0 and voter_count 0 and the
    // handler answered 201 for a subset covering no voters.
    let app = app();
    let id = seed_and_get_id(&app).await;

    let missing = send(
        &app,
        "POST",
        &format!("/v1/by-elections/{id}/register-subset"),
        Some(json!({"snapshot_id": "SNAP-1", "constituency_id": "CONST-047"})),
    )
    .await;
    assert_eq!(
        missing.status(),
        StatusCode::UNPROCESSABLE_ENTITY,
        "counts must be supplied, not assumed to be zero"
    );

    let ok = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/by-elections/{id}/register-subset"),
            Some(json!({
                "snapshot_id": "SNAP-1",
                "constituency_id": "CONST-047",
                "station_count": 3,
                "voter_count": 145000
            })),
        )
        .await,
    )
    .await;
    assert_eq!(ok["station_count"], 3);
    assert_eq!(ok["voter_count"], 145000);
    assert_eq!(ok["snapshot_id"], "SNAP-1");
    assert!(ok["frozen_at"].is_null(), "a fresh subset is not frozen");
}

#[tokio::test]
async fn cost_report_reads_back_the_submitted_figures() {
    // Previously get_cost_report returned a default report with every line zero,
    // bypassing build_cost_report entirely.
    let app = app();
    let id = seed_and_get_id(&app).await;

    let absent = send(&app, "GET", &format!("/v1/by-elections/{id}/cost-report"), None).await;
    assert_eq!(
        absent.status(),
        StatusCode::NOT_FOUND,
        "no submitted report is not an all-zero report"
    );

    let submitted = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/by-elections/{id}/cost-report"),
            Some(json!({
                "kit_days_kes": 1000,
                "staff_days_kes": 2000,
                "transport_kes": 3000,
                "printing_kes": 4000,
                "venue_kes": 5000,
                "other_kes": 6000
            })),
        )
        .await,
    )
    .await;
    assert_eq!(
        submitted["total_kes"], 21000,
        "build_cost_report's arithmetic should be visible in the response"
    );

    let fetched = body_json(send(&app, "GET", &format!("/v1/by-elections/{id}/cost-report"), None).await).await;
    assert_eq!(fetched["total_kes"], 21000, "the submitted total must read back");
    assert_eq!(fetched["kit_days_kes"], 1000);
    assert_eq!(fetched["other_kes"], 6000);
}

#[tokio::test]
async fn activation_requires_the_gazetted_state() {
    // Previously 204 for any id, with no transition checked.
    let app = app();
    let (draft, gazetted) = two_elections(&app).await;

    let too_early = send(&app, "PUT", &format!("/v1/by-elections/{draft}/activate"), None).await;
    assert_eq!(
        too_early.status(),
        StatusCode::CONFLICT,
        "activation must require GAZETTED, which its own comment named"
    );

    let ok = send(&app, "PUT", &format!("/v1/by-elections/{gazetted}/activate"), None).await;
    assert_eq!(ok.status(), StatusCode::NO_CONTENT);
}

/// Create two by-elections through the public API, returning (draft, gazetted).
async fn two_elections(app: &axum::Router) -> (String, String) {
    let mut ids = Vec::new();
    for (notice, date) in [(None, "2026-11-01"), (Some("GZ-2026-01"), "2026-11-01")] {
        let mut body = json!({
            "constituency_id": "CONST-047",
            "seat_type": "Mp",
            "trigger": "Vacancy",
            "notice_date": "2026-08-01",
            "election_date": date,
            "cost_ceiling_kes": 5000000
        });
        if let Some(n) = notice {
            body["gazette_notice_id"] = json!(n);
        }
        let resp = send(app, "POST", "/v1/by-elections", Some(body)).await;
        if resp.status() == StatusCode::CREATED {
            let created = body_json(resp).await;
            ids.push(created["by_election"]["id"].as_str().unwrap_or_default().to_string());
        }
    }
    assert_eq!(ids.len(), 2, "both by-elections should be created");
    (ids[0].clone(), ids[1].clone())
}

/// Create one by-election and return its id.
async fn seed_and_get_id(app: &axum::Router) -> String {
    let resp = send(
        app,
        "POST",
        "/v1/by-elections",
        Some(json!({
            "constituency_id": "CONST-047",
            "seat_type": "Mp",
            "trigger": "Vacancy",
            "notice_date": "2026-08-01",
            "election_date": "2026-11-01",
            "cost_ceiling_kes": 5000000,
            "gazette_notice_id": "GZ-2026-01"
        })),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "create_by_election must work for the other handlers to be reachable"
    );
    body_json(resp).await["by_election"]["id"].as_str().unwrap().to_string()
}