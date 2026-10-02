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

use chrono::Utc;
use uuid::Uuid;

use crate::{
    domain::*,
    error::{AdhocError, Result},
};

pub async fn list_reports(
    _pool: &sqlx::PgPool,
    _author_id: Uuid,
    _visibility: Option<Visibility>,
    _include_shared: bool,
) -> Result<Vec<AdhocReport>> {
    Ok(Vec::new())
}

pub async fn create_report(
    _pool: &sqlx::PgPool,
    author_id: Uuid,
    req: CreateReportRequest,
) -> Result<AdhocReport> {
    let now = Utc::now();
    let report = AdhocReport {
        adhoc_id: Uuid::now_v7(),
        slug: req.slug,
        title: req.title,
        description: req.description,
        author_id,
        data_class: req.data_class,
        visibility: req.visibility,
        published_as_definition_id: None,
        created_at: now,
        updated_at: now,
        archived_at: None,
    };
    Ok(report)
}

pub async fn update_report(
    _pool: &sqlx::PgPool,
    adhoc_id: Uuid,
    _author_id: Uuid,
    _req: UpdateReportRequest,
) -> Result<AdhocReport> {
    Err(AdhocError::NotFound(adhoc_id.to_string()))
}

pub async fn archive_report(
    _pool: &sqlx::PgPool,
    _adhoc_id: Uuid,
    _author_id: Uuid,
) -> Result<()> {
    Ok(())
}

pub async fn create_version(
    _pool: &sqlx::PgPool,
    adhoc_id: Uuid,
    created_by: Uuid,
    req: CreateVersionRequest,
    lint_report: serde_json::Value,
    safety_score: f32,
) -> Result<AdhocReportVersion> {
    let now = Utc::now();
    let version = AdhocReportVersion {
        version_id: Uuid::now_v7(),
        adhoc_id,
        version_number: 1,
        source_type: req.source_type,
        source: req.source,
        parameters: req.parameters.unwrap_or_else(|| serde_json::json!([])),
        allowed_tables: req.allowed_tables,
        column_projection: None,
        render_pipeline: req.render_pipeline.unwrap_or_else(|| {
            vec!["pdf".to_string(), "docx".to_string(), "markdown".to_string(), "csv".to_string(), "json".to_string()]
        }),
        created_at: now,
        created_by,
        commit_message: req.commit_message,
        linted_at: Some(now),
        lint_report,
        safety_score,
    };
    Ok(version)
}

pub async fn fetch_latest_version(
    _pool: &sqlx::PgPool,
    adhoc_id: Uuid,
) -> Result<AdhocReportVersion> {
    Err(AdhocError::NotFound(adhoc_id.to_string()))
}

pub async fn fetch_version(
    _pool: &sqlx::PgPool,
    version_id: Uuid,
) -> Result<AdhocReportVersion> {
    Err(AdhocError::NotFound(version_id.to_string()))
}

pub async fn create_preview(
    _pool: &sqlx::PgPool,
    version_id: Uuid,
    requested_by: Uuid,
    req: PreviewRequest,
) -> Result<AdhocPreview> {
    let now = Utc::now();
    let preview = AdhocPreview {
        preview_id: Uuid::now_v7(),
        version_id,
        parameter_values: req.parameter_values.unwrap_or_else(|| serde_json::json!({})),
        sample_limit: req.sample_limit.unwrap_or(100).min(1000),
        rows_returned: None,
        elapsed_ms: None,
        preview_format: req.format.unwrap_or(PreviewFormat::Json),
        preview_bytes: None,
        requested_by,
        requested_at: now,
        expires_at: now + chrono::Duration::hours(2),
        error_message: None,
    };
    Ok(preview)
}

pub async fn fetch_preview(
    _pool: &sqlx::PgPool,
    preview_id: Uuid,
) -> Result<AdhocPreview> {
    Err(AdhocError::NotFound(preview_id.to_string()))
}

pub async fn purge_expired_previews(_pool: &sqlx::PgPool) -> Result<u64> {
    Ok(0)
}

pub async fn create_share(
    _pool: &sqlx::PgPool,
    adhoc_id: Uuid,
    granted_by: Uuid,
    req: ShareRequest,
) -> Result<AdhocShare> {
    let now = Utc::now();
    Ok(AdhocShare {
        share_id: Uuid::now_v7(),
        adhoc_id,
        shared_with: req.shared_with,
        permission: req.permission,
        granted_by,
        granted_at: now,
        revoked_at: None,
        expires_at: req.expires_at,
    })
}

pub async fn revoke_share(
    _pool: &sqlx::PgPool,
    _share_id: Uuid,
    _revoker_id: Uuid,
) -> Result<()> {
    Ok(())
}

pub async fn list_shares(
    _pool: &sqlx::PgPool,
    _adhoc_id: Uuid,
) -> Result<Vec<AdhocShare>> {
    Ok(Vec::new())
}

pub async fn create_snippet(
    _pool: &sqlx::PgPool,
    author_id: Uuid,
    req: CreateSnippetRequest,
) -> Result<AdhocSnippet> {
    let now = Utc::now();
    Ok(AdhocSnippet {
        snippet_id: Uuid::now_v7(),
        author_id,
        name: req.name,
        body: req.body,
        kind: req.kind,
        visibility: req.visibility.unwrap_or(Visibility::Private),
        created_at: now,
        updated_at: now,
    })
}

pub async fn list_snippets(
    _pool: &sqlx::PgPool,
    _author_id: Uuid,
) -> Result<Vec<AdhocSnippet>> {
    Ok(Vec::new())
}

pub async fn delete_snippet(
    _pool: &sqlx::PgPool,
    _snippet_id: Uuid,
    _author_id: Uuid,
) -> Result<()> {
    Ok(())
}
