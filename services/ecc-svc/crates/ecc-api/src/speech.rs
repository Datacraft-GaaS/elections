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

//! Speech-integrity (M17) public contract handlers.
//!
//! These expose the existing [`Store`](crate::store::Store) speech-integrity
//! methods over the `ecc/speech.yaml` spec paths.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tracing::info;
use uuid::Uuid;

use crate::store::{AdjudicationRow, FindingRow, MediaItemRow, Store, StoreError};

// ---------------------------------------------------------------------------
// Spec DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MediaItemSubmission {
    pub url: String,
    pub media_type: String,
    pub language_code: String,
    pub source_platform: String,
    pub captured_at: String,
}

#[derive(Debug, Serialize)]
pub struct MediaItem {
    pub item_id: String,
    pub status: String,
    pub submitted_at: String,
}

#[derive(Debug, Serialize)]
pub struct MediaItemSummary {
    pub item_id: String,
    pub media_type: String,
    pub language_code: String,
    pub source_platform: String,
    pub status: String,
    pub top_category: Option<String>,
    pub captured_at: String,
    pub submitted_at: String,
}

#[derive(Debug, Serialize)]
pub struct AnalysisFinding {
    pub category: String,
    pub confidence: f64,
    pub excerpt: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MediaItemDetail {
    pub item_id: String,
    pub media_type: String,
    pub language_code: String,
    pub source_platform: String,
    pub status: String,
    pub findings: Vec<AnalysisFinding>,
    pub analyst_notes: Option<String>,
    pub decided_at: Option<String>,
    pub verdict: Option<String>,
    pub submitted_at: String,
    pub captured_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SpeechAdjudication {
    pub verdict: String,
    pub grounds: String,
    pub referral_agency: String,
    pub analyst_notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdjudicationResult {
    pub adjudication_id: String,
    pub item_id: String,
    pub verdict: String,
    pub referral_agency: String,
    pub decided_at: String,
}

fn err(status: StatusCode, detail: &str) -> axum::response::Response {
    (status, Json(serde_json::json!({ "detail": detail }))).into_response()
}

fn map_store(e: StoreError) -> axum::response::Response {
    match e {
        StoreError::DuplicateMediaItem => err(StatusCode::CONFLICT, "media item already exists"),
        StoreError::AdjudicationExists => err(StatusCode::CONFLICT, "media item already adjudicated"),
        StoreError::S44aConfirmationExists => err(StatusCode::CONFLICT, "s44a confirmation already exists"),
        StoreError::S44aNotAwaitingOfficer2 => err(StatusCode::CONFLICT, "s44a not awaiting officer2"),
        StoreError::Db(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "store unavailable"),
    }
}

fn parse_captured(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /speech/media-items — submit a media item for analysis.
pub async fn submit_media_item(
    State(store): State<Store>,
    Json(req): Json<MediaItemSubmission>,
) -> impl IntoResponse {
    info!(media_type = %req.media_type, "submit_media_item");
    let row = MediaItemRow {
        item_id: Uuid::now_v7(),
        url: req.url.clone(),
        url_sha256: hex::encode(sha2::Sha256::digest(req.url.as_bytes())),
        media_type: req.media_type,
        language_code: req.language_code,
        source_platform: req.source_platform,
        captured_at: parse_captured(&req.captured_at),
        status: "queued".to_string(),
        top_category: None,
        submitted_at: Utc::now(),
    };
    let item_id = row.item_id;
    match store.insert_media_item(&row).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(MediaItem {
                item_id: item_id.to_string(),
                status: "queued".to_string(),
                submitted_at: row.submitted_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => map_store(e),
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    pub status: Option<String>,
    pub media_type: Option<String>,
    pub from_date: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// GET /speech/media-items — list media items.
pub async fn list_media_items(
    State(store): State<Store>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    info!("list_media_items");
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(50).clamp(1, 200);
    let from = q
        .from_date
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    match store
        .list_media_items(
            q.status.as_deref(),
            q.media_type.as_deref(),
            from,
            page_size,
            (page - 1) * page_size,
        )
        .await
    {
        Ok((rows, total)) => {
            let items: Vec<MediaItemSummary> = rows
                .into_iter()
                .map(|r| MediaItemSummary {
                    item_id: r.item_id.to_string(),
                    media_type: r.media_type,
                    language_code: r.language_code,
                    source_platform: r.source_platform,
                    status: r.status,
                    top_category: r.top_category,
                    captured_at: r.captured_at.to_rfc3339(),
                    submitted_at: r.submitted_at.to_rfc3339(),
                })
                .collect();
            Json(serde_json::json!({ "items": items, "total": total, "page": page, "page_size": page_size }))
                .into_response()
        }
        Err(e) => map_store(e),
    }
}

/// GET /speech/media-items/{item_id} — fetch a media item with findings.
pub async fn get_media_item(
    State(store): State<Store>,
    Path(item_id): Path<Uuid>,
) -> impl IntoResponse {
    info!(%item_id, "get_media_item");
    let row = match store.fetch_media_item(item_id).await {
        Ok(Some(r)) => r,
        Ok(None) => return err(StatusCode::NOT_FOUND, "media item not found"),
        Err(e) => return map_store(e),
    };
    let findings: Vec<FindingRow> = store.fetch_findings(item_id).await.unwrap_or_default();
    let adjudication = store.fetch_adjudication(item_id).await.unwrap_or_default();
    Json(MediaItemDetail {
        item_id: row.item_id.to_string(),
        media_type: row.media_type,
        language_code: row.language_code,
        source_platform: row.source_platform,
        status: row.status.clone(),
        findings: findings
            .into_iter()
            .map(|f| AnalysisFinding {
                category: f.category,
                confidence: f.confidence,
                excerpt: f.excerpt,
            })
            .collect(),
        analyst_notes: adjudication.as_ref().and_then(|a| a.analyst_notes.clone()),
        decided_at: adjudication.as_ref().map(|a| a.decided_at.to_rfc3339()),
        verdict: adjudication.map(|a| a.verdict),
        submitted_at: row.submitted_at.to_rfc3339(),
        captured_at: row.captured_at.to_rfc3339(),
    })
    .into_response()
}

/// POST /speech/media-items/{item_id}/adjudications — record an adjudication.
pub async fn adjudicate_media_item(
    State(store): State<Store>,
    Path(item_id): Path<Uuid>,
    Json(req): Json<SpeechAdjudication>,
) -> impl IntoResponse {
    info!(%item_id, verdict = %req.verdict, "adjudicate_media_item");
    let row = AdjudicationRow {
        adjudication_id: Uuid::now_v7(),
        item_id,
        verdict: req.verdict,
        referral_agency: req.referral_agency,
        analyst_notes: req.analyst_notes,
        decided_at: Utc::now(),
    };
    let next_status = "adjudicated";
    let adjudication_id = row.adjudication_id;
    let verdict = row.verdict.clone();
    let referral_agency = row.referral_agency.clone();
    let decided_at = row.decided_at;
    match store.record_adjudication(&row, &req.grounds, next_status).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(AdjudicationResult {
                adjudication_id: adjudication_id.to_string(),
                item_id: item_id.to_string(),
                verdict,
                referral_agency,
                decided_at: decided_at.to_rfc3339(),
            }),
        )
            .into_response(),
        Err(e) => map_store(e),
    }
}

/// GET /speech/reports/summary — aggregate speech-integrity summary.
pub async fn get_speech_summary_report(
    State(store): State<Store>,
    Query(q): Query<SummaryQuery>,
) -> impl IntoResponse {
    info!("get_speech_summary_report");
    let from = q
        .from
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let to = q
        .to
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    match store.summary_counts(from, to).await {
        Ok(c) => Json(serde_json::json!({
            "period_from": from.map(|d| d.to_string()),
            "period_to": to.map(|d| d.to_string()),
            "total_items": c.total_items,
            "by_status": c.by_status,
            "by_category": c.by_category,
            "by_verdict": c.by_verdict,
            "referrals_by_agency": c.referrals_by_agency,
        }))
        .into_response(),
        Err(e) => map_store(e),
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct SummaryQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}
