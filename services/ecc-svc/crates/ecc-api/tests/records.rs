//! Round-trip tests for the ECC station, grievance, escalation and IAB endpoints.
//!
//! Every handler here took no state at all. `create_complaint` minted a
//! reference code and dropped it, so a complaint filed through the grievance
//! portal could never be retrieved. `get_complaint` answered 200 for any
//! reference code with `complainant_name: "stub"` and a fixed 2027 date.
//! `patch_escalation` acknowledged a resolution for an escalation that had never
//! been raised. `get_audit_log` returned `{"entries": []}`, which is an audit log
//! that records nothing. `whatsapp_webhook` answered 200 OK for any payload while
//! discarding it.
//!
//! Every test here fails against that code.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

use ecc_api::{router::build_router, store::Store};

fn app() -> axum::Router {
    build_router(Store::Mem(Default::default()))
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

fn complaint_body() -> Value {
    json!({
        "complainant_name": "Wanjiku Mwangi",
        "complainant_contact": "+254700000001",
        "category": "long_queue",
        "description": "queued from 06:00 to 15:00 at STN-001",
        "station_code": "STN-001"
    })
}

async fn complaint(app: &axum::Router) -> (String, String) {
    let resp = send(app, "POST", "/grievance/complaints", Some(complaint_body())).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    (
        created["complaint_id"].as_str().unwrap().to_string(),
        created["reference_code"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn filed_complaint_can_be_retrieved_by_reference_code() {
    let app = app();
    let (id, ref_code) = complaint(&app).await;

    // Previously this answered 200 for any code, with "stub" as the complainant
    // and a fixed 2027 timestamp.
    let fetched = body_json(
        send(&app, "GET", &format!("/grievance/complaints/{ref_code}"), None).await,
    )
    .await;
    assert_eq!(fetched["reference_code"], ref_code.as_str());
    assert_eq!(fetched["complainant_name"], "Wanjiku Mwangi");
    assert_eq!(fetched["category"], "long_queue");
    assert_eq!(fetched["status"], "received");
    assert!(
        !fetched["submitted_at"].as_str().unwrap().starts_with("2027"),
        "the timestamp must be real, not the hardcoded 2027 date"
    );
    assert_eq!(fetched["complaint_id"], id.as_str());
}

#[tokio::test]
async fn unknown_reference_code_is_404() {
    let app = app();
    let resp = send(&app, "GET", "/grievance/complaints/GRV-NOPE", None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "get_complaint previously answered 200 for any reference code"
    );
}

#[tokio::test]
async fn filed_complaint_appears_in_the_list() {
    let app = app();
    complaint(&app).await;
    complaint(&app).await;
    let listed = body_json(send(&app, "GET", "/grievance/complaints", None).await).await;
    assert_eq!(
        listed["complaints"].as_array().unwrap().len(),
        2,
        "complaints were previously never stored, so this was always empty"
    );
}

#[tokio::test]
async fn complaint_update_is_stored_and_listed_against_its_complaint() {
    let app = app();
    let (id, _ref) = complaint(&app).await;

    let created = body_json(
        send(
            &app,
            "POST",
            &format!("/grievance/complaints/{id}/updates"),
            Some(json!({"note": "County coordinator contacted", "updated_by": "cco"})),
        )
        .await,
    )
    .await;
    assert_eq!(created["note"], "County coordinator contacted");
    assert!(
        !created["updated_at"].as_str().unwrap().starts_with("2027"),
        "updated_at was hardcoded to a 2027 date"
    );

    let listed = body_json(
        send(&app, "GET", &format!("/grievance/complaints/{id}/updates"), None).await,
    )
    .await;
    assert_eq!(listed["updates"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn update_to_an_unknown_complaint_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/grievance/complaints/00000000-0000-0000-0000-000000000001/updates",
        Some(json!({"note": "n", "updated_by": "u"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn escalation_is_created_listed_and_then_resolved() {
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/ecc/escalations",
            Some(json!({
                "station_code": "STN-014",
                "category": "material_shortage",
                "severity": "high",
                "description": "ballots did not arrive",
                "raised_by": "returning officer"
            })),
        )
        .await,
    )
    .await;
    let id = created["escalation_id"].as_str().unwrap().to_string();
    assert_eq!(created["status"], "open");

    let listed = body_json(send(&app, "GET", "/ecc/escalations", None).await).await;
    assert_eq!(listed["escalations"].as_array().unwrap().len(), 1);

    let fetched = body_json(send(&app, "GET", &format!("/ecc/escalations/{id}"), None).await).await;
    assert_eq!(fetched["station_code"], "STN-014");
    assert_eq!(fetched["severity"], "high");
    assert!(
        fetched["description"].as_str().unwrap() != "stub",
        "get_escalation previously answered with description \"stub\" for any id"
    );

    let resolved = send(
        &app,
        "PATCH",
        &format!("/ecc/escalations/{id}"),
        Some(json!({"status": "resolved", "resolution_notes": "ballots airlifted"})),
    )
    .await;
    assert_eq!(resolved.status(), StatusCode::OK);

    let after = body_json(send(&app, "GET", &format!("/ecc/escalations/{id}"), None).await).await;
    assert_eq!(after["status"], "resolved");
    assert!(
        !after["resolved_at"].is_null(),
        "resolving must be observable on the escalation"
    );
}

#[tokio::test]
async fn resolving_an_escalation_that_was_never_raised_is_404() {
    // Previously patch_escalation acknowledged any id, so a resolution was
    // reported for an escalation that did not exist.
    let app = app();
    let resp = send(
        &app,
        "PATCH",
        "/ecc/escalations/00000000-0000-0000-0000-000000000009",
        Some(json!({"status": "resolved"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn situational_awareness_counts_real_escalations() {
    let app = app();
    let before = body_json(send(&app, "GET", "/ecc/situational-awareness", None).await).await;
    assert_eq!(before["pending_escalations"], 0);

    for _ in 0..2 {
        send(
            &app,
            "POST",
            "/ecc/escalations",
            Some(json!({
                "station_code": "STN-001",
                "category": "c",
                "severity": "high",
                "description": "d",
                "raised_by": "r"
            })),
        )
        .await;
    }
    let after = body_json(send(&app, "GET", "/ecc/situational-awareness", None).await).await;
    assert_eq!(
        after["pending_escalations"], 2,
        "pending_escalations must be derived from real escalations, not a constant"
    );

    // Resolving one must reduce the count.
    let listed = body_json(send(&app, "GET", "/ecc/escalations", None).await).await;
    let id = listed["escalations"][0]["escalation_id"].as_str().unwrap().to_string();
    send(
        &app,
        "PATCH",
        &format!("/ecc/escalations/{id}"),
        Some(json!({"status": "resolved"})),
    )
    .await;
    let final_view = body_json(send(&app, "GET", "/ecc/situational-awareness", None).await).await;
    assert_eq!(final_view["pending_escalations"], 1);
}

#[tokio::test]
async fn audit_log_records_what_actually_happened() {
    // Previously always {"entries": []}.
    let app = app();
    let empty = body_json(send(&app, "GET", "/ecc/audit-log", None).await).await;
    assert!(empty["entries"].as_array().unwrap().is_empty());

    let (_id, ref_code) = complaint(&app).await;
    let log = body_json(send(&app, "GET", "/ecc/audit-log", None).await).await;
    let entries = log["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "filing a complaint must leave an audit entry");
    assert_eq!(entries[0]["kind"], "grievance.complaint.created");
    assert_eq!(entries[0]["subject"], ref_code);
}

#[tokio::test]
async fn audit_trail_for_a_module_reflects_its_own_records() {
    let app = app();
    let flagged = send(
        &app,
        "POST",
        "/iab/flags",
        Some(json!({
            "module_code": "M04",
            "description": "register extract stale",
            "severity": "high",
            "raised_by": "auditor"
        })),
    )
    .await;
    assert_eq!(flagged.status(), StatusCode::CREATED);

    let trails = body_json(send(&app, "GET", "/iab/audit-trails/M04", None).await).await;
    let entries = trails["entries"].as_array().unwrap();
    assert!(
        !entries.is_empty(),
        "a flag raised against M04 must appear in the M04 trail"
    );

    let other = body_json(send(&app, "GET", "/iab/audit-trails/M99", None).await).await;
    assert!(
        other["entries"].as_array().unwrap().is_empty(),
        "another module's trail must not contain this module's entries"
    );
}

#[tokio::test]
async fn audit_report_round_trips_and_is_counted_in_the_summary() {
    let app = app();
    let created = body_json(
        send(
            &app,
            "POST",
            "/iab/audit-reports",
            Some(json!({
                "module_code": "M02",
                "title": "Snapshot reconciliation",
                "body": "finding one\nfinding two\n\nfinding three",
                "submitted_by": "auditor"
            })),
        )
        .await,
    )
    .await;
    let report_id = created["report_id"].as_str().unwrap().to_string();

    let fetched = body_json(
        send(&app, "GET", &format!("/iab/audit-reports/{report_id}"), None).await,
    )
    .await;
    assert_eq!(
        fetched["title"], "Snapshot reconciliation",
        "get_audit_report previously answered with title \"stub\" for any id"
    );

    let listed = body_json(send(&app, "GET", "/iab/audit-reports", None).await).await;
    assert_eq!(listed["reports"].as_array().unwrap().len(), 1);

    let summary = body_json(send(&app, "GET", "/iab/dashboards/summary", None).await).await;
    assert_eq!(
        summary["open_reports"], 1,
        "the IAB summary must count real reports rather than reporting zero"
    );
    assert!(
        !summary["as_of"].as_str().unwrap().starts_with("2027"),
        "as_of was a fixed 2027 timestamp"
    );
}

#[tokio::test]
async fn unknown_audit_report_is_404() {
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/iab/audit-reports/00000000-0000-0000-0000-0000000000aa",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn station_lookups_do_not_invent_stations() {
    let app = app();
    let missing = send(&app, "GET", "/ecc/stations/STN-999", None).await;
    assert_eq!(
        missing.status(),
        StatusCode::NOT_FOUND,
        "get_station previously answered 200 with name \"stub\" for any code"
    );
    assert!(body_json(send(&app, "GET", "/ecc/stations", None).await).await["stations"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn ussd_session_advances_and_can_end() {
    // Previously every request at any step returned the same welcome message and
    // never ended, so nothing could be lodged through USSD.
    let app = app();
    let step = |input: &str| {
        let a = app.clone();
        let input = input.to_string();
        async move {
            body_json(
                send(
                    &a,
                    "POST",
                    "/grievance/ussd/session",
                    Some(json!({
                        "session_id": "S-1",
                        "msisdn": "+254700000002",
                        "input": input,
                        "network_code": "SAFARICOM"
                    })),
                )
                .await,
            )
            .await
        }
    };
    let menu = step("").await;
    assert!(menu["response"].as_str().unwrap().contains("Lodge a complaint"));
    assert_eq!(menu["end_session"], false);

    let lodged = step("1").await;
    assert!(lodged["response"].as_str().unwrap().contains("reference code"));

    let ended = step("0").await;
    assert_eq!(ended["end_session"], true, "a USSD session must be able to end");

    assert_ne!(
        step("1").await["response"],
        step("").await["response"],
        "different steps must not return the same canned response"
    );
}

#[tokio::test]
async fn whatsapp_webhook_refuses_without_signature_verification() {
    // Previously 200 OK for any payload, discarded. Accepting unverified payloads
    // would let anyone post to this endpoint.
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/grievance/whatsapp/webhook",
        Some(json!({"object": "whatsapp_business_account", "entry": {"id": "x"}})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "an unverified webhook must not be accepted as processed"
    );
}

#[tokio::test]
async fn routers_do_not_share_records() {
    // Each build_router constructs its own Records. If they leaked, the empty-state
    // assertions above would pass for the wrong reason.
    let a = app();
    let b = app();
    complaint(&a).await;
    let b_listed = body_json(send(&b, "GET", "/grievance/complaints", None).await).await;
    assert!(
        b_listed["complaints"].as_array().unwrap().is_empty(),
        "records leaked between routers"
    );
}