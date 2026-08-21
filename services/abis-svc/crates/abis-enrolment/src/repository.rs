//! sqlx repository for enrolment jobs and decisions.
//! Runtime queries only - no compile-time macros - so no DATABASE_URL required.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{EnrolmentDecision, EnrolmentJob, EnrolmentStatus};
use crate::error::EnrolmentError;

/// Insert a new enrolment job. Returns `EnrolmentError::DuplicateJob` on unique constraint
/// violation (same subject_id + source + captured_at).
pub async fn insert_job(pool: &PgPool, job: &EnrolmentJob) -> Result<(), EnrolmentError> {
    let templates = serde_json::to_value(&job.templates)?;
    let source = job.source.as_str();
    let status = job.status.as_str();

    sqlx::query(
        r#"
        INSERT INTO eel_enrolment_jobs (
            id, subject_id, source, status, captured_at,
            captured_by_kit_id, captured_by_officer, templates
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (subject_id, source, captured_at) DO NOTHING
        "#,
    )
    .bind(job.id)
    .bind(job.subject_id)
    .bind(source)
    .bind(status)
    .bind(job.captured_at)
    .bind(job.captured_by_kit_id.as_deref())
    .bind(job.captured_by_officer.as_deref())
    .bind(&templates)
    .execute(pool)
    .await
    .map_err(EnrolmentError::Database)?;

    Ok(())
}

/// Update the status (and optionally quality_report) of a job.
pub async fn update_job_status(
    pool: &PgPool,
    job_id: Uuid,
    status: &EnrolmentStatus,
    quality_report: Option<&serde_json::Value>,
) -> Result<(), EnrolmentError> {
    sqlx::query(
        r#"
        UPDATE eel_enrolment_jobs
        SET status = $1,
            quality_report = COALESCE($2, quality_report),
            updated_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(status.as_str())
    .bind(quality_report)
    .bind(job_id)
    .execute(pool)
    .await
    .map_err(EnrolmentError::Database)?;

    Ok(())
}

/// Claim the next unclaimed job in a given status via SELECT FOR UPDATE SKIP LOCKED.
/// Returns None when no job is available.
pub async fn claim_next_job(
    pool: &PgPool,
    status: &EnrolmentStatus,
    worker_id: &str,
    lease_seconds: i64,
) -> Result<Option<JobRow>, EnrolmentError> {
    // Use a sub-query to atomically select and update the lease.
    let row = sqlx::query_as::<_, JobRow>(
        r#"
        UPDATE eel_enrolment_jobs
        SET worker_id = $1,
            lease_expires_at = NOW() + ($2 || ' seconds')::interval,
            updated_at = NOW()
        WHERE id = (
            SELECT id FROM eel_enrolment_jobs
            WHERE status = $3
              AND (lease_expires_at IS NULL OR lease_expires_at < NOW())
            ORDER BY created_at
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, subject_id, source, status, captured_at, created_at
        "#,
    )
    .bind(worker_id)
    .bind(lease_seconds.to_string())
    .bind(status.as_str())
    .fetch_optional(pool)
    .await
    .map_err(EnrolmentError::Database)?;

    Ok(row)
}

/// Insert a final enrolment decision.
pub async fn insert_decision(
    pool: &PgPool,
    decision: &EnrolmentDecision,
) -> Result<(), EnrolmentError> {
    let previous = serde_json::to_value(&decision.previous_records
        .iter()
        .map(|u| u.to_string())
        .collect::<Vec<_>>())?;

    sqlx::query(
        r#"
        INSERT INTO eel_enrolment_decisions (
            id, enrolment_job_id, subject_id, enrolled_at, gallery_shard_id,
            previous_records, merge_from, adjudicator_id
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        "#,
    )
    .bind(decision.id)
    .bind(decision.enrolment_job_id)
    .bind(decision.subject_id)
    .bind(decision.enrolled_at)
    .bind(&decision.gallery_shard_id)
    .bind(&previous)
    .bind(decision.merge_from)
    .bind(decision.adjudicator_id)
    .execute(pool)
    .await
    .map_err(EnrolmentError::Database)?;

    Ok(())
}

/// Minimal row projection for orchestrator polling.
#[derive(Debug, sqlx::FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub source: String,
    pub status: String,
    pub captured_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
}
