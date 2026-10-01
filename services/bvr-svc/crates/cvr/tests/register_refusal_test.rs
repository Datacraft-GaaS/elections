//! The CVR register endpoints refuse instead of answering.
//!
//! There were no tests here before. That is itself the finding: three handlers that
//! returned a definitive `registered: false`, an empty page and a zeroed register
//! had no test asserting any of it, so nothing would have caught the fabrication
//! either way.
//!
//! Each test names the behaviour it replaced.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use cvr::state::AppState;

fn state() -> AppState {
    // The handlers under test never touch the pool; a lazy handle is enough and no
    // database is needed to run these.
    AppState::new(
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://localhost/cvr")
            .expect("lazy pool"),
    )
}

async fn post(uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = cvr::routes::spec::router()
        .with_state(state())
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn get(uri: &str) -> (StatusCode, Value) {
    let resp = cvr::routes::spec::router()
        .with_state(state())
        .oneshot(Request::builder().uri(uri).method("GET").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
async fn a_lookup_never_claims_a_voter_is_not_registered() {
    // Previously: 200 with `registered: false` and three nulls, for every request.
    // A false negative here can turn away a registered voter.
    let (status, body) = post("/api/v1/cvr/lookups", json!({ "id_number_hash": "abc123" })).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    // The point of the test: nothing in the body asserts a registration status.
    assert!(body.get("registered").is_none(), "the body must not assert a registration status");
    let detail = body["error"].as_str().unwrap_or_default();
    assert!(detail.contains("registration service"), "{detail}");
    assert!(detail.contains("eel_voter_register"), "{detail}");
}

#[tokio::test]
async fn the_snapshot_refusal_says_why() {
    // Previously: 404 "no snapshot available" for every request, which reads as no
    // snapshot having been published.
    let (status, body) = get("/api/v1/cvr/register/snapshot").await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(body["error"].as_str().unwrap_or_default().contains("MinIO"));
}

#[tokio::test]
async fn the_stats_refusal_says_why() {
    // Previously: 200 with `total_registered: 0, counties: 0`, which reads as a
    // measured register that happens to be empty.
    let (status, body) = get("/api/v1/cvr/register/stats").await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    let detail = body["error"].as_str().unwrap_or_default();
    assert!(detail.contains("county"), "{detail}");
}
