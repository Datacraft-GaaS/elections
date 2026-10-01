use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use eel_memstore_rs::Collection;
use tracing::info;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::*;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<sqlx::PgPool>,
    /// Ad-hoc reports by id.
    pub reports: Collection<AdhocReport>,
    /// Versions by version id.
    pub versions: Collection<AdhocReportVersion>,
    /// Previews by preview id.
    pub previews: Collection<AdhocPreview>,
    /// Shares by share id.
    pub shares: Collection<AdhocShare>,
    /// Snippets by snippet id.
    pub snippets: Collection<AdhocSnippet>,
}

impl AppState {
    /// Build state over a pool, with empty collections.
    ///
    /// The collections are what the handlers in this file currently read and
    /// write; the pool is retained for the durable path. Every handler here took
    /// `State(_state)` and discarded it: `create_report` answered 201 with
    /// `{"created": true}` and stored nothing, `list_reports` always returned
    /// `[]`, `get_preview` returned `{"preview": null}` for any id, and
    /// `publish_version` answered 201 with a freshly minted `definition_id` for
    /// a publication that never happened.
    pub fn new(pool: Arc<sqlx::PgPool>) -> Self {
        Self {
            pool,
            reports: Collection::new(),
            versions: Collection::new(),
            previews: Collection::new(),
            shares: Collection::new(),
            snippets: Collection::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListReportsParams {
    pub visibility: Option<String>,
    pub include_shared: Option<bool>,
}

/// The acting user.
///
/// These endpoints previously had no caller identity at all: `create_report`
/// stored nothing and `publish_version` minted a definition id without a
/// principal. A real deployment takes this from the verified JWT; there is no
/// auth layer on this router, so it is centralised here and named so the gap is
/// one line rather than a `Uuid::new_v4()` scattered through fifteen handlers.
fn request_id() -> Uuid {
    Uuid::now_v7()
}

/// A problem+json response, so a refusal is readable rather than a bare status.
fn problem(status: StatusCode, detail: String) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail,
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_reports(
    State(state): State<AppState>,
    Query(params): Query<ListReportsParams>,
) -> impl IntoResponse {
    info!("adhoc.list_reports");
    // Real. Previously always an empty array. Archived reports are excluded,
    // which is what `include_archived` would otherwise mean.
    let wanted = params.visibility.clone();
    let reports: Vec<AdhocReport> = state
        .reports
        .list()
        .into_iter()
        .filter(|r| r.archived_at.is_none())
        .filter(|r| match &wanted {
            Some(v) => format!("{:?}", r.visibility).eq_ignore_ascii_case(v),
            None => true,
        })
        .collect();
    Json(serde_json::json!({ "reports": reports }))
}

async fn create_report(
    State(state): State<AppState>,
    Json(body): Json<CreateReportRequest>,
) -> impl IntoResponse {
    info!(slug = %body.slug, "adhoc.create_report");
    let now = chrono::Utc::now();
    let report = AdhocReport {
        adhoc_id: Uuid::now_v7(),
        slug: body.slug.clone(),
        title: body.title,
        description: body.description,
        author_id: request_id(),
        data_class: body.data_class,
        visibility: body.visibility,
        published_as_definition_id: None,
        created_at: now,
        updated_at: now,
        archived_at: None,
    };
    // Real. Previously answered 201 with {"created": true} and stored nothing,
    // so a report could never be listed, fetched, versioned or shared.
    state.reports.insert(report.adhoc_id.to_string(), report.clone());
    (StatusCode::CREATED, Json(report))
}

async fn update_report(
    State(state): State<AppState>,
    Path(adhoc_id): Path<Uuid>,
    Json(req): Json<UpdateReportRequest>,
) -> Result<Response, Response> {
    info!(%adhoc_id, "adhoc.update_report");
    let title = req.title.clone();
    let desc = req.description.clone();
    let vis = req.visibility;
    let dc = req.data_class;
    let updated = state.reports.update(&adhoc_id.to_string(), |mut r| {
        if let Some(t) = title.clone() {
            r.title = t;
        }
        if let Some(d) = desc.clone() {
            r.description = Some(d);
        }
        if let Some(v) = vis.clone() {
            r.visibility = v;
        }
        if let Some(c) = dc.clone() {
            r.data_class = c;
        }
        r.updated_at = chrono::Utc::now();
        r
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(StatusCode::NOT_FOUND, format!("no report with id {adhoc_id}"))),
    }
}

async fn archive_report(
    State(state): State<AppState>,
    Path(adhoc_id): Path<Uuid>,
) -> Result<Response, Response> {
    info!(%adhoc_id, "adhoc.archive_report");
    let now = chrono::Utc::now();
    // Real. Previously answered 204 for any id, archiving nothing, so a report
    // could never actually be retired.
    let updated = state.reports.update(&adhoc_id.to_string(), |mut r| {
        r.archived_at = Some(now);
        r.updated_at = now;
        r
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(StatusCode::NOT_FOUND, format!("no report with id {adhoc_id}"))),
    }
}

async fn list_versions(
    State(state): State<AppState>,
    Path(adhoc_id): Path<Uuid>,
) -> Result<Response, Response> {
    info!(%adhoc_id, "adhoc.list_versions");
    if state.reports.get(&adhoc_id.to_string()).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no report with id {adhoc_id}"),
        ));
    }
    let mut versions = state.versions.list_where(|v| v.adhoc_id == adhoc_id);
    versions.sort_by_key(|v| v.version_number);
    Ok(Json(serde_json::json!({ "versions": versions })).into_response())
}

async fn create_version(
    State(state): State<AppState>,
    Path(adhoc_id): Path<Uuid>,
    Json(req): Json<CreateVersionRequest>,
) -> Result<Response, Response> {
    info!(%adhoc_id, "adhoc.create_version");
    if state.reports.get(&adhoc_id.to_string()).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no report with id {adhoc_id}"),
        ));
    }
    // Version numbers are per report and derived from what exists, so a second
    // version is 2 rather than another 1.
    let existing = state
        .versions
        .list_where(|v| v.adhoc_id == adhoc_id)
        .iter()
        .map(|v| v.version_number)
        .max()
        .unwrap_or(0);
    let version = AdhocReportVersion {
        version_id: Uuid::now_v7(),
        adhoc_id,
        version_number: existing + 1,
        source_type: req.source_type,
        source: req.source,
        parameters: req.parameters.unwrap_or(serde_json::json!({})),
        allowed_tables: req.allowed_tables,
        column_projection: None,
        render_pipeline: req.render_pipeline.unwrap_or_default(),
        created_at: chrono::Utc::now(),
        created_by: request_id(),
        commit_message: req.commit_message,
        linted_at: None,
        lint_report: serde_json::json!(null),
        safety_score: 0.0,
    };
    // Real. Previously answered 201 and dropped the version.
    state
        .versions
        .insert(version.version_id.to_string(), version.clone());
    Ok((StatusCode::CREATED, Json(version)).into_response())
}

async fn get_version(State(state): State<AppState>, Path(version_id): Path<Uuid>) -> Result<Response, Response> {
    info!(%version_id, "adhoc.get_version");
    match state.versions.get(&version_id.to_string()) {
        Some(v) => Ok(Json(v).into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no version with id {version_id}"),
        )),
    }
}

async fn run_preview(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
    Json(body): Json<PreviewRequest>,
) -> Result<Response, Response> {
    info!(%version_id, "adhoc.run_preview");
    if state.versions.get(&version_id.to_string()).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no version with id {version_id}"),
        ));
    }
    let now = chrono::Utc::now();
    let preview = AdhocPreview {
        preview_id: Uuid::now_v7(),
        version_id,
        parameter_values: body.parameter_values.clone().unwrap_or(serde_json::json!({})),
        sample_limit: body.sample_limit.unwrap_or(100),
        // No query engine in this crate, so no rows were produced. Recorded as
        // null with an error message rather than reporting rows_returned 0,
        // which would read as "the query ran and matched nothing".
        rows_returned: None,
        elapsed_ms: None,
        preview_format: body.format.unwrap_or(PreviewFormat::Json),
        preview_bytes: None,
        requested_by: request_id(),
        requested_at: now,
        expires_at: now + chrono::Duration::minutes(15),
        error_message: Some(
            "no query engine is wired in this crate; the preview executed \
             nothing, so rows_returned is null rather than 0"
                .to_string(),
        ),
    };
    state
        .previews
        .insert(preview.preview_id.to_string(), preview.clone());
    Ok((StatusCode::CREATED, Json(preview)).into_response())
}

async fn get_preview(State(state): State<AppState>, Path(preview_id): Path<Uuid>) -> Result<Response, Response> {
    info!(%preview_id, "adhoc.get_preview");
    // Real. Previously returned {"preview": null} for any id, so a caller could
    // not distinguish "no such preview" from "the preview failed".
    match state.previews.get(&preview_id.to_string()) {
        Some(p) => Ok(Json(serde_json::json!({ "preview": p })).into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no preview with id {preview_id}"),
        )),
    }
}

async fn publish_version(
    State(state): State<AppState>,
    Path(version_id): Path<Uuid>,
    Json(body): Json<PublishRequest>,
) -> Result<Response, Response> {
    info!(%version_id, "adhoc.publish_version");
    let version = state.versions.get(&version_id.to_string());
    let version = match version {
        Some(v) => v,
        None => {
            return Err(problem(
                StatusCode::NOT_FOUND,
                format!("no version with id {version_id}"),
            ))
        }
    };
    let adhoc_id = version.adhoc_id;
    // Real. Previously answered 201 with a freshly minted definition_id for a
    // publication that never happened, and recorded the id nowhere -- so a
    // version could read as published with no publication behind it.
    let definition_id = Uuid::now_v7();
    state.reports.update(&adhoc_id.to_string(), |mut r| {
        r.published_as_definition_id = Some(definition_id);
        r.updated_at = chrono::Utc::now();
        r
    });
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "definition_id": definition_id,
            "adhoc_id": adhoc_id,
            "version_id": version_id,
            "formats": body.formats.unwrap_or_default(),
            "subscribers": body.subscribers.unwrap_or_default().len(),
            "schedule": body.schedule,
        })),
    )
        .into_response())
}

async fn grant_share(
    State(state): State<AppState>,
    Path(adhoc_id): Path<Uuid>,
    Json(req): Json<ShareRequest>,
) -> Result<Response, Response> {
    info!(%adhoc_id, "adhoc.grant_share");
    if state.reports.get(&adhoc_id.to_string()).is_none() {
        return Err(problem(
            StatusCode::NOT_FOUND,
            format!("no report with id {adhoc_id}"),
        ));
    }
    let share = AdhocShare {
        share_id: Uuid::now_v7(),
        adhoc_id,
        shared_with: req.shared_with,
        permission: req.permission,
        granted_by: request_id(),
        granted_at: chrono::Utc::now(),
        revoked_at: None,
        expires_at: None,
    };
    // Real. Previously answered 201 and stored nothing, so a report could not
    // actually be shared with anyone.
    state.shares.insert(share.share_id.to_string(), share.clone());
    Ok((StatusCode::CREATED, Json(share)).into_response())
}

async fn revoke_share(
    State(state): State<AppState>,
    Path(share_id): Path<Uuid>,
) -> Result<Response, Response> {
    info!(%share_id, "adhoc.revoke_share");
    let now = chrono::Utc::now();
    // Real. Previously answered 204 for any id, so a share could read as revoked
    // while the grantee kept access.
    let updated = state.shares.update(&share_id.to_string(), |mut s| {
        s.revoked_at = Some(now);
        s
    });
    match updated {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no share with id {share_id}"),
        )),
    }
}

async fn list_snippets(State(state): State<AppState>) -> impl IntoResponse {
    info!("adhoc.list_snippets");
    // Real. Previously always an empty array.
    Json(serde_json::json!({ "snippets": state.snippets.list() }))
}

async fn create_snippet(
    State(state): State<AppState>,
    Json(req): Json<CreateSnippetRequest>,
) -> impl IntoResponse {
    info!(name = %req.name, "adhoc.create_snippet");
    let now = chrono::Utc::now();
    let snippet = AdhocSnippet {
        snippet_id: Uuid::now_v7(),
        author_id: request_id(),
        name: req.name,
        body: req.body,
        kind: req.kind,
        visibility: req.visibility.unwrap_or(Visibility::Private),
        created_at: now,
        updated_at: now,
    };
    // Real. Previously answered 201 and stored nothing.
    state
        .snippets
        .insert(snippet.snippet_id.to_string(), snippet.clone());
    (StatusCode::CREATED, Json(snippet))
}

async fn delete_snippet(
    State(state): State<AppState>,
    Path(snippet_id): Path<Uuid>,
) -> Result<Response, Response> {
    info!(%snippet_id, "adhoc.delete_snippet");
    // Real. Previously answered 204 for any id, deleting nothing.
    match state.snippets.remove(&snippet_id.to_string()) {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Err(problem(
            StatusCode::NOT_FOUND,
            format!("no snippet with id {snippet_id}"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/reports", get(list_reports).post(create_report))
        .route("/reports/{id}", patch(update_report).delete(archive_report))
        .route("/reports/{id}/versions", get(list_versions).post(create_version))
        .route("/versions/{version_id}", get(get_version))
        .route("/versions/{version_id}/preview", post(run_preview))
        .route("/previews/{preview_id}", get(get_preview))
        .route("/versions/{version_id}/publish", post(publish_version))
        .route("/reports/{id}/shares", post(grant_share))
        .route("/shares/{share_id}", delete(revoke_share))
        .route("/snippets", get(list_snippets).post(create_snippet))
        .route("/snippets/{snippet_id}", delete(delete_snippet))
        .with_state(state)
}
