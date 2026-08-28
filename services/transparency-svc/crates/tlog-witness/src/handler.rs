//! Axum handlers for witness cosign API.
//!
//! Every observed head -- whether submitted for cosigning here or gossiped
//! from a configured peer -- flows through [`SplitViewDetector`]; any two
//! different roots at the same tree size are retained as findings and
//! surfaced on `GET .../split-views` so equivocation cannot go unnoticed.

use std::sync::{Arc, Mutex};

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct WitnessState {
    pub pool: Arc<PgPool>,
    pub detector: Arc<Mutex<crate::split_view::SplitViewDetector>>,
}

impl WitnessState {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            detector: Arc::new(Mutex::new(crate::split_view::SplitViewDetector::new())),
        }
    }
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

    // Feed the head this witness is vouching for into split-view detection.
    if let Ok(Some((tree_size, root_hash))) = sqlx::query_as::<_, (i64, String)>(
        "SELECT tree_size, encode(root_hash, 'hex') FROM sth WHERE id = $1",
    )
    .bind(req.sth_id)
    .fetch_optional(&*state.pool)
    .await
    {
        let mut detector = state.detector.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        detector.observe_root(&req.witness_id, tree_size as u64, &root_hash);
    }

    Ok(Json(CosignResponse { id, accepted: true }))
}

/// The head this witness currently sees (proxied from tlog-api). Peers poll
/// this to compare views.
pub async fn get_head(
    State(state): State<WitnessState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let row: Option<(i64, String, String)> = sqlx::query_as(
        "SELECT tree_size, encode(root_hash, 'hex'), cadence FROM sth \
         ORDER BY tree_size DESC, signed_at DESC LIMIT 1",
    )
    .fetch_optional(&*state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((tree_size, root_hash, cadence)) => Ok(Json(serde_json::json!({
            "tree_size": tree_size,
            "root_hash": root_hash,
            "cadence": cadence,
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Serialize)]
pub struct SplitViewFinding {
    pub tree_size: u64,
    pub reported_by: String,
    pub conflicting_root: String,
    pub detected_at: String,
}

/// All split-view findings recorded by this witness.
pub async fn get_split_views(State(state): State<WitnessState>) -> Json<serde_json::Value> {
    let detector = state.detector.lock().expect("detector lock");
    let findings: Vec<SplitViewFinding> = detector
        .findings
        .iter()
        .map(|f| SplitViewFinding {
            tree_size: f.tree_size,
            reported_by: f.reported_by.clone(),
            conflicting_root: f.conflicting_root.clone(),
            detected_at: f.detected_at.to_rfc3339(),
        })
        .collect();
    Json(serde_json::json!({ "findings": findings }))
}
