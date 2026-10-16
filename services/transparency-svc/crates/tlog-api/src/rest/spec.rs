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

//! Public contract handlers (transparency/openapi.yaml log surface).

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::state::ApiState;

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub leaf_index: i64,
    pub leaf_hash: String,
    pub timestamp: String,
    pub entry_type: String,
    pub data_hash: String,
    pub submitter_id: String,
    pub extra_data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct LeafAppendRequest {
    pub entry_type: String,
    pub data_hash: String,
    pub submitter_id: String,
    pub submitter_sig: String,
    pub hlc: String,
    pub extra_data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct LeafAppendResult {
    pub leaf_index: u64,
    pub leaf_hash: String,
    pub queued_at: String,
    pub estimated_inclusion_s: u64,
}

/// A row of the `leaves` table, in the order get_log_entries selects it.
type LogEntryRow = (
    i64,
    String,
    Vec<u8>,
    String,
    String,
    chrono::DateTime<chrono::Utc>,
);

/// GET /api/v1/log/entries — list recent log entries.
pub async fn get_log_entries(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rows: Vec<LogEntryRow> = sqlx::query_as(
        "SELECT index, domain, hash, producer_spiffe_id, source_event_id, event_time \
             FROM leaves ORDER BY index DESC LIMIT 1000",
    )
    .fetch_all(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entries: Vec<LogEntry> = rows
        .into_iter()
        .map(
            |(idx, domain, hash, producer, event_id, event_time)| LogEntry {
                leaf_index: idx,
                leaf_hash: hex::encode(hash),
                timestamp: event_time.to_rfc3339(),
                entry_type: domain,
                data_hash: event_id,
                submitter_id: producer,
                extra_data: serde_json::Value::Null,
            },
        )
        .collect();

    let tree_size: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leaves")
        .fetch_one(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let start = entries.last().map(|e| e.leaf_index).unwrap_or(0);
    let end = entries.first().map(|e| e.leaf_index).unwrap_or(0);
    Ok(Json(serde_json::json!({
        "entries": entries,
        "start": start,
        "end": end,
        "tree_size": tree_size,
    })))
}

/// POST /api/v1/log/leaves — append a leaf to the log.
pub async fn append_leaf(
    State(state): State<ApiState>,
    Json(req): Json<LeafAppendRequest>,
) -> Result<Json<LeafAppendResult>, StatusCode> {
    let domain = match req.entry_type.as_str() {
        "voter-events" => tlog_domain::Domain::VoterEvents,
        "results-events" => tlog_domain::Domain::ResultsEvents,
        "custody-events" => tlog_domain::Domain::CustodyEvents,
        "credential-events" => tlog_domain::Domain::CredentialEvents,
        _ => tlog_domain::Domain::MetaEvents,
    };

    let canonical = serde_json::to_vec(&req.extra_data).unwrap_or_default();
    let sig = hex::decode(&req.submitter_sig).unwrap_or_default();

    let result = state
        .sequencer
        .append(
            domain,
            &canonical,
            &req.submitter_id,
            &req.hlc,
            "spec-append",
            chrono::Utc::now(),
            &sig,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Some(r) => Ok(Json(LeafAppendResult {
            leaf_index: r.leaf_index,
            leaf_hash: r.proof.root_hash.clone(),
            queued_at: chrono::Utc::now().to_rfc3339(),
            estimated_inclusion_s: 1,
        })),
        None => Err(StatusCode::TOO_MANY_REQUESTS),
    }
}
