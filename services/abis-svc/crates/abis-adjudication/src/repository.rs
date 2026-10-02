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

//! sqlx repository for adjudication cases and reviewer actions.
//! All queries use runtime form (no compile-time macros) to avoid needing
//! a live DATABASE_URL during compilation.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{AdjudicationCase, CaseState};
use crate::error::AdjudicationError;

/// Insert a new adjudication case into the database.
pub async fn insert_case(pool: &PgPool, case: &AdjudicationCase) -> Result<(), AdjudicationError> {
    let priority = format!("{:?}", case.priority);
    let tier = format!("{:?}", case.tier);
    let state = case_state_str(&case.state);
    let enrolment_job_id = case.enrolment_job_id;
    let probable_ids: Vec<String> =
        case.probable_match_subject_ids.iter().map(|u| u.to_string()).collect();
    let probable_json = serde_json::to_value(&probable_ids).unwrap_or(serde_json::Value::Array(vec![]));

    sqlx::query(
        r#"
        INSERT INTO eel_adjudication_cases_v2 (
            id, opened_at, priority, tier, state, enrolment_job_id,
            probable_match_subject_ids, evidence_bundle_ref, sla_deadline_at,
            created_at, updated_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,NOW(),NOW())
        "#,
    )
    .bind(case.id)
    .bind(case.opened_at)
    .bind(&priority)
    .bind(&tier)
    .bind(&state)
    .bind(enrolment_job_id)
    .bind(&probable_json)
    .bind(case.evidence_bundle_ref)
    .bind(case.sla_deadline_at)
    .execute(pool)
    .await
    .map_err(AdjudicationError::Database)?;

    Ok(())
}

/// Update the state, assigned_to, and updated_at of a case.
pub async fn update_case_state(
    pool: &PgPool,
    case_id: Uuid,
    new_state: &CaseState,
    assigned_to: Option<Uuid>,
) -> Result<(), AdjudicationError> {
    let state = case_state_str(new_state);
    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE eel_adjudication_cases_v2
        SET state = $1, assigned_to = $2, assigned_at = CASE WHEN $2 IS NOT NULL THEN $3 ELSE assigned_at END,
            updated_at = $3
        WHERE id = $4
        "#,
    )
    .bind(&state)
    .bind(assigned_to)
    .bind(now)
    .bind(case_id)
    .execute(pool)
    .await
    .map_err(AdjudicationError::Database)?;

    Ok(())
}

/// Fetch cases ordered by priority (ElectionDayHold first) and SLA deadline.
pub async fn fetch_open_cases(pool: &PgPool, limit: i64) -> Result<Vec<CaseRow>, AdjudicationError> {
    let rows = sqlx::query_as::<_, CaseRow>(
        r#"
        SELECT id, state, priority, tier, assigned_to, sla_deadline_at, opened_at
        FROM eel_adjudication_cases_v2
        WHERE state NOT IN ('Decided','Withdrawn')
        ORDER BY
            CASE priority
                WHEN 'ElectionDayHold' THEN 0
                WHEN 'Urgent' THEN 1
                ELSE 2
            END ASC,
            sla_deadline_at ASC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AdjudicationError::Database)?;

    Ok(rows)
}

/// Minimal row projection for list views.
#[derive(Debug, sqlx::FromRow)]
pub struct CaseRow {
    pub id: Uuid,
    pub state: String,
    pub priority: String,
    pub tier: String,
    pub assigned_to: Option<Uuid>,
    pub sla_deadline_at: chrono::DateTime<Utc>,
    pub opened_at: chrono::DateTime<Utc>,
}

fn case_state_str(s: &CaseState) -> &'static str {
    match s {
        CaseState::Open => "Open",
        CaseState::Assigned => "Assigned",
        CaseState::InReview => "InReview",
        CaseState::Decided => "Decided",
        CaseState::Escalated => "Escalated",
        CaseState::Withdrawn => "Withdrawn",
        CaseState::Reopened => "Reopened",
    }
}
