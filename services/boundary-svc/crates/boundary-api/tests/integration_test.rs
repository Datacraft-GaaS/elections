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

//! Integration tests for boundary-svc.
//!
//! Uses axum::Router directly (no real DB in CI; testcontainers-postgres
//! for the PostGIS path is a separate test that requires `--features db`).

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; // for `oneshot`

fn app() -> axum::Router {
    boundary_api::routes::router()
}

#[tokio::test]
async fn list_constituencies_returns_200() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/boundary/constituencies")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_counties_returns_200() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/boundary/counties")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_wards_returns_200() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/boundary/wards")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_stations_returns_200() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/boundary/stations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn unknown_constituency_returns_404() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/boundary/constituencies/UNKNOWN")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn bulk_geojson_returns_200() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/api/v1/boundaries/bulk.geojson.gz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn upload_geofence_unsigned_polygon_rejected() {
    use axum::http::header::CONTENT_TYPE;
    let body = serde_json::json!({
        "station_code": "KE001PS001"
        // no "signature" field
    });
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/geofence/upload")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Unsigned polygon must be rejected with 422
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
