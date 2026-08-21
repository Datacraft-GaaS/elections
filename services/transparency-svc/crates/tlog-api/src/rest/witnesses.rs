use axum::{extract::State, http::StatusCode, Json};
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
