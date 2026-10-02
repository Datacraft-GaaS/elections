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

//! Round-trip tests for the `/v1/observer-missions` surface.
//!
//! Previously these endpoints held no state at all: every list returned `vec![]`,
//! `register_mission` hardcoded `workspace_id: "ws-stub"` for every mission and
//! discarded the mission, `submit_report` accepted a report from any mission id
//! including one that had never been registered, and `publish_joint_statement`
//! accepted participants that did not exist.
//!
//! `ObserverEngine::can_submit_report` encoded the rule that only an **active**
//! mission may file an observation report. The handler never called it.
//!
//! Each test fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use eel_governance_intl_observer::{router::build_router, state::AppState};
use std::sync::Arc;

fn app() -> axum::Router {
    build_router(Arc::new(AppState::default()))
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn send(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> axum::response::Response {
    let mut b = Request::builder().method(method).uri(uri);
    let payload = match body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    app.clone()
        .oneshot(b.body(payload).unwrap())
        .await
        .unwrap()
}

/// Register an accredited (therefore active) mission.
async fn active_mission(app: &axum::Router, org: &str) -> String {
    let resp = send(
        app,
        "POST",
        "/v1/observer-missions",
        Some(json!({
            "organisation_name": org,
            "accreditation_ref": format!("acc-{org}"),
            "lead_contact_hash": "sha256:abc"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let m = body_json(resp).await;
    assert_eq!(m["status"], "ACTIVE", "an accredited mission is active");
    m["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_registered_mission_is_listed_and_stored() {
    let app = app();
    let id = active_mission(&app, "African Union").await;

    let listed = body_json(send(&app, "GET", "/v1/observer-missions", None).await).await;
    let missions = listed["missions"].as_array().unwrap();
    assert_eq!(missions.len(), 1, "a registered mission must be listed");
    assert_eq!(missions[0]["id"], id.as_str());
    assert_eq!(missions[0]["organisation_name"], "African Union");
    assert_eq!(missions[0]["accreditation_ref"], "acc-African Union");
}

#[tokio::test]
async fn each_mission_gets_its_own_workspace_id() {
    // Every mission previously carried the literal workspace_id "ws-stub", so two
    // missions shared one workspace reference.
    let app = app();
    let a = active_mission(&app, "AU").await;
    let b = active_mission(&app, "COMESA").await;
    assert_ne!(a, b);

    let listed = body_json(send(&app, "GET", "/v1/observer-missions", None).await).await;
    let ids: Vec<&str> = listed["missions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["workspace_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids.len(), 2);
    assert_ne!(ids[0], ids[1], "workspace ids must differ between missions");
    assert!(ids.iter().all(|w| w.starts_with("ws-")), "got {ids:?}");
    assert!(!ids.contains(&"ws-stub"), "the stub workspace id is gone");
}

#[tokio::test]
async fn an_unaccredited_mission_is_registered_not_active() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/observer-missions",
        Some(json!({
            "organisation_name": "Unaccredited NGO",
            "accreditation_ref": "",
            "lead_contact_hash": "sha256:abc"
        })),
    )
    .await;
    let m = body_json(resp).await;
    assert_eq!(m["status"], "REGISTERED");
}

#[tokio::test]
async fn registration_requires_an_organisation() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/observer-missions",
        Some(json!({
            "organisation_name": "  ",
            "accreditation_ref": "acc-1",
            "lead_contact_hash": "sha256:abc"
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_unregistered_mission_is_404() {
    // Previously any mission id was accepted, so a briefing or report could be
    // attached to a mission that did not exist.
    let app = app();
    for (method, path) in [
        ("GET", "/v1/observer-missions/nope/briefings"),
        ("GET", "/v1/observer-missions/nope/reports"),
    ] {
        assert_eq!(
            send(&app, method, path, None).await.status(),
            StatusCode::NOT_FOUND,
            "{method} {path}"
        );
    }
    let scheduled = send(
        &app,
        "POST",
        "/v1/observer-missions/nope/briefings",
        Some(json!({"topic": "T", "scheduled_at": "2026-09-01T09:00:00Z"})),
    )
    .await;
    assert_eq!(scheduled.status(), StatusCode::NOT_FOUND);
    let report = send(
        &app,
        "POST",
        "/v1/observer-missions/nope/reports",
        Some(json!({"report_type": "interim", "content_url": "https://example.invalid/r", "classified": false})),
    )
    .await;
    assert_eq!(report.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_scheduled_briefing_is_listed_for_its_mission() {
    let app = app();
    let id = active_mission(&app, "AU").await;
    let created = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/observer-missions/{id}/briefings"),
            Some(json!({
                "topic": "Tabulation centre walkthrough",
                "scheduled_at": "2026-09-01T09:00:00Z",
                "materials_url": "https://example.invalid/pack"
            })),
        )
        .await,
    )
    .await;
    assert_eq!(created["mission_id"], id.as_str());
    assert_eq!(
        created["attendance_confirmed"], false,
        "scheduling a briefing must not confirm the mission attended it"
    );

    let listed = body_json(
        send(&app, "GET", &format!("/v1/observer-missions/{id}/briefings"), None).await,
    )
    .await;
    assert_eq!(
        listed["briefings"].as_array().unwrap().len(),
        1,
        "a scheduled briefing must be listed; previously the list was always empty"
    );
}

#[tokio::test]
async fn briefings_do_not_leak_between_missions() {
    let app = app();
    let a = active_mission(&app, "AU").await;
    let b = active_mission(&app, "COMESA").await;
    send(
        &app,
        "POST",
        &format!("/v1/observer-missions/{a}/briefings"),
        Some(json!({"topic": "T", "scheduled_at": "2026-09-01T09:00:00Z"})),
    )
    .await;
    let other = body_json(
        send(&app, "GET", &format!("/v1/observer-missions/{b}/briefings"), None).await,
    )
    .await;
    assert!(other["briefings"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_briefing_requires_a_topic_and_a_time() {
    let app = app();
    let id = active_mission(&app, "AU").await;
    let resp = send(
        &app,
        "POST",
        &format!("/v1/observer-missions/{id}/briefings"),
        Some(json!({"topic": "  ", "scheduled_at": "2026-09-01T09:00:00Z"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn an_active_mission_files_an_observation_report() {
    let app = app();
    let id = active_mission(&app, "AU").await;
    let report = body_json(
        send(
            &app,
            "POST",
            &format!("/v1/observer-missions/{id}/reports"),
            Some(json!({
                "report_type": "interim",
                "content_url": "https://example.invalid/report",
                "classified": false
            })),
        )
        .await,
    )
    .await;
    assert_eq!(report["mission_id"], id.as_str());
    assert_eq!(report["report_type"], "interim");

    let listed = body_json(
        send(&app, "GET", &format!("/v1/observer-missions/{id}/reports"), None).await,
    )
    .await;
    assert_eq!(listed["reports"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn an_unaccredited_mission_cannot_file_a_report() {
    // The headline. can_submit_report requires Active; the handler never called
    // it, so a mission that had only been registered could file observations.
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/observer-missions",
        Some(json!({
            "organisation_name": "Unaccredited NGO",
            "accreditation_ref": "",
            "lead_contact_hash": "sha256:abc"
        })),
    )
    .await;
    let id = body_json(resp).await["id"].as_str().unwrap().to_string();

    let report = send(
        &app,
        "POST",
        &format!("/v1/observer-missions/{id}/reports"),
        Some(json!({"report_type": "interim", "content_url": "https://example.invalid/r", "classified": false})),
    )
    .await;
    assert_eq!(
        report.status(),
        StatusCode::CONFLICT,
        "an unaccredited mission must not file an observation report"
    );
    let detail = body_json(report).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(
        detail.contains("REGISTERED"),
        "the reason should name the status: {detail}"
    );
}

#[tokio::test]
async fn reports_do_not_leak_between_missions() {
    let app = app();
    let a = active_mission(&app, "AU").await;
    let b = active_mission(&app, "COMESA").await;
    send(
        &app,
        "POST",
        &format!("/v1/observer-missions/{a}/reports"),
        Some(json!({"report_type": "final", "content_url": "https://example.invalid/r", "classified": false})),
    )
    .await;
    let other = body_json(
        send(&app, "GET", &format!("/v1/observer-missions/{b}/reports"), None).await,
    )
    .await;
    assert!(other["reports"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_joint_statement_resolves_every_participant() {
    let app = app();
    let a = active_mission(&app, "AU").await;
    let b = active_mission(&app, "COMESA").await;
    let stmt = body_json(
        send(
            &app,
            "POST",
            "/v1/joint-statements",
            Some(json!({
                "participating_mission_ids": [a.clone(), b.clone()],
                "content_url": "https://example.invalid/statement",
                "signed_by": ["lead-a", "lead-b"]
            })),
        )
        .await,
    )
    .await;
    assert_eq!(stmt["participating_mission_ids"].as_array().unwrap().len(), 2);

    let listed = body_json(send(&app, "GET", "/v1/joint-statements", None).await).await;
    assert_eq!(
        listed["joint_statements"].as_array().unwrap().len(),
        1,
        "a published statement must be listed"
    );
}

#[tokio::test]
async fn a_joint_statement_cannot_claim_unknown_missions() {
    // The engine's check is only "not empty" -- it cannot see the store. A
    // statement attributing itself to missions that do not exist is worse than
    // no statement.
    let app = app();
    let real = active_mission(&app, "AU").await;
    let resp = send(
        &app,
        "POST",
        "/v1/joint-statements",
        Some(json!({
            "participating_mission_ids": [real, "mission-that-does-not-exist"],
            "content_url": "https://example.invalid/statement",
            "signed_by": ["lead-a"]
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let detail = body_json(resp).await["detail"].as_str().unwrap_or_default().to_string();
    assert!(
        detail.contains("mission-that-does-not-exist"),
        "the reason should name the unknown mission: {detail}"
    );

    let listed = body_json(send(&app, "GET", "/v1/joint-statements", None).await).await;
    assert!(
        listed["joint_statements"].as_array().unwrap().is_empty(),
        "a rejected statement must not be stored"
    );
}

#[tokio::test]
async fn a_joint_statement_needs_at_least_one_mission() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/v1/joint-statements",
        Some(json!({"participating_mission_ids": [], "content_url": "x", "signed_by": []})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn the_two_surfaces_do_not_share_a_store() {
    // The legacy and contract-bound mission stores are separate; registering on
    // one must not make a mission appear on the other.
    let app = app();
    active_mission(&app, "AU").await;
    let legacy = body_json(send(&app, "GET", "/v1/observer-missions", None).await).await;
    assert_eq!(legacy["missions"].as_array().unwrap().len(), 1);
    let contract = body_json(
        send(&app, "GET", "/api/v1/governance/intl-observer/missions", None).await,
    )
    .await;
    assert!(
        contract["items"].as_array().unwrap().is_empty(),
        "a legacy mission must not appear in the contract surface"
    );
}

#[tokio::test]
async fn routers_do_not_share_missions() {
    let a = app();
    let b = app();
    active_mission(&a, "AU").await;
    let listed = body_json(send(&b, "GET", "/v1/observer-missions", None).await).await;
    assert!(listed["missions"].as_array().unwrap().is_empty());
}