//! Axum handlers for witness cosign API.

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct WitnessState {
    pub pool: Arc<PgPool>,
}

/// Marker type exported for use in lib.rs re-exports.
pub struct WitnessHandler;

#[derive(Debug, Deserialize)]
pub struct CosignRequest {
    pub sth_id: Uuid,
    pub witness_id: String,
    pub witness_spiffe_id: Option<String>,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct CosignResponse {
    pub id: Uuid,
    pub accepted: bool,
}

pub async fn post_cosign(
    State(state): State<WitnessState>,
    Json(req): Json<CosignRequest>,
) -> Result<Json<CosignResponse>, StatusCode> {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO witness_cosigs (id, sth_id, witness_id, witness_spiffe_id, signature, cosigned_at, is_internal)
        VALUES ($1, $2, $3, $4, decode($5, 'hex'), now(), false)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(req.sth_id)
    .bind(&req.witness_id)
    .bind(req.witness_spiffe_id.as_deref().unwrap_or(""))
    .bind(&req.signature)
    .execute(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CosignResponse { id, accepted: true }))
}
