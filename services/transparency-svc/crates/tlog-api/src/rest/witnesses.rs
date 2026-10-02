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

use axum::{extract::{Query, State}, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::ApiState;

#[derive(Serialize)]
pub struct WitnessInfo {
    pub witness_id: String,
    pub witness_spiffe_id: String,
}

pub async fn list_witnesses(
    State(state): State<ApiState>,
) -> Result<Json<Vec<WitnessInfo>>, StatusCode> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT witness_id, witness_spiffe_id FROM witnesses",
    )
    .fetch_all(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|(wid, spiffe)| WitnessInfo {
                witness_id: wid,
                witness_spiffe_id: spiffe,
            })
            .collect(),
    ))
}

/// One STH in a cross-check report.
#[derive(Serialize)]
pub struct SthRow {
    pub id: Uuid,
    pub tree_size: i64,
    /// Root hash, hex encoded.
    pub root_hash: String,
    pub signed_at: DateTime<Utc>,
}

/// One witness cosignature in a cross-check report.
#[derive(Serialize)]
pub struct CosigRow {
    pub sth_id: Uuid,
    pub witness_id: String,
    /// Ed25519 signature over `concat(tree_size_le64, root_hash)`, hex encoded.
    pub signature: String,
}

/// A registered witness with its verification key.
#[derive(Serialize)]
pub struct WitnessKeyRow {
    pub witness_id: String,
    /// Raw Ed25519 verifying key (32 bytes), hex encoded.
    pub public_key_hex: String,
}

/// Everything `tlog-admin witness-cross-check` needs to verify cosignatures
/// and detect split views without direct database access.
#[derive(Serialize)]
pub struct WitnessReport {
    pub sths: Vec<SthRow>,
    pub cosignatures: Vec<CosigRow>,
    pub witnesses: Vec<WitnessKeyRow>,
}

#[derive(Deserialize)]
pub struct ReportQuery {
    limit: Option<i64>,
}

pub async fn witness_report(
    State(state): State<ApiState>,
    Query(q): Query<ReportQuery>,
) -> Result<Json<WitnessReport>, StatusCode> {
    let limit = q.limit.unwrap_or(50).clamp(1, 1000);

    let sths: Vec<(Uuid, i64, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, tree_size, encode(root_hash, 'hex'), signed_at \
         FROM sth ORDER BY tree_size DESC, signed_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sth_ids: Vec<Uuid> = sths.iter().map(|(id, ..)| *id).collect();
    let cosigs: Vec<(Uuid, String, String)> = if sth_ids.is_empty() {
        Vec::new()
    } else {
        sqlx::query_as(
            "SELECT sth_id, witness_id, encode(signature, 'hex') \
             FROM witness_cosigs WHERE sth_id = ANY($1)",
        )
        .bind(&sth_ids)
        .fetch_all(&*state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let witnesses: Vec<(String, String)> = sqlx::query_as(
        "SELECT witness_id, public_key_hex FROM witnesses",
    )
    .fetch_all(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(WitnessReport {
        sths: sths
            .into_iter()
            .map(|(id, tree_size, root_hash, signed_at)| SthRow {
                id,
                tree_size,
                root_hash,
                signed_at,
            })
            .collect(),
        cosignatures: cosigs
            .into_iter()
            .map(|(sth_id, witness_id, signature)| CosigRow {
                sth_id,
                witness_id,
                signature,
            })
            .collect(),
        witnesses: witnesses
            .into_iter()
            .map(|(witness_id, public_key_hex)| WitnessKeyRow {
                witness_id,
                public_key_hex,
            })
            .collect(),
    }))
}

#[derive(Deserialize)]
pub struct CosignRequest {
    pub sth_id: Uuid,
    pub witness_id: String,
    pub signature: String,
}

#[derive(Serialize)]
pub struct CosignResponse {
    pub id: Uuid,
    pub accepted: bool,
}

pub async fn post_cosign(
    State(state): State<ApiState>,
    Json(req): Json<CosignRequest>,
) -> Result<Json<CosignResponse>, StatusCode> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO witness_cosigs \
         (id, sth_id, witness_id, witness_spiffe_id, signature, cosigned_at, is_internal) \
         VALUES ($1, $2, $3, '', decode($4,'hex'), now(), false) \
         ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(req.sth_id)
    .bind(&req.witness_id)
    .bind(&req.signature)
    .execute(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(CosignResponse { id, accepted: true }))
}
