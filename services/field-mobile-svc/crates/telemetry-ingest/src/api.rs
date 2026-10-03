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

//! Axum 0.8 REST router for telemetry ingest.
//!
//! Routes:
//!   POST   /v1/telemetry/ingest                    single signal
//!   POST   /v1/telemetry/ingest-batch              up to 1000 signals
//!   POST   /v1/telemetry/sessions                  open a session
//!   PATCH  /v1/telemetry/sessions/{id}/close       close a session
//!   POST   /v1/telemetry/devices                   register a device
//!   PATCH  /v1/telemetry/devices/{id}/revoke       revoke a device
//!   GET    /v1/telemetry/snapshots                 list snapshots (date + device_id)
//!   GET    /v1/telemetry/snapshots/{id}/inclusion-proof  fetch inclusion proof

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{ExitReason, SignalClass, TelemetryDevice, TelemetrySession};
use crate::error::TelemetryError;
use crate::ingest::{ingest_signal_pure, IngestRequest};
use crate::replay_defence::InMemoryNonceStore;
use crate::repository;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    /// Server signing key for counter-signatures (32 bytes).
    pub server_signing_key: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Error helper
// ---------------------------------------------------------------------------

fn err_response(status: StatusCode, msg: &str) -> impl IntoResponse {
    (status, Json(serde_json::json!({"error": msg})))
}

impl IntoResponse for TelemetryError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            TelemetryError::SvidRejected(_) | TelemetryError::KitCertInvalid { .. } => {
                StatusCode::UNAUTHORIZED
            }
            TelemetryError::HmacFailure { .. } | TelemetryError::InvalidHmacKeyRef(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            TelemetryError::ReplayDetected { .. } | TelemetryError::CounterOutOfOrder { .. } => {
                StatusCode::CONFLICT
            }
            TelemetryError::DeviceRevoked { .. } => StatusCode::FORBIDDEN,
            TelemetryError::DeviceNotFound { .. } | TelemetryError::SessionNotFound { .. } => {
                StatusCode::NOT_FOUND
            }
            TelemetryError::BatchTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            TelemetryError::Database(_) | TelemetryError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({"error": self.to_string()}))).into_response()
    }
}

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct IngestSignalDto {
    pub device_id: Uuid,
    pub session_id: Option<Uuid>,
    pub class: String,
    pub payload: serde_json::Value,
    pub occurred_at: chrono::DateTime<Utc>,
    pub monotonic_counter: i64,
    pub prev_hash: Option<String>,
    pub hmac_signature: String,
    pub nonce: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub actor_svid: String,
    pub device_secret_hex: String,
}

#[derive(Debug, Serialize)]
pub struct IngestSignalResponse {
    pub signal_id: String,
    pub this_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct IngestBatchDto {
    pub signals: Vec<IngestSignalDto>,
}

#[derive(Debug, Serialize)]
pub struct BatchItemResponse {
    pub index: usize,
    pub accepted: bool,
    pub signal_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IngestBatchResponse {
    pub accepted: usize,
    pub rejected: usize,
    pub items: Vec<BatchItemResponse>,
}

#[derive(Debug, Deserialize)]
pub struct OpenSessionDto {
    pub device_id: Uuid,
    pub actor_svid: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub entered_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CloseSessionDto {
    pub exit_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceDto {
    pub kit_id: String,
    pub app_id: String,
    pub persona_role: String,
    pub spiffe_id: Option<String>,
    pub hmac_key_hsm_ref: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub device_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RevokeDeviceDto {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotQuery {
    pub date: Option<chrono::NaiveDate>,
    pub device_id: Option<Uuid>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn ingest_single(
    State(state): State<AppState>,
    Json(dto): Json<IngestSignalDto>,
) -> Result<impl IntoResponse, TelemetryError> {
    let class = SignalClass::from_str(&dto.class)
        .ok_or_else(|| TelemetryError::Internal(format!("unknown signal class: {}", dto.class)))?;

    let prev_hash_bytes = dto
        .prev_hash
        .as_deref()
        .map(|s| hex::decode(s).map_err(|e| TelemetryError::Internal(e.to_string())))
        .transpose()?;

    let hmac_sig =
        hex::decode(&dto.hmac_signature).map_err(|e| TelemetryError::Internal(e.to_string()))?;
    let nonce = hex::decode(&dto.nonce).map_err(|e| TelemetryError::Internal(e.to_string()))?;
    let device_secret =
        hex::decode(&dto.device_secret_hex).map_err(|e| TelemetryError::Internal(e.to_string()))?;

    let trace_id = dto
        .trace_id
        .as_deref()
        .map(|s| hex::decode(s).map_err(|e| TelemetryError::Internal(e.to_string())))
        .transpose()?;
    let span_id = dto
        .span_id
        .as_deref()
        .map(|s| hex::decode(s).map_err(|e| TelemetryError::Internal(e.to_string())))
        .transpose()?;

    let (last_counter, last_hash) = repository::fetch_last_signal_state(&state.pool, dto.device_id)
        .await?
        .map(|(c, h)| (c, h))
        .unwrap_or((-1, vec![]));

    let req = IngestRequest {
        device_id: dto.device_id,
        session_id: dto.session_id,
        class,
        payload: dto.payload,
        occurred_at: dto.occurred_at,
        monotonic_counter: dto.monotonic_counter,
        prev_hash: prev_hash_bytes,
        hmac_signature: hmac_sig,
        nonce,
        trace_id,
        span_id,
        correlation_id: dto.correlation_id,
        causation_id: dto.causation_id,
        actor_svid: dto.actor_svid,
        device_secret,
        last_known_counter: last_counter,
        last_known_hash: last_hash,
    };

    let mut nonce_store = InMemoryNonceStore::new();
    let outcome = ingest_signal_pure(&req, &mut nonce_store, &state.server_signing_key)?;

    repository::insert_signal(&state.pool, &outcome.signal).await?;

    Ok((
        StatusCode::CREATED,
        Json(IngestSignalResponse {
            signal_id: outcome.signal.signal_id.to_string(),
            this_hash: hex::encode(&outcome.signal.this_hash),
        }),
    ))
}

async fn ingest_batch_handler(
    State(state): State<AppState>,
    Json(dto): Json<IngestBatchDto>,
) -> Result<impl IntoResponse, TelemetryError> {
    use crate::batch::{ingest_batch, BatchItemResult};

    let count = dto.signals.len();
    if count > 1000 {
        return Err(TelemetryError::BatchTooLarge { count });
    }

    let mut requests = Vec::with_capacity(count);
    for s in dto.signals {
        let class = SignalClass::from_str(&s.class)
            .ok_or_else(|| TelemetryError::Internal(format!("unknown class: {}", s.class)))?;
        let hmac_sig =
            hex::decode(&s.hmac_signature).map_err(|e| TelemetryError::Internal(e.to_string()))?;
        let nonce = hex::decode(&s.nonce).map_err(|e| TelemetryError::Internal(e.to_string()))?;
        let device_secret = hex::decode(&s.device_secret_hex)
            .map_err(|e| TelemetryError::Internal(e.to_string()))?;

        let (last_counter, last_hash) =
            repository::fetch_last_signal_state(&state.pool, s.device_id)
                .await?
                .unwrap_or((-1, vec![]));

        requests.push(IngestRequest {
            device_id: s.device_id,
            session_id: s.session_id,
            class,
            payload: s.payload,
            occurred_at: s.occurred_at,
            monotonic_counter: s.monotonic_counter,
            prev_hash: None,
            hmac_signature: hmac_sig,
            nonce,
            trace_id: None,
            span_id: None,
            correlation_id: s.correlation_id,
            causation_id: s.causation_id,
            actor_svid: s.actor_svid,
            device_secret,
            last_known_counter: last_counter,
            last_known_hash: last_hash,
        });
    }

    let mut nonce_store = InMemoryNonceStore::new();
    let result = ingest_batch(requests, &mut nonce_store, &state.server_signing_key)?;

    let mut items = Vec::with_capacity(result.items.len());
    for item in result.items {
        match item {
            BatchItemResult::Accepted(outcome) => {
                repository::insert_signal(&state.pool, &outcome.signal).await?;
                items.push(BatchItemResponse {
                    index: items.len(),
                    accepted: true,
                    signal_id: Some(outcome.signal.signal_id.to_string()),
                    reason: None,
                });
            }
            BatchItemResult::Rejected { index, reason } => {
                items.push(BatchItemResponse {
                    index,
                    accepted: false,
                    signal_id: None,
                    reason: Some(reason),
                });
            }
        }
    }

    Ok((
        StatusCode::MULTI_STATUS,
        Json(IngestBatchResponse {
            accepted: result.accepted,
            rejected: result.rejected,
            items,
        }),
    ))
}

async fn open_session(
    State(state): State<AppState>,
    Json(dto): Json<OpenSessionDto>,
) -> Result<impl IntoResponse, TelemetryError> {
    let now = Utc::now();
    let session = TelemetrySession {
        session_id: Uuid::now_v7(),
        device_id: dto.device_id,
        actor_svid: dto.actor_svid,
        entered_at: now,
        exited_at: None,
        exit_reason: None,
        signals_count: 0,
    };
    repository::insert_session(&state.pool, &session).await?;
    Ok((
        StatusCode::CREATED,
        Json(SessionResponse {
            session_id: session.session_id.to_string(),
            entered_at: session.entered_at.to_rfc3339(),
        }),
    ))
}

async fn close_session(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<CloseSessionDto>,
) -> Result<impl IntoResponse, TelemetryError> {
    let exit_reason = ExitReason::from_str(&dto.exit_reason).ok_or_else(|| {
        TelemetryError::Internal(format!("unknown exit reason: {}", dto.exit_reason))
    })?;
    repository::close_session(&state.pool, id, &exit_reason).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn register_device(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDeviceDto>,
) -> Result<impl IntoResponse, TelemetryError> {
    let now = Utc::now();
    let device = TelemetryDevice {
        device_id: Uuid::now_v7(),
        kit_id: dto.kit_id,
        app_id: dto.app_id,
        persona_role: dto.persona_role,
        spiffe_id: dto.spiffe_id,
        hmac_key_hsm_ref: dto.hmac_key_hsm_ref,
        first_seen_at: now,
        last_seen_at: now,
        revoked_at: None,
        revocation_reason: None,
    };
    repository::insert_device(&state.pool, &device).await?;
    Ok((
        StatusCode::CREATED,
        Json(DeviceResponse {
            device_id: device.device_id.to_string(),
        }),
    ))
}

async fn revoke_device(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<RevokeDeviceDto>,
) -> Result<impl IntoResponse, TelemetryError> {
    repository::revoke_device(&state.pool, id, &dto.reason).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_snapshots(
    State(_state): State<AppState>,
    Query(_q): Query<SnapshotQuery>,
) -> Result<impl IntoResponse, TelemetryError> {
    // Stub: returns empty list. Full implementation queries with date/device_id filters.
    Ok(Json(serde_json::json!({"snapshots": []})))
}

async fn get_inclusion_proof(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, TelemetryError> {
    let snap = repository::fetch_snapshot(&state.pool, id).await?;
    match snap {
        Some(s) => Ok(Json(serde_json::json!({
            "snapshot_id": s.snapshot_id.to_string(),
            "inclusion_proof": s.inclusion_proof.as_deref().map(hex::encode),
            "tlog_root_hash": s.tlog_root_hash.as_deref().map(hex::encode),
            "tlog_leaf_index": s.tlog_leaf_index,
        }))),
        None => Err(TelemetryError::Internal("snapshot not found".to_owned())),
    }
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/v1/telemetry/ingest", post(ingest_single))
        .route("/v1/telemetry/ingest-batch", post(ingest_batch_handler))
        .route("/v1/telemetry/sessions", post(open_session))
        .route("/v1/telemetry/sessions/{id}/close", patch(close_session))
        .route("/v1/telemetry/devices", post(register_device))
        .route("/v1/telemetry/devices/{id}/revoke", patch(revoke_device))
        .route("/v1/telemetry/snapshots", get(list_snapshots))
        .route(
            "/v1/telemetry/snapshots/{id}/inclusion-proof",
            get(get_inclusion_proof),
        )
        .with_state(state)
}
