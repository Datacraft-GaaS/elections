use axum::{body::Body, http::Request};
use special_voter::{
    http::{router, AppState},
    spec::{EventRequest, Facility},
};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn router_smoke() {
    let state = AppState::new_shared();
    let app = router(state);
    drop(app);
}

#[tokio::test]
async fn contract_submit_and_list_events() {
    let state = AppState::new_shared();
    // Seed a facility directly
    let facility = Facility {
        id: Uuid::now_v7(),
        category: "REMAND_PRISON".to_string(),
        county_code: "001".to_string(),
        presiding_officer_id: "officer-1".to_string(),
        voting_date: "2026-08-13".to_string(),
        published_voter_count: Some(100),
        session_state: "OPEN".to_string(),
        violation_pending_ack: None,
    };
    let fid = facility.id;
    state.facilities.lock().expect("lock poisoned").insert(fid, facility);

    let app = router(state);

    // Submit event
    let req_body = serde_json::to_value(EventRequest {
        event_type: "OPEN".to_string(),
        officer_id: "officer-1".to_string(),
        witness_ids: vec!["w1".to_string()],
        reason_code: None,
    })
    .unwrap();
    let req = Request::builder()
        .method("POST")
        .uri(format!("/v1/special/facilities/{fid}/events"))
        .header("content-type", "application/json")
        .body(Body::from(req_body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 201);

    // List events
    let req = Request::builder()
        .uri(format!("/v1/special/facilities/{fid}/events"))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v["events"].as_array().unwrap().len() == 1);
}
