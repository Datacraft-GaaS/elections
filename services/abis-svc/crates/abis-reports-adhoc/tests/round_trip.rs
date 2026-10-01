//! Round-trip tests for the ad-hoc report endpoints.
//!
//! Every handler here took `State(_state)` and discarded it. `create_report`
//! answered 201 with `{"created": true}` and stored nothing, so a report could
//! never be listed, versioned, shared or archived. `get_preview` returned
//! `{"preview": null}` for any id, so a caller could not distinguish "no such
//! preview" from "the preview failed". `publish_version` answered 201 with a
//! freshly minted `definition_id` for a publication that never happened and
//! recorded the id nowhere, so a version could read as published with nothing
//! behind it. `revoke_share` and `delete_snippet` answered 204 for any id.
//!
//! Every test here fails against that code.

use eel_abis_reports_adhoc::{
    api::{router, AppState},
    domain::*,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

/// A lazily-connected pool: these endpoints read from the in-process
/// collections, and `connect_lazy` needs no database to construct.
fn app() -> axum::Router {
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(1))
        .connect_lazy("postgres://127.0.0.1:5432/eel_abis")
        .expect("lazy pool");
    router(AppState::new(std::sync::Arc::new(pool)))
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

fn report_body() -> Value {
    json!({
        "slug": "turnout-by-county",
        "title": "Turnout by county",
        "description": "Ad-hoc turnout extract",
        "data_class": "public",
        "visibility": "private"
    })
}

async fn create_report(app: &axum::Router) -> String {
    let resp = send(app, "POST", "/reports", Some(report_body())).await;
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "create_report must work for the rest to be reachable"
    );
    let created = body_json(resp).await;
    assert_ne!(
        created.get("created").and_then(|v| v.as_bool()),
        Some(true),
        "create_report used to answer created:true and store nothing"
    );
    created["adhoc_id"].as_str().expect("adhoc_id").to_string()
}

#[tokio::test]
async fn created_report_is_listed_and_updated() {
    let app = app();
    let id = create_report(&app).await;

    let listed = body_json(send(&app, "GET", "/reports", None).await).await;
    let reports = listed["reports"].as_array().unwrap();
    assert_eq!(
        reports.len(),
        1,
        "create_report answered 201 and dropped the report, so this was always empty"
    );
    assert_eq!(reports[0]["adhoc_id"], id.as_str());
    assert_eq!(reports[0]["title"], "Turnout by county");

    let updated = send(
        &app,
        "PATCH",
        &format!("/reports/{id}"),
        Some(json!({"title": "Turnout by county (revised)", "description": null, "visibility": null, "data_class": null})),
    )
    .await;
    assert_eq!(updated.status(), StatusCode::NO_CONTENT);

    let after = body_json(send(&app, "GET", "/reports", None).await).await;
    assert_eq!(after["reports"][0]["title"], "Turnout by county (revised)");
}

#[tokio::test]
async fn archived_report_leaves_the_list() {
    let app = app();
    let id = create_report(&app).await;
    assert_eq!(
        send(&app, "DELETE", &format!("/reports/{id}"), None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    let listed = body_json(send(&app, "GET", "/reports", None).await).await;
    assert!(
        listed["reports"].as_array().unwrap().is_empty(),
        "archive_report previously returned 204 for any id, archiving nothing"
    );
}

#[tokio::test]
async fn updating_or_archiving_an_unknown_report_is_404() {
    let app = app();
    let ghost = "00000000-0000-0000-0000-0000000000aa";
    assert_eq!(
        send(&app, "PATCH", &format!("/reports/{ghost}"), Some(json!({"title": "x", "description": null, "visibility": null, "data_class": null}))).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send(&app, "DELETE", &format!("/reports/{ghost}"), None).await.status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn versions_are_numbered_per_report_and_listed() {
    let app = app();
    let id = create_report(&app).await;

    for msg in ["first cut", "second cut"] {
        let resp = send(
            &app,
            "POST",
            &format!("/reports/{id}/versions"),
            Some(json!({
                "source_type": "sql",
                "source": "SELECT county, SUM(votes) FROM results GROUP BY county",
                "parameters": null,
                "allowed_tables": ["results"],
                "commit_message": msg,
                "render_pipeline": null
            })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::CREATED, "create_version must work");
    }

    let listed = body_json(send(&app, "GET", &format!("/reports/{id}/versions"), None).await).await;
    let versions = listed["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2, "create_version previously stored nothing");
    assert_eq!(versions[0]["version_number"], 1);
    assert_eq!(versions[1]["version_number"], 2, "version numbers must increment");
    assert_eq!(versions[1]["commit_message"], "second cut");

    let fetched = body_json(
        send(&app, "GET", &format!("/versions/{}", versions[1]["version_id"].as_str().unwrap()), None).await,
    )
    .await;
    assert_eq!(fetched["version_number"], 2);
}

#[tokio::test]
async fn a_version_on_an_unknown_report_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/reports/00000000-0000-0000-0000-0000000000bb/versions",
        Some(json!({
            "source_type": "sql",
            "source": "SELECT 1",
            "parameters": null,
            "allowed_tables": [],
            "commit_message": null,
            "render_pipeline": null
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn publishing_a_version_records_it_on_the_report() {
    // Previously answered 201 with a minted definition_id and recorded nothing,
    // so a version could read as published with no publication behind it.
    let app = app();
    let id = create_report(&app).await;
    let v = body_json(
        send(
            &app,
            "POST",
            &format!("/reports/{id}/versions"),
            Some(json!({
                "source_type": "sql",
                "source": "SELECT 1",
                "parameters": null,
                "allowed_tables": [],
                "commit_message": null,
                "render_pipeline": null
            })),
        )
        .await,
    )
    .await;
    let version_id = v["version_id"].as_str().unwrap().to_string();

    let resp = send(
        &app,
        "POST",
        &format!("/versions/{version_id}/publish"),
        Some(json!({"schedule": null, "subscribers": null, "formats": ["json"]})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let published = body_json(resp).await;
    let definition_id = published["definition_id"].as_str().unwrap().to_string();
    assert_eq!(published["adhoc_id"], id.as_str());

    // The report now carries it, which is what makes "published" a fact.
    let reports = body_json(send(&app, "GET", "/reports", None).await).await;
    assert_eq!(
        reports["reports"][0]["published_as_definition_id"].as_str(),
        Some(definition_id.as_str()),
        "the definition id must be recorded on the report, not minted and dropped"
    );
}

#[tokio::test]
async fn publishing_an_unknown_version_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/versions/00000000-0000-0000-0000-0000000000cc/publish",
        Some(json!({"schedule": null, "subscribers": null, "formats": null})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_preview_reports_that_it_executed_nothing() {
    // No query engine is wired, so rows_returned stays null with an error
    // message. Reporting 0 would read as "the query ran and matched nothing".
    let app = app();
    let id = create_report(&app).await;
    let v = body_json(
        send(
            &app,
            "POST",
            &format!("/reports/{id}/versions"),
            Some(json!({
                "source_type": "sql",
                "source": "SELECT 1",
                "parameters": null,
                "allowed_tables": [],
                "commit_message": null,
                "render_pipeline": null
            })),
        )
        .await,
    )
    .await;
    let version_id = v["version_id"].as_str().unwrap().to_string();

    let resp = send(
        &app,
        "POST",
        &format!("/versions/{version_id}/preview"),
        Some(json!({
            "parameter_values": null,
            "sample_limit": null,
            "format": null,
            "idempotency_key": null
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let preview = body_json(resp).await;
    assert!(preview["rows_returned"].is_null());
    assert!(
        !preview["error_message"].is_null(),
        "a preview that executed nothing must say so"
    );
    let preview_id = preview["preview_id"].as_str().unwrap().to_string();

    let fetched = body_json(send(&app, "GET", &format!("/previews/{preview_id}"), None).await).await;
    assert_eq!(fetched["preview"]["preview_id"], preview_id.as_str());
}

#[tokio::test]
async fn get_preview_of_an_unknown_id_is_404() {
    // Previously {"preview": null} for any id, which cannot be distinguished
    // from a failed preview.
    let app = app();
    let resp = send(
        &app,
        "GET",
        "/previews/00000000-0000-0000-0000-0000000000dd",
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn snippet_round_trips_and_deletes() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/snippets",
        Some(json!({
            "name": "county filter",
            "body": "WHERE county = :county",
            "kind": "where",
            "visibility": null
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let id = created["snippet_id"].as_str().unwrap().to_string();
    assert_eq!(created["visibility"], "private", "defaults to private, not open");

    let listed = body_json(send(&app, "GET", "/snippets", None).await).await;
    assert_eq!(listed["snippets"].as_array().unwrap().len(), 1);

    assert_eq!(
        send(&app, "DELETE", &format!("/snippets/{id}"), None).await.status(),
        StatusCode::NO_CONTENT
    );
    assert!(body_json(send(&app, "GET", "/snippets", None).await).await["snippets"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn deleting_an_unknown_snippet_is_404() {
    let app = app();
    let resp = send(&app, "DELETE", "/snippets/00000000-0000-0000-0000-0000000000ee", None).await;
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "delete_snippet previously returned 204 for any id"
    );
}

#[tokio::test]
async fn a_share_can_be_granted_and_revoked() {
    let app = app();
    let id = create_report(&app).await;
    let other = "00000000-0000-0000-0000-0000000000ff";

    let resp = send(
        &app,
        "POST",
        &format!("/reports/{id}/shares"),
        Some(json!({"shared_with": other, "permission": "view"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let share = body_json(resp).await;
    let share_id = share["share_id"].as_str().unwrap().to_string();
    assert!(share["revoked_at"].is_null());

    assert_eq!(
        send(&app, "DELETE", &format!("/shares/{share_id}"), None).await.status(),
        StatusCode::NO_CONTENT
    );
    // Revoking an unknown share must not report success.
    assert_eq!(
        send(&app, "DELETE", "/shares/00000000-0000-0000-0000-0000000000ab", None)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn sharing_an_unknown_report_is_404() {
    let app = app();
    let resp = send(
        &app,
        "POST",
        "/reports/00000000-0000-0000-0000-0000000000ac/shares",
        Some(json!({"shared_with": "00000000-0000-0000-0000-0000000000ff", "permission": "view"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn routers_do_not_share_records() {
    let a = app();
    let b = app();
    create_report(&a).await;
    assert!(
        body_json(send(&b, "GET", "/reports", None).await).await["reports"]
            .as_array()
            .unwrap()
            .is_empty(),
        "records leaked between routers"
    );
}

#[tokio::test]
async fn visibility_filter_is_applied() {
    let app = app();
    create_report(&app).await;
    let priv_reports = body_json(send(&app, "GET", "/reports?visibility=private", None).await).await;
    assert_eq!(priv_reports["reports"].as_array().unwrap().len(), 1);
    let other = body_json(send(&app, "GET", "/reports?visibility=org", None).await).await;
    assert!(
        other["reports"].as_array().unwrap().is_empty(),
        "the visibility filter must actually filter"
    );
}