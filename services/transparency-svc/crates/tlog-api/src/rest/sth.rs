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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use tlog_domain::Domain;

use crate::state::ApiState;

#[derive(Serialize)]
pub struct SthResponse {
    pub id: String,
    pub tree_size: i64,
    pub root_hash: String,
    pub cadence: String,
    pub signed_at: String,
    pub signature: String,
}

/// A row of the `sth` table: id, tree_size, root_hash, cadence, signed_at, signature.
type SthRow = (String, i64, Vec<u8>, String, chrono::DateTime<chrono::Utc>, Vec<u8>);

pub async fn get_latest_sth(
    State(state): State<ApiState>,
) -> Result<Json<SthResponse>, StatusCode> {
    let row: Option<SthRow> = sqlx::query_as(
        "SELECT id::text, tree_size, root_hash, cadence, signed_at, signature \
             FROM sth ORDER BY tree_size DESC LIMIT 1",
    )
    .fetch_optional(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        None => Err(StatusCode::NOT_FOUND),
        Some((id, tree_size, root_hash, cadence, signed_at, sig)) => Ok(Json(SthResponse {
            id,
            tree_size,
            root_hash: hex::encode(root_hash),
            cadence,
            signed_at: signed_at.to_rfc3339(),
            signature: hex::encode(sig),
        })),
    }
}

pub async fn get_sth_by_size(
    State(state): State<ApiState>,
    Path(tree_size): Path<i64>,
) -> Result<Json<SthResponse>, StatusCode> {
    let row: Option<SthRow> = sqlx::query_as(
        "SELECT id::text, tree_size, root_hash, cadence, signed_at, signature \
             FROM sth WHERE tree_size = $1 LIMIT 1",
    )
    .bind(tree_size)
    .fetch_optional(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        None => Err(StatusCode::NOT_FOUND),
        Some((id, ts, root_hash, cadence, signed_at, sig)) => Ok(Json(SthResponse {
            id,
            tree_size: ts,
            root_hash: hex::encode(root_hash),
            cadence,
            signed_at: signed_at.to_rfc3339(),
            signature: hex::encode(sig),
        })),
    }
}

#[derive(Serialize)]
pub struct DomainInfo {
    pub name: String,
    pub description: String,
}

pub async fn list_domains() -> Json<Vec<DomainInfo>> {
    let domains = [
        Domain::VoterEvents,
        Domain::ResultsEvents,
        Domain::CustodyEvents,
        Domain::CredentialEvents,
        Domain::MetaEvents,
    ];
    Json(
        domains
            .into_iter()
            .map(|d| DomainInfo {
                name: d.as_str().to_string(),
                description: format!("{} domain", d),
            })
            .collect(),
    )
}
