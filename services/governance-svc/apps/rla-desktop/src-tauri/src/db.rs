use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

pub async fn init_db(path: &str) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(path)?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sampled_ballots (
            ballot_id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            station_code TEXT NOT NULL,
            batch_id TEXT NOT NULL,
            ballot_position INTEGER NOT NULL,
            auditor_interpretation TEXT,
            discrepancy INTEGER NOT NULL DEFAULT 0,
            verdict TEXT,
            recorded_at TEXT,
            synced INTEGER NOT NULL DEFAULT 0
        )"
    ).execute(&pool).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_runs (
            run_id TEXT PRIMARY KEY,
            election_id TEXT NOT NULL,
            position_code TEXT NOT NULL,
            risk_limit REAL NOT NULL,
            seed TEXT,
            status TEXT NOT NULL DEFAULT 'initialised',
            created_at TEXT NOT NULL
        )"
    ).execute(&pool).await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sync_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL,
            sent INTEGER NOT NULL DEFAULT 0
        )"
    ).execute(&pool).await?;
    Ok(pool)
}
