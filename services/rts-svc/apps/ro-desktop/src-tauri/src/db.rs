//! Local SQLite WAL cache for the RO desktop (EC-OPS-05 crash-safety).
//!
//! Every action (verify, amend, compile 34B/34C, dispute) is written to the
//! `local_queue` table *before* it is sent to the backend. On power restoration
//! the app resumes uncommitted actions without re-emitting completed ones,
//! preventing duplicate 34B (EC-OPS-05).
//!
//! WAL mode is set at connection time. The journal-before-act pattern:
//!   1. INSERT into local_queue with status = 'pending'
//!   2. Execute backend call
//!   3. UPDATE status = 'committed' inside the same SQLite transaction as any
//!      local state mutation
//!
//! On resume, rows with status = 'pending' are resubmitted. The backend's
//! Idempotency-Key prevents re-execution on the server side.
//!
//! Uses runtime `sqlx::query` (not the compile-time macro) to avoid requiring
//! DATABASE_URL at build time. The schema is validated by integration tests.

use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

/// Initialise (or reopen) the WAL-mode SQLite database at `db_path`.
pub async fn open_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::query(CREATE_SCHEMA).execute(&pool).await?;
    Ok(pool)
}

const CREATE_SCHEMA: &str = "
    CREATE TABLE IF NOT EXISTS local_queue (
        id               TEXT PRIMARY KEY,
        action_type      TEXT NOT NULL,
        payload          TEXT NOT NULL,
        idempotency_key  TEXT NOT NULL UNIQUE,
        status           TEXT NOT NULL DEFAULT 'pending',
        created_at       TEXT NOT NULL,
        committed_at     TEXT
    );
    CREATE TABLE IF NOT EXISTS form34b_log (
        id               TEXT PRIMARY KEY,
        constituency_code TEXT NOT NULL,
        election_id      TEXT NOT NULL,
        position_code    TEXT NOT NULL,
        compiled_at      TEXT NOT NULL,
        status           TEXT NOT NULL DEFAULT 'compiled'
    );
";

/// Enqueue a pending action. Returns the row id (UUIDv7 string).
pub async fn enqueue_action(
    pool: &SqlitePool,
    action_type: &str,
    payload: &serde_json::Value,
    idempotency_key: Uuid,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    let payload_str = payload.to_string();
    let idem = idempotency_key.to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO local_queue (id, action_type, payload, idempotency_key, status, created_at) \
         VALUES (?, ?, ?, ?, 'pending', ?)",
    )
    .bind(&id)
    .bind(action_type)
    .bind(&payload_str)
    .bind(&idem)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(id)
}

/// Mark an enqueued action as committed (backend confirmed).
pub async fn commit_action(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE local_queue SET status = 'committed', committed_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// List all pending (uncommitted) actions for resume after power restoration.
pub async fn pending_actions(pool: &SqlitePool) -> Result<Vec<PendingAction>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, action_type, payload, idempotency_key FROM local_queue \
         WHERE status = 'pending' ORDER BY created_at",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PendingAction {
            id: r.get("id"),
            action_type: r.get("action_type"),
            payload: serde_json::from_str(r.get::<&str, _>("payload")).unwrap_or_default(),
            idempotency_key: r.get("idempotency_key"),
        })
        .collect())
}

/// Record a compiled Form 34B; returns Err if the constituency+position already
/// has a compiled 34B (duplicate detection — EC-OPS-05).
pub async fn record_form34b(
    pool: &SqlitePool,
    constituency_code: &str,
    election_id: &str,
    position_code: &str,
) -> Result<String, crate::error::AppError> {
    // Check for existing compiled 34B for this constituency+position.
    let existing = sqlx::query(
        "SELECT id FROM form34b_log \
         WHERE constituency_code = ? AND election_id = ? AND position_code = ? AND status = 'compiled'",
    )
    .bind(constituency_code)
    .bind(election_id)
    .bind(position_code)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Err(crate::error::AppError::Duplicate34B(
            constituency_code.to_string(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO form34b_log (id, constituency_code, election_id, position_code, compiled_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(constituency_code)
    .bind(election_id)
    .bind(position_code)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(id)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PendingAction {
    pub id: String,
    pub action_type: String,
    pub payload: serde_json::Value,
    pub idempotency_key: String,
}
