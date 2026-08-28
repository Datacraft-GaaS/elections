use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;

fn mem_app() -> axum::Router {
    ecc_api::router::build_router(ecc_api::store::Store::Mem(Default::default()))
}

#[tokio::test]
async fn health_returns_200() {
    let app = ecc_api::router::build_router(ecc_api::store::Store::Mem(Default::default()));
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn situational_awareness_returns_200() {
    let app = ecc_api::router::build_router(ecc_api::store::Store::Mem(Default::default()));
    let req = Request::builder()
        .uri("/ecc/situational-awareness")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_complaints_returns_200() {
    let app = ecc_api::router::build_router(ecc_api::store::Store::Mem(Default::default()));
    let req = Request::builder()
        .uri("/grievance/complaints")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn s44a_dual_officer_flow_completes() {
    let app = mem_app();

    // Officer 1 confirms.
    let req = Request::builder()
        .method("POST")
        .uri("/ecc/s44a/officer1")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"county_code":"KE-030","station_code":"POLL-NRB-001","officer_id":"off-1"}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Duplicate officer 1 for the same station conflicts.
    let req = Request::builder()
        .method("POST")
        .uri("/ecc/s44a/officer1")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"county_code":"KE-030","station_code":"POLL-NRB-001","officer_id":"off-1b"}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    // Officer 2 completes the dual-officer verification.
    let req = Request::builder()
        .method("POST")
        .uri("/ecc/s44a/officer2")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"station_code":"POLL-NRB-001","officer_id":"off-2"}"#,
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["status"], "complete");
    assert_eq!(v["station_code"], "POLL-NRB-001");

    // A second officer 2 has nothing pending.
    let req = Request::builder()
        .method("POST")
        .uri("/ecc/s44a/officer2")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"station_code":"POLL-NRB-001","officer_id":"off-3"}"#,
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}
