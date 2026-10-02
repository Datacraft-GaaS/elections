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

//! Tauri IPC commands: web UI to Rust core (WP-3.2 M09).
//!
//! Each command follows the journal-before-act pattern from db.rs:
//!   1. Enqueue action in SQLite WAL (crash-safe)
//!   2. Call the rts-ro-api backend
//!   3. Commit the local queue row
//!
//! On power-cut between steps 2 and 3, the action is replayed on resume; the
//! server deduplicates via the Idempotency-Key.

use tauri::State;
use uuid::Uuid;

use crate::db::{commit_action, enqueue_action, pending_actions, record_form34b, PendingAction};
use crate::error::AppError;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub backend_url: String,
}

/// List pending 34As awaiting review (fetched from backend; cached locally).
#[tauri::command]
pub async fn get_queue(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    // In production: GET {backend_url}/api/v1/rts/ro/queue with JWT.
    // Stub returns cached local pending count.
    let pending = pending_actions(&state.db).await?;
    Ok(serde_json::json!({
        "pending_local": pending.len(),
        "items": []
    }))
}

/// Mark a Form 34A reviewed and countersigned by the RO.
#[tauri::command]
pub async fn review_form34a(
    state: State<'_, AppState>,
    form_id: String,
    ro_signature: String,
) -> Result<serde_json::Value, AppError> {
    let idempotency_key = Uuid::new_v4();
    let payload = serde_json::json!({
        "form_id": form_id,
        "ro_signature": ro_signature,
        "idempotency_key": idempotency_key,
    });
    let row_id = enqueue_action(&state.db, "verify_form34a", &payload, idempotency_key).await?;

    // POST to backend (stub: assume success).
    let event_id = Uuid::new_v4();

    commit_action(&state.db, &row_id).await?;

    Ok(serde_json::json!({
        "form_id": form_id,
        "event_id": event_id,
        "status": "verified"
    }))
}

/// Compile Form 34B for a constituency; guards against duplicate on resume.
#[tauri::command]
pub async fn compile_34b(
    state: State<'_, AppState>,
    constituency_code: String,
    election_id: String,
    position_code: String,
    ro_signature: String,
    witness_signature: String,
) -> Result<serde_json::Value, AppError> {
    let idempotency_key = Uuid::new_v4();
    let payload = serde_json::json!({
        "constituency_code": constituency_code,
        "election_id": election_id,
        "position_code": position_code,
        "idempotency_key": idempotency_key,
    });
    let row_id = enqueue_action(&state.db, "compile_form34b", &payload, idempotency_key).await?;

    // Guard: record in form34b_log; returns Duplicate34B if already compiled.
    let _log_id = record_form34b(
        &state.db,
        &constituency_code,
        &election_id,
        &position_code,
    )
    .await?;

    // POST to backend (stub: assume success).
    let form34b_id = Uuid::new_v4();

    commit_action(&state.db, &row_id).await?;

    Ok(serde_json::json!({
        "form34b_id": form34b_id,
        "status": "compiled",
        "ro_signature": ro_signature,
        "witness_signature": witness_signature,
    }))
}

/// Raise a dispute case.
#[tauri::command]
pub async fn create_dispute(
    state: State<'_, AppState>,
    form_id: String,
    reason: String,
) -> Result<serde_json::Value, AppError> {
    let idempotency_key = Uuid::new_v4();
    let payload = serde_json::json!({
        "form_id": form_id,
        "reason": reason,
        "idempotency_key": idempotency_key,
    });
    let row_id = enqueue_action(&state.db, "create_dispute", &payload, idempotency_key).await?;
    let dispute_id = Uuid::new_v4();
    commit_action(&state.db, &row_id).await?;
    Ok(serde_json::json!({ "dispute_id": dispute_id, "status": "open" }))
}

/// Start or stop session recording; anchor hash every 60 s.
#[tauri::command]
pub async fn session_recording(
    state: State<'_, AppState>,
    action: String,
    segment_hash: Option<String>,
) -> Result<serde_json::Value, AppError> {
    let idempotency_key = Uuid::new_v4();
    let payload = serde_json::json!({
        "action": action,
        "segment_hash": segment_hash,
        "idempotency_key": idempotency_key,
    });
    let row_id =
        enqueue_action(&state.db, "session_recording", &payload, idempotency_key).await?;
    commit_action(&state.db, &row_id).await?;
    Ok(serde_json::json!({ "session_id": Uuid::new_v4(), "status": action }))
}

/// Return all pending local actions (for resume UI indicator).
#[tauri::command]
pub async fn get_pending_actions(
    state: State<'_, AppState>,
) -> Result<Vec<PendingAction>, AppError> {
    Ok(pending_actions(&state.db).await?)
}
