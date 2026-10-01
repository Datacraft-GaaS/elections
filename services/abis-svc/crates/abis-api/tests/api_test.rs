use std::sync::Arc;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use abis_api::{app, state::AppState};

fn db_url() -> String {
    std::env::var("EEL_ABIS_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://127.0.0.1:5432/eel_abis".to_string())
}

fn test_app() -> axum::Router {
    app(Arc::new(AppState::new()))
}

/// Connect to the database, or `None` when none is reachable.
///
/// Integration tests that touch the store use this and skip gracefully when
/// Postgres is unavailable (CI provides it via a service + migrations).
async fn db_pool() -> Option<sqlx::PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect(&db_url())
        .await
        .ok()?;
    sqlx::query("SELECT 1").execute(&pool).await.ok()?;
    Some(pool)
}

/// Seed the minimal crypto/blob chain the match handlers require:
/// a KEK generation and a probe blob manifest row (see `eel_match_requests`
/// → `eel_blob_manifest` → `eel_kek_generations` FKs).
async fn seed_probe(pool: &sqlx::PgPool, blob_ref: &str) {
    sqlx::query(
        "INSERT INTO eel_kek_generations (kek_version, hsm_key_id, cycle_id) \
         VALUES (1, 'test-hsm-key', 'test-cycle') \
         ON CONFLICT (kek_version) DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("seed kek generation");

    sqlx::query(
        "INSERT INTO eel_blob_manifest \
             (blob_ref, sha256, size_bytes, kek_version, dek_wrapped, \
              minio_bucket, minio_object_key, class, retention_until) \
         VALUES ($1, $2, 64, 1, $3, 'test-bucket', $4, 'public', NOW() + INTERVAL '1 year') \
         ON CONFLICT (blob_ref) DO NOTHING",
    )
    .bind(blob_ref)
    .bind([0u8; 32].to_vec())
    .bind([0u8; 32].to_vec())
    .bind(format!("probes/{blob_ref}"))
    .execute(pool)
    .await
    .expect("seed probe blob manifest");
}

fn json_body(s: &str) -> Body {
    Body::from(s.to_string())
}

#[tokio::test]
async fn test_healthz() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_match_1n() {
    let Some(pool) = db_pool().await else {
        eprintln!("skipping: no reachable ABIS database");
        return;
    };
    seed_probe(&pool, "p1").await;
    let app = app(Arc::new(AppState { db: pool, store: abis_api::store::AbisStore::new() }));
    let body = r#"{"probe_ref":"p1","modality":"fingerprint","caller_id":"c1","election_id":"33333333-2222-3333-4444-555555555555"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/abis/match/1n")
                .header("content-type", "application/json")
                .header("idempotency-key", "11111111-2222-3333-4444-555555555555")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_match_11() {
    let Some(pool) = db_pool().await else {
        eprintln!("skipping: no reachable ABIS database");
        return;
    };
    seed_probe(&pool, "p1").await;
    let app = app(Arc::new(AppState { db: pool, store: abis_api::store::AbisStore::new() }));
    let body = r#"{"probe_ref":"p1","gallery_ref":"g1","modality":"iris","caller_id":"c1"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/abis/match/11")
                .header("content-type", "application/json")
                .header("idempotency-key", "22222222-2222-3333-4444-555555555555")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_match_result() {
    let Some(pool) = db_pool().await else {
        eprintln!("skipping: no reachable ABIS database");
        return;
    };
    seed_probe(&pool, "p2").await;
    let app = app(Arc::new(AppState { db: pool, store: abis_api::store::AbisStore::new() }));

    // Mint a request row via 1:n, then fetch its result back.
    let body = r#"{"probe_ref":"p2","modality":"fingerprint","caller_id":"c1","election_id":"33333333-2222-3333-4444-555555555555"}"#;
    let submit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/abis/match/1n")
                .header("content-type", "application/json")
                .header("idempotency-key", "44444444-2222-3333-4444-555555555555")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(submit.status(), StatusCode::ACCEPTED);

    let bytes = axum::body::to_bytes(submit.into_body(), 1_000_000).await.unwrap();
    let request_id = serde_json::from_slice::<serde_json::Value>(&bytes)
        .unwrap()["request_id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/abis/match/{request_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

/// Assert an endpoint refuses rather than inventing a success.
///
/// These four handlers used to answer 200/202 with a fabricated biometric
/// verdict, a job id for work never queued, an empty adjudication page and a
/// zeroed metrics summary. The tests below asserted only `StatusCode::OK`, so
/// the suite was green while the endpoints lied: a green test here meant the
/// fabrication was reachable, not that it was correct. They now assert the
/// refusal *and* that the body says why, which is the part a caller reads.
async fn assert_refuses_not_invents(app: axum::Router, req: Request<Body>) {
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::NOT_IMPLEMENTED,
        "expected 501 rather than a fabricated success"
    );
    assert_eq!(
        resp.headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/problem+json"),
        "a refusal must be problem+json so callers can read it"
    );
    let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], 501);
    let detail = body["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("no") || detail.contains("never"),
        "detail should say the operation is unimplemented, got: {detail}"
    );
}

#[tokio::test]
async fn test_list_adjudication() {
    assert_refuses_not_invents(
        test_app(),
        Request::builder()
            .uri("/api/v1/abis/adjudication")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
}

#[tokio::test]
async fn test_resolve_adjudication() {
    let app = test_app();
    let body = r#"{"outcome":"no_match","adjudicator_id":"adj-1","resolution_notes":"distinct"}"#;
    assert_refuses_not_invents(
        app,
        Request::builder()
            .method("PATCH")
            .uri("/api/v1/abis/adjudication/case-001")
            .header("content-type", "application/json")
            .body(json_body(body))
            .unwrap(),
    )
    .await;
}

#[tokio::test]
async fn test_submit_batch_dedup() {
    let app = test_app();
    let body = r#"{"election_id":"e1","mode":"full","requester_id":"r1"}"#;
    assert_refuses_not_invents(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/abis/batch/dedup")
            .header("content-type", "application/json")
            .body(json_body(body))
            .unwrap(),
    )
    .await;
}

#[tokio::test]
async fn test_get_reports() {
    assert_refuses_not_invents(
        test_app(),
        Request::builder()
            .uri("/api/v1/abis/reports")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
}

#[tokio::test]
async fn test_kit_enrol_requires_attestation() {
    let app = test_app();
    let body = r#"{"subject_id":"00000000-0000-4000-8000-000000000001","operator_id":"op-1","kit_id":"KIT-1","templates":[]}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/kit/enrolment")
                .header("content-type", "application/json")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_kit_enrol_rejects_non_uuid_subject() {
    let app = test_app();
    let body = r#"{"subject_id":"not-a-uuid","operator_id":"op-1","kit_id":"KIT-1","templates":[{"modality":"iris","data_b64":"eA==","quality":0.9}]}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/kit/enrolment")
                .header("content-type", "application/json")
                .header("x-kit-attestation", "test-attestation")
                .body(json_body(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
