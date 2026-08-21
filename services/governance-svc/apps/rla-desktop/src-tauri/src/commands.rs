use crate::error::RlaError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotVerdict {
    pub ballot_id: String,
    pub run_id: String,
    pub auditor_interpretation: String,
    pub discrepancy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SampledBallot {
    pub ballot_id: String,
    pub run_id: String,
    pub station_code: String,
    pub batch_id: String,
    pub ballot_position: i64,
    pub auditor_interpretation: Option<String>,
    pub discrepancy: bool,
    pub synced: bool,
}

#[tauri::command]
pub async fn list_ballots(
    run_id: String,
    db: State<'_, SqlitePool>,
) -> Result<Vec<SampledBallot>, RlaError> {
    let rows = sqlx::query_as::<_, SampledBallot>(
        "SELECT ballot_id, run_id, station_code, batch_id, ballot_position,
                auditor_interpretation,
                CAST(discrepancy AS BOOLEAN) AS discrepancy,
                CAST(synced AS BOOLEAN) AS synced
         FROM sampled_ballots WHERE run_id = ?"
    )
    .bind(&run_id)
    .fetch_all(db.inner())
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn record_verdict(
    verdict: BallotVerdict,
    db: State<'_, SqlitePool>,
) -> Result<(), RlaError> {
    let now = chrono::Utc::now().to_rfc3339();
    let discrepancy = verdict.discrepancy as i64;
    sqlx::query(
        "UPDATE sampled_ballots SET auditor_interpretation = ?, discrepancy = ?, verdict = 'recorded', recorded_at = ?, synced = 0 WHERE ballot_id = ? AND run_id = ?"
    )
    .bind(&verdict.auditor_interpretation)
    .bind(discrepancy)
    .bind(&now)
    .bind(&verdict.ballot_id)
    .bind(&verdict.run_id)
    .execute(db.inner())
    .await?;
    let payload = serde_json::to_string(&verdict)?;
    sqlx::query("INSERT INTO sync_queue (payload, created_at) VALUES (?, ?)")
        .bind(&payload)
        .bind(&now)
        .execute(db.inner())
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn pending_sync_count(db: State<'_, SqlitePool>) -> Result<i64, RlaError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sync_queue WHERE sent = 0")
        .fetch_one(db.inner())
        .await?;
    Ok(row.0)
}

#[tauri::command]
pub async fn mark_synced(id: i64, db: State<'_, SqlitePool>) -> Result<(), RlaError> {
    sqlx::query("UPDATE sync_queue SET sent = 1 WHERE id = ?")
        .bind(id)
        .execute(db.inner())
        .await?;
    Ok(())
}
