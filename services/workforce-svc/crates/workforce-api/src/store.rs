//! Persistence for workforce-svc.
//!
//! [`Store`] is an enum over the two backends so handlers stay dyn-free:
//! `Pg` runs sqlx runtime queries against the workforce schema
//! (`services/workforce-svc/migrations/V1__workforce.sql`); `Mem` is an
//! in-process fake used by hermetic tests and mirrors exactly the same
//! state transitions, preferred-contact uniqueness and open-claim semantics.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::*;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("entity not found")]
    NotFound,
    #[error("an unreturned equipment claim already exists for this posting")]
    OpenClaimExists,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Outbox(#[from] eel_outbox_rs::OutboxError),
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum Store {
    Pg(PgPool),
    Mem(MemStore),
}

const WORKER_COLS: &str =
    "worker_id, legal_name, national_id_hash, home_county, role_eligibility, \
     payroll_contract_ref, next_of_kin_name, next_of_kin_phone_hash, created_at";
type WorkerTuple = (
    Uuid,
    String,
    String,
    String,
    serde_json::Value,
    Option<String>,
    Option<String>,
    Option<String>,
    DateTime<Utc>,
);

const CONTACT_COLS: &str =
    "contact_id, worker_id, channel, value_hash_hex, verified, consented, preferred, changed_at";
type ContactTuple = (Uuid, Uuid, String, String, bool, bool, bool, DateTime<Utc>);

const CASE_COLS: &str =
    "case_id, application_ref, applicant_name, national_id_hash, role_code, county, \
     stage, reject_reason, waitlist_rank, composite_score, created_at";
type CaseTuple = (
    Uuid,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<i64>,
    Option<f32>,
    DateTime<Utc>,
);

const HISTORY_COLS: &str = "from_stage, to_stage, actor, at, reason";

type LogTuple = (Uuid, String, DateTime<Utc>, String, String, Option<String>);

const POSTING_COLS: &str =
    "posting_id, worker_id, station_code, role_code, effective_from, effective_to, \
     posted_by, superseded_by, status, created_at";
type PostingTuple = (
    Uuid,
    Uuid,
    String,
    String,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
    String,
    Option<Uuid>,
    String,
    DateTime<Utc>,
);

const CLAIM_COLS: &str =
    "claim_id, posting_id, kit_serials, acknowledged, returned, loss_flags, created_at";
type ClaimTuple = (
    Uuid,
    Uuid,
    serde_json::Value,
    bool,
    bool,
    serde_json::Value,
    DateTime<Utc>,
);

const TEMPLATE_COLS: &str =
    "template_id, role_code, phase, title, critical, training_content_id, created_at";
type TemplateTuple = (
    Uuid,
    String,
    String,
    String,
    bool,
    Option<String>,
    DateTime<Utc>,
);

const ASSIGNMENT_COLS: &str = "assignment_id, posting_id, template_id, due_from, due_to, status, \
     evidence_hash, block_reason, updated_at";
type AssignmentTuple = (
    Uuid,
    Uuid,
    Uuid,
    DateTime<Utc>,
    DateTime<Utc>,
    String,
    Option<String>,
    Option<String>,
    DateTime<Utc>,
);

const PERF_COLS: &str = "record_id, worker_id, category, score, detail, recorded_by, recorded_at";
type PerfTuple = (
    Uuid,
    Uuid,
    String,
    Option<f32>,
    String,
    String,
    DateTime<Utc>,
);

type StageChangeTuple = (Uuid, String, String, String, DateTime<Utc>, Option<String>);

const INTERVIEW_COLS: &str =
    "interview_id, case_id, interviewer, scheduled_at, score, notes, outcome, recorded_at";
type InterviewTuple = (
    Uuid,
    Uuid,
    String,
    Option<DateTime<Utc>>,
    Option<f32>,
    String,
    String,
    DateTime<Utc>,
);

const NOTIFICATION_COLS: &str =
    "notification_id, worker_id, kind, channel, subject, body, delivery_ref, created_at";
type NotificationTuple = (
    Uuid,
    Uuid,
    String,
    Option<String>,
    String,
    String,
    Option<String>,
    DateTime<Utc>,
);

impl Store {
    // -- Workers -----------------------------------------------------------

    pub async fn insert_worker(&self, w: &Worker) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_workers \
                     (worker_id, legal_name, national_id_hash, home_county, role_eligibility, \
                      payroll_contract_ref, next_of_kin_name, next_of_kin_phone_hash, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                )
                .bind(w.worker_id)
                .bind(&w.legal_name)
                .bind(&w.national_id_hash)
                .bind(&w.home_county)
                .bind(json!(w.role_eligibility))
                .bind(&w.payroll_contract_ref)
                .bind(&w.next_of_kin_name)
                .bind(&w.next_of_kin_phone_hash)
                .bind(w.created_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                db.workers.push(w.clone());
                Ok(())
            }
        }
    }

    pub async fn get_worker(&self, worker_id: Uuid) -> Result<Option<Worker>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<WorkerTuple> = sqlx::query_as(&format!(
                    "SELECT {WORKER_COLS} FROM workforce_workers WHERE worker_id = $1"
                ))
                .bind(worker_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(worker_from_tuple))
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .workers
                .iter()
                .find(|w| w.worker_id == worker_id)
                .cloned()),
        }
    }

    pub async fn list_workers(
        &self,
        home_county: Option<&str>,
        role: Option<&str>,
    ) -> Result<Vec<Worker>, StoreError> {
        match self {
            Store::Pg(pool) => {
                // role filter uses jsonb containment against a single-element array.
                let role_json = role.map(|r| json!([r]));
                let rows: Vec<WorkerTuple> = sqlx::query_as(&format!(
                    "SELECT {WORKER_COLS} FROM workforce_workers \
                     WHERE ($1::text IS NULL OR home_county = $1) \
                       AND ($2::jsonb IS NULL OR role_eligibility @> $2) \
                     ORDER BY created_at"
                ))
                .bind(home_county)
                .bind(role_json)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(worker_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .workers
                    .iter()
                    .filter(|w| home_county.is_none_or(|c| w.home_county == c))
                    .filter(|w| role.is_none_or(|r| w.role_eligibility.iter().any(|e| e == r)))
                    .cloned()
                    .collect())
            }
        }
    }

    /// Patch an existing worker. `None` fields are left unchanged; the method
    /// returns the stored worker post-update or `NotFound` when absent.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_worker(
        &self,
        worker_id: Uuid,
        home_county: Option<String>,
        role_eligibility: Option<Vec<String>>,
        payroll_contract_ref: Option<String>,
        next_of_kin_name: Option<String>,
        next_of_kin_phone_hash: Option<String>,
    ) -> Result<Worker, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<WorkerTuple> = sqlx::query_as(&format!(
                    "UPDATE workforce_workers SET \
                       home_county = COALESCE($2, home_county), \
                       role_eligibility = COALESCE($3, role_eligibility), \
                       payroll_contract_ref = COALESCE($4, payroll_contract_ref), \
                       next_of_kin_name = COALESCE($5, next_of_kin_name), \
                       next_of_kin_phone_hash = COALESCE($6, next_of_kin_phone_hash) \
                     WHERE worker_id = $1 \
                     RETURNING {WORKER_COLS}"
                ))
                .bind(worker_id)
                .bind(home_county)
                .bind(role_eligibility.map(|v| json!(v)))
                .bind(payroll_contract_ref)
                .bind(next_of_kin_name)
                .bind(next_of_kin_phone_hash)
                .fetch_optional(pool)
                .await?;
                row.map(worker_from_tuple).ok_or(StoreError::NotFound)
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let w = db
                    .workers
                    .iter_mut()
                    .find(|w| w.worker_id == worker_id)
                    .ok_or(StoreError::NotFound)?;
                if let Some(v) = home_county {
                    w.home_county = v;
                }
                if let Some(v) = role_eligibility {
                    w.role_eligibility = v;
                }
                if let Some(v) = payroll_contract_ref {
                    w.payroll_contract_ref = Some(v);
                }
                if let Some(v) = next_of_kin_name {
                    w.next_of_kin_name = Some(v);
                }
                if let Some(v) = next_of_kin_phone_hash {
                    w.next_of_kin_phone_hash = Some(v);
                }
                Ok(w.clone())
            }
        }
    }

    // -- Contacts ----------------------------------------------------------

    /// Insert-or-update the contact point identified by
    /// (worker, channel, value hash). When `preferred` is set, every other
    /// contact point on the same channel is demoted (at most one preferred
    /// per channel). Returns the stored point, whether it was newly added,
    /// and its previous consent flag.
    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_contact(
        &self,
        worker_id: Uuid,
        channel: Channel,
        value_hash_hex: &str,
        verified: bool,
        consented: bool,
        preferred: bool,
        now: DateTime<Utc>,
    ) -> Result<(ContactPoint, bool, bool), StoreError> {
        match self {
            Store::Pg(pool) => {
                let existing: Option<(Uuid, bool)> = sqlx::query_as(
                    "SELECT contact_id, consented FROM workforce_contacts \
                     WHERE worker_id = $1 AND channel = $2 AND value_hash_hex = $3",
                )
                .bind(worker_id)
                .bind(channel.as_str())
                .bind(value_hash_hex)
                .fetch_optional(pool)
                .await?;
                let (contact_id, added, prev_consented) = match existing {
                    Some((cid, prev_consented)) => {
                        sqlx::query(
                            "UPDATE workforce_contacts SET verified = $2, consented = $3, preferred = $4, changed_at = $5 \
                             WHERE contact_id = $1",
                        )
                        .bind(cid)
                        .bind(verified)
                        .bind(consented)
                        .bind(preferred)
                        .bind(now)
                        .execute(pool)
                        .await?;
                        (cid, false, prev_consented)
                    }
                    None => {
                        let cid = Uuid::now_v7();
                        sqlx::query(
                            "INSERT INTO workforce_contacts \
                             (contact_id, worker_id, channel, value_hash_hex, verified, consented, preferred, changed_at) \
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        )
                        .bind(cid)
                        .bind(worker_id)
                        .bind(channel.as_str())
                        .bind(value_hash_hex)
                        .bind(verified)
                        .bind(consented)
                        .bind(preferred)
                        .bind(now)
                        .execute(pool)
                        .await?;
                        (cid, true, false)
                    }
                };
                if preferred {
                    sqlx::query(
                        "UPDATE workforce_contacts SET preferred = FALSE \
                         WHERE worker_id = $1 AND channel = $2 AND contact_id <> $3 AND preferred",
                    )
                    .bind(worker_id)
                    .bind(channel.as_str())
                    .bind(contact_id)
                    .execute(pool)
                    .await?;
                }
                let row: Option<ContactTuple> = sqlx::query_as(&format!(
                    "SELECT {CONTACT_COLS} FROM workforce_contacts WHERE contact_id = $1"
                ))
                .bind(contact_id)
                .fetch_optional(pool)
                .await?;
                let point = row.map(contact_from_tuple).ok_or(StoreError::NotFound)?;
                Ok((point, added, prev_consented))
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                if !db.workers.iter().any(|w| w.worker_id == worker_id) {
                    return Err(StoreError::NotFound);
                }
                let existing = db.contacts.iter_mut().find(|c| {
                    c.worker_id == worker_id
                        && c.channel == channel
                        && c.value_hash_hex == value_hash_hex
                });
                let (contact_id, added, prev_consented) = match existing {
                    Some(c) => {
                        let prev = c.consented;
                        c.verified = verified;
                        c.consented = consented;
                        c.preferred = preferred;
                        c.changed_at = now;
                        (c.contact_id, false, prev)
                    }
                    None => {
                        let cid = Uuid::now_v7();
                        db.contacts.push(ContactPoint {
                            contact_id: cid,
                            worker_id,
                            channel,
                            value_hash_hex: value_hash_hex.to_string(),
                            verified,
                            consented,
                            preferred,
                            changed_at: now,
                        });
                        (cid, true, false)
                    }
                };
                if preferred {
                    for c in db.contacts.iter_mut() {
                        if c.worker_id == worker_id
                            && c.channel == channel
                            && c.contact_id != contact_id
                        {
                            c.preferred = false;
                        }
                    }
                }
                let point = db
                    .contacts
                    .iter()
                    .find(|c| c.contact_id == contact_id)
                    .cloned()
                    .ok_or(StoreError::NotFound)?;
                Ok((point, added, prev_consented))
            }
        }
    }

    pub async fn list_contacts(
        &self,
        worker_id: Uuid,
        channel: Option<Channel>,
    ) -> Result<Vec<ContactPoint>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<ContactTuple> = sqlx::query_as(&format!(
                    "SELECT {CONTACT_COLS} FROM workforce_contacts \
                     WHERE worker_id = $1 AND ($2::text IS NULL OR channel = $2) ORDER BY changed_at"
                ))
                .bind(worker_id)
                .bind(channel.map(|c| c.as_str().to_string()))
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(contact_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .contacts
                    .iter()
                    .filter(|c| c.worker_id == worker_id)
                    .filter(|c| channel.is_none_or(|ch| c.channel == ch))
                    .cloned()
                    .collect())
            }
        }
    }

    pub async fn verify_contact_by_id(
        &self,
        contact_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<ContactPoint, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<ContactTuple> = sqlx::query_as(&format!(
                    "UPDATE workforce_contacts SET verified = TRUE, changed_at = $2 WHERE contact_id = $1 \
                     RETURNING {CONTACT_COLS}"
                ))
                .bind(contact_id)
                .bind(now)
                .fetch_optional(pool)
                .await?;
                row.map(contact_from_tuple).ok_or(StoreError::NotFound)
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let c = db
                    .contacts
                    .iter_mut()
                    .find(|c| c.contact_id == contact_id)
                    .ok_or(StoreError::NotFound)?;
                c.verified = true;
                c.changed_at = now;
                Ok(c.clone())
            }
        }
    }

    pub async fn remove_contact_by_id(&self, contact_id: Uuid) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let result = sqlx::query("DELETE FROM workforce_contacts WHERE contact_id = $1")
                    .bind(contact_id)
                    .execute(pool)
                    .await?;
                if result.rows_affected() == 0 {
                    Err(StoreError::NotFound)
                } else {
                    Ok(())
                }
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let before = db.contacts.len();
                db.contacts.retain(|c| c.contact_id != contact_id);
                if db.contacts.len() == before {
                    Err(StoreError::NotFound)
                } else {
                    Ok(())
                }
            }
        }
    }

    pub async fn append_contact_log(&self, entry: &ContactLogEntry) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_contact_log (log_id, worker_id, channel, at, actor, action, reason) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(Uuid::now_v7())
                .bind(entry.worker_id)
                .bind(entry.channel.as_str())
                .bind(entry.at)
                .bind(&entry.actor)
                .bind(entry.action.as_str())
                .bind(&entry.reason)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().contact_log.push(entry.clone());
                Ok(())
            }
        }
    }

    pub async fn contact_history(
        &self,
        worker_id: Uuid,
    ) -> Result<Vec<ContactLogEntry>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<LogTuple> = sqlx::query_as(
                    "SELECT worker_id, channel, at, actor, action, reason \
                         FROM workforce_contact_log WHERE worker_id = $1 ORDER BY at",
                )
                .bind(worker_id)
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(worker_id, channel, at, actor, action, reason)| ContactLogEntry {
                            worker_id,
                            channel: Channel::parse(&channel).unwrap_or(Channel::Phone),
                            at,
                            actor,
                            action: ContactAction::parse(&action).unwrap_or(ContactAction::Updated),
                            reason,
                        },
                    )
                    .collect())
            }
            Store::Mem(mem) => {
                let mut entries: Vec<ContactLogEntry> = mem
                    .lock()
                    .contact_log
                    .iter()
                    .filter(|e| e.worker_id == worker_id)
                    .cloned()
                    .collect();
                entries.sort_by_key(|e| e.at);
                Ok(entries)
            }
        }
    }

    // -- Recruitment cases ---------------------------------------------------

    /// Flattened recruitment stage changes across all cases, newest first.
    pub async fn list_stage_changes(&self) -> Result<Vec<(Uuid, StageChange)>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<StageChangeTuple> = sqlx::query_as(
                    "SELECT case_id, from_stage, to_stage, actor, at, reason \
                     FROM workforce_stage_history ORDER BY at DESC",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(|(case_id, from, to, actor, at, reason)| {
                        (
                            case_id,
                            StageChange {
                                from,
                                to,
                                actor,
                                at,
                                reason,
                            },
                        )
                    })
                    .collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                let mut out = Vec::new();
                for case in &db.cases {
                    for ch in &case.stage_history {
                        out.push((case.case_id, ch.clone()));
                    }
                }
                out.sort_by_key(|(_, ch)| std::cmp::Reverse(ch.at));
                Ok(out)
            }
        }
    }

    /// All contact-point change-log entries, newest first.
    pub async fn list_all_contact_log(&self) -> Result<Vec<ContactLogEntry>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<LogTuple> = sqlx::query_as(
                    "SELECT worker_id, channel, at, actor, action, reason \
                     FROM workforce_contact_log ORDER BY at DESC",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(worker_id, channel, at, actor, action, reason)| ContactLogEntry {
                            worker_id,
                            channel: Channel::parse(&channel).unwrap_or(Channel::Phone),
                            at,
                            actor,
                            action: ContactAction::parse(&action).unwrap_or(ContactAction::Updated),
                            reason,
                        },
                    )
                    .collect())
            }
            Store::Mem(mem) => {
                let mut entries = mem.lock().contact_log.clone();
                entries.sort_by_key(|e| std::cmp::Reverse(e.at));
                Ok(entries)
            }
        }
    }

    pub async fn insert_case(&self, case: &RecruitmentCase) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_recruitment_cases \
                     (case_id, application_ref, applicant_name, national_id_hash, role_code, county, \
                      stage, reject_reason, waitlist_rank, composite_score, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                )
                .bind(case.case_id)
                .bind(&case.application_ref)
                .bind(&case.applicant_name)
                .bind(&case.national_id_hash)
                .bind(&case.role_code)
                .bind(&case.county)
                .bind(case.stage.state())
                .bind(stage_reject(&case.stage))
                .bind(stage_rank(&case.stage))
                .bind(case.composite_score)
                .bind(case.created_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().cases.push(case.clone());
                Ok(())
            }
        }
    }

    pub async fn get_case(&self, case_id: Uuid) -> Result<Option<RecruitmentCase>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<CaseTuple> = sqlx::query_as(&format!(
                    "SELECT {CASE_COLS} FROM workforce_recruitment_cases WHERE case_id = $1"
                ))
                .bind(case_id)
                .fetch_optional(pool)
                .await?;
                let Some(row) = row else { return Ok(None) };
                let history = sqlx::query_as::<_, (String, String, String, DateTime<Utc>, Option<String>)>(
                    &format!("SELECT {HISTORY_COLS} FROM workforce_stage_history WHERE case_id = $1 ORDER BY at"),
                )
                .bind(case_id)
                .fetch_all(pool)
                .await?;
                let mut case = case_from_tuple(row);
                case.stage_history = history.into_iter().map(history_from_tuple).collect();
                Ok(Some(case))
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db.cases.iter().find(|c| c.case_id == case_id).cloned())
            }
        }
    }

    pub async fn list_cases(
        &self,
        stage: Option<&str>,
        county: Option<&str>,
    ) -> Result<Vec<RecruitmentCase>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<CaseTuple> = sqlx::query_as(&format!(
                    "SELECT {CASE_COLS} FROM workforce_recruitment_cases \
                     WHERE ($1::text IS NULL OR stage = $1) AND ($2::text IS NULL OR county = $2) \
                     ORDER BY created_at"
                ))
                .bind(stage)
                .bind(county)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(case_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .cases
                    .iter()
                    .filter(|c| stage.is_none_or(|s| c.stage.state() == s))
                    .filter(|c| county.is_none_or(|c2| c.county == c2))
                    .cloned()
                    .collect())
            }
        }
    }

    /// Persist a stage transition plus its attributed history entry.
    pub async fn apply_transition(
        &self,
        case_id: Uuid,
        to: &str,
        reject_reason: Option<&str>,
        waitlist_rank: Option<i64>,
        change: &StageChange,
    ) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let mut tx = pool.begin().await?;
                sqlx::query(
                    "UPDATE workforce_recruitment_cases SET stage = $2, reject_reason = $3, waitlist_rank = $4 \
                     WHERE case_id = $1",
                )
                .bind(case_id)
                .bind(to)
                .bind(reject_reason)
                .bind(waitlist_rank)
                .execute(&mut *tx)
                .await?;
                sqlx::query(
                    "INSERT INTO workforce_stage_history (history_id, case_id, from_stage, to_stage, actor, at, reason) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(Uuid::now_v7())
                .bind(case_id)
                .bind(&change.from)
                .bind(&change.to)
                .bind(&change.actor)
                .bind(change.at)
                .bind(&change.reason)
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let case = db
                    .cases
                    .iter_mut()
                    .find(|c| c.case_id == case_id)
                    .ok_or(StoreError::NotFound)?;
                case.stage = Stage::from_row(to, reject_reason.map(str::to_string), waitlist_rank);
                case.stage_history.push(change.clone());
                Ok(())
            }
        }
    }

    /// Candidates holding or past a selection slot for (county, role):
    /// stages `selected` and `appointed` both consume capacity.
    pub async fn count_committed(&self, county: &str, role_code: &str) -> Result<i64, StoreError> {
        match self {
            Store::Pg(pool) => {
                let (n,): (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM workforce_recruitment_cases \
                     WHERE county = $1 AND role_code = $2 AND stage IN ('selected', 'appointed')",
                )
                .bind(county)
                .bind(role_code)
                .fetch_one(pool)
                .await?;
                Ok(n)
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .cases
                    .iter()
                    .filter(|c| c.county == county && c.role_code == role_code)
                    .filter(|c| matches!(c.stage.state(), "selected" | "appointed"))
                    .count() as i64)
            }
        }
    }

    /// Number of waitlisted candidates for (county, role) whose composite
    /// score ranks at or above `score` (None scores sort last).
    pub async fn count_waitlist_ahead(
        &self,
        county: &str,
        role_code: &str,
        score: Option<f32>,
    ) -> Result<i64, StoreError> {
        const FLOOR: f32 = f32::MIN;
        match self {
            Store::Pg(pool) => {
                let (n,): (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM workforce_recruitment_cases \
                     WHERE county = $1 AND role_code = $2 AND stage = 'waitlist' \
                       AND COALESCE(composite_score, $3::real) >= COALESCE($4::real, $3::real)",
                )
                .bind(county)
                .bind(role_code)
                .bind(FLOOR)
                .bind(score)
                .fetch_one(pool)
                .await?;
                Ok(n)
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .cases
                    .iter()
                    .filter(|c| c.county == county && c.role_code == role_code)
                    .filter(|c| c.stage.state() == "waitlist")
                    .filter(|c| c.composite_score.unwrap_or(FLOOR) >= score.unwrap_or(FLOOR))
                    .count() as i64)
            }
        }
    }

    // -- Postings ----------------------------------------------------------

    /// Insert a posting; when `supersede` is set that predecessor becomes
    /// `superseded` pointing at the new posting (chain preserved).
    pub async fn insert_posting(
        &self,
        posting: &PostingOrder,
        supersede: Option<Uuid>,
    ) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let mut tx = pool.begin().await?;
                sqlx::query(
                    "INSERT INTO workforce_postings \
                     (posting_id, worker_id, station_code, role_code, effective_from, effective_to, \
                      posted_by, superseded_by, status, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                )
                .bind(posting.posting_id)
                .bind(posting.worker_id)
                .bind(&posting.station_code)
                .bind(&posting.role_code)
                .bind(posting.effective_from)
                .bind(posting.effective_to)
                .bind(&posting.posted_by)
                .bind(posting.superseded_by)
                .bind(&posting.status)
                .bind(posting.created_at)
                .execute(&mut *tx)
                .await?;
                if let Some(old) = supersede {
                    sqlx::query(
                        "UPDATE workforce_postings SET status = 'superseded', superseded_by = $2 WHERE posting_id = $1",
                    )
                    .bind(old)
                    .bind(posting.posting_id)
                    .execute(&mut *tx)
                    .await?;
                }
                tx.commit().await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                if let Some(old) = supersede {
                    if let Some(p) = db.postings.iter_mut().find(|p| p.posting_id == old) {
                        p.status = "superseded".to_string();
                        p.superseded_by = Some(posting.posting_id);
                    }
                }
                db.postings.push(posting.clone());
                Ok(())
            }
        }
    }

    pub async fn get_posting(&self, posting_id: Uuid) -> Result<Option<PostingOrder>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<PostingTuple> = sqlx::query_as(&format!(
                    "SELECT {POSTING_COLS} FROM workforce_postings WHERE posting_id = $1"
                ))
                .bind(posting_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(posting_from_tuple))
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .postings
                .iter()
                .find(|p| p.posting_id == posting_id)
                .cloned()),
        }
    }

    pub async fn active_posting_for_worker(
        &self,
        worker_id: Uuid,
    ) -> Result<Option<PostingOrder>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<PostingTuple> = sqlx::query_as(&format!(
                    "SELECT {POSTING_COLS} FROM workforce_postings \
                     WHERE worker_id = $1 AND status = 'active' ORDER BY created_at DESC LIMIT 1"
                ))
                .bind(worker_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(posting_from_tuple))
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .postings
                    .iter()
                    .filter(|p| p.worker_id == worker_id && p.status == "active")
                    .max_by_key(|p| p.created_at)
                    .cloned())
            }
        }
    }

    pub async fn list_postings(
        &self,
        worker_id: Option<Uuid>,
        station: Option<&str>,
        active: Option<bool>,
    ) -> Result<Vec<PostingOrder>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<PostingTuple> = sqlx::query_as(&format!(
                    "SELECT {POSTING_COLS} FROM workforce_postings \
                     WHERE ($1::uuid IS NULL OR worker_id = $1) \
                       AND ($2::text IS NULL OR station_code = $2) \
                       AND ($3::bool IS NULL OR status = CASE WHEN $3 THEN 'active' ELSE 'superseded' END) \
                     ORDER BY created_at"
                ))
                .bind(worker_id)
                .bind(station)
                .bind(active)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(posting_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .postings
                    .iter()
                    .filter(|p| worker_id.is_none_or(|w| p.worker_id == w))
                    .filter(|p| station.is_none_or(|s| p.station_code == s))
                    .filter(|p| active.is_none_or(|a| (p.status == "active") == a))
                    .cloned()
                    .collect())
            }
        }
    }

    // -- Equipment claims --------------------------------------------------

    pub async fn insert_claim(&self, claim: &EquipmentClaim) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let open: Option<(Uuid,)> = sqlx::query_as(
                    "SELECT claim_id FROM workforce_equipment_claims WHERE posting_id = $1 AND returned = FALSE LIMIT 1",
                )
                .bind(claim.posting_id)
                .fetch_optional(pool)
                .await?;
                if open.is_some() {
                    return Err(StoreError::OpenClaimExists);
                }
                sqlx::query(
                    "INSERT INTO workforce_equipment_claims \
                     (claim_id, posting_id, kit_serials, acknowledged, returned, loss_flags, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(claim.claim_id)
                .bind(claim.posting_id)
                .bind(json!(claim.kit_serials))
                .bind(claim.acknowledged)
                .bind(claim.returned)
                .bind(json!(claim.loss_flags))
                .bind(claim.created_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                if db
                    .claims
                    .iter()
                    .any(|c| c.posting_id == claim.posting_id && !c.returned)
                {
                    return Err(StoreError::OpenClaimExists);
                }
                db.claims.push(claim.clone());
                Ok(())
            }
        }
    }

    pub async fn get_claim(&self, claim_id: Uuid) -> Result<Option<EquipmentClaim>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<ClaimTuple> = sqlx::query_as(&format!(
                    "SELECT {CLAIM_COLS} FROM workforce_equipment_claims WHERE claim_id = $1"
                ))
                .bind(claim_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(claim_from_tuple))
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .claims
                .iter()
                .find(|c| c.claim_id == claim_id)
                .cloned()),
        }
    }

    pub async fn update_claim(
        &self,
        claim_id: Uuid,
        acknowledged: Option<bool>,
        returned: Option<bool>,
    ) -> Result<EquipmentClaim, StoreError> {
        match self {
            Store::Pg(pool) => {
                let mut sets: Vec<String> = Vec::new();
                let mut binds: Vec<bool> = Vec::new();
                if let Some(a) = acknowledged {
                    binds.push(a);
                    sets.push(format!("acknowledged = ${}", 2 + binds.len()));
                }
                if let Some(r) = returned {
                    binds.push(r);
                    sets.push(format!("returned = ${}", 2 + binds.len()));
                }
                let sql = format!(
                    "UPDATE workforce_equipment_claims SET {} WHERE claim_id = $1 RETURNING {CLAIM_COLS}",
                    sets.join(", ")
                );
                let mut q = sqlx::query_as::<_, ClaimTuple>(&sql).bind(claim_id);
                for b in binds {
                    q = q.bind(b);
                }
                let row = q.fetch_optional(pool).await?;
                row.map(claim_from_tuple).ok_or(StoreError::NotFound)
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let claim = db
                    .claims
                    .iter_mut()
                    .find(|c| c.claim_id == claim_id)
                    .ok_or(StoreError::NotFound)?;
                if let Some(a) = acknowledged {
                    claim.acknowledged = a;
                }
                if let Some(r) = returned {
                    claim.returned = r;
                }
                Ok(claim.clone())
            }
        }
    }

    pub async fn list_claims(&self, open_only: bool) -> Result<Vec<EquipmentClaim>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<ClaimTuple> = sqlx::query_as(&format!(
                    "SELECT {CLAIM_COLS} FROM workforce_equipment_claims \
                     WHERE ($1::bool IS NULL OR returned <> $1) ORDER BY created_at"
                ))
                .bind(if open_only { Some(true) } else { None })
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(claim_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .claims
                    .iter()
                    .filter(|c| !open_only || !c.returned)
                    .cloned()
                    .collect())
            }
        }
    }

    pub async fn append_loss_flags(
        &self,
        claim_id: Uuid,
        flags: &[String],
    ) -> Result<EquipmentClaim, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<ClaimTuple> = sqlx::query_as(&format!(
                    "UPDATE workforce_equipment_claims SET loss_flags = loss_flags || $2::jsonb \
                     WHERE claim_id = $1 RETURNING {CLAIM_COLS}"
                ))
                .bind(claim_id)
                .bind(json!(flags))
                .fetch_optional(pool)
                .await?;
                row.map(claim_from_tuple).ok_or(StoreError::NotFound)
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let claim = db
                    .claims
                    .iter_mut()
                    .find(|c| c.claim_id == claim_id)
                    .ok_or(StoreError::NotFound)?;
                claim.loss_flags.extend(flags.iter().cloned());
                Ok(claim.clone())
            }
        }
    }

    // -- Tasking -----------------------------------------------------------

    pub async fn insert_template(&self, t: &TaskTemplate) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_task_templates \
                     (template_id, role_code, phase, title, critical, training_content_id, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(t.template_id)
                .bind(&t.role_code)
                .bind(t.phase.as_str())
                .bind(&t.title)
                .bind(t.critical)
                .bind(&t.training_content_id)
                .bind(t.created_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().templates.push(t.clone());
                Ok(())
            }
        }
    }

    pub async fn get_template(
        &self,
        template_id: Uuid,
    ) -> Result<Option<TaskTemplate>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<TemplateTuple> = sqlx::query_as(&format!(
                    "SELECT {TEMPLATE_COLS} FROM workforce_task_templates WHERE template_id = $1"
                ))
                .bind(template_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(template_from_tuple))
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .templates
                .iter()
                .find(|t| t.template_id == template_id)
                .cloned()),
        }
    }

    pub async fn list_templates(
        &self,
        role: Option<&str>,
        phase: Option<&str>,
    ) -> Result<Vec<TaskTemplate>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<TemplateTuple> = sqlx::query_as(&format!(
                    "SELECT {TEMPLATE_COLS} FROM workforce_task_templates \
                     WHERE ($1::text IS NULL OR role_code = $1) AND ($2::text IS NULL OR phase = $2) \
                     ORDER BY created_at"
                ))
                .bind(role)
                .bind(phase)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(template_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .templates
                    .iter()
                    .filter(|t| role.is_none_or(|r| t.role_code == r))
                    .filter(|t| phase.is_none_or(|p| t.phase.as_str() == p))
                    .cloned()
                    .collect())
            }
        }
    }

    pub async fn insert_assignment(&self, a: &TaskAssignment) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_task_assignments \
                     (assignment_id, posting_id, template_id, due_from, due_to, status, evidence_hash, block_reason, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                )
                .bind(a.assignment_id)
                .bind(a.posting_id)
                .bind(a.template_id)
                .bind(a.due_from)
                .bind(a.due_to)
                .bind(a.status.as_str())
                .bind(&a.evidence_hash)
                .bind(&a.block_reason)
                .bind(a.updated_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().assignments.push(a.clone());
                Ok(())
            }
        }
    }

    pub async fn get_assignment(
        &self,
        assignment_id: Uuid,
    ) -> Result<Option<TaskAssignment>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<AssignmentTuple> = sqlx::query_as(&format!(
                    "SELECT {ASSIGNMENT_COLS} FROM workforce_task_assignments WHERE assignment_id = $1"
                ))
                .bind(assignment_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(assignment_from_tuple))
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .assignments
                .iter()
                .find(|a| a.assignment_id == assignment_id)
                .cloned()),
        }
    }

    pub async fn list_assignments(
        &self,
        posting_id: Option<Uuid>,
        status: Option<&str>,
    ) -> Result<Vec<TaskAssignment>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<AssignmentTuple> = sqlx::query_as(&format!(
                    "SELECT {ASSIGNMENT_COLS} FROM workforce_task_assignments \
                     WHERE ($1::uuid IS NULL OR posting_id = $1) AND ($2::text IS NULL OR status = $2) \
                     ORDER BY updated_at"
                ))
                .bind(posting_id)
                .bind(status)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(assignment_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .assignments
                    .iter()
                    .filter(|a| posting_id.is_none_or(|p| a.posting_id == p))
                    .filter(|a| status.is_none_or(|s| a.status.as_str() == s))
                    .cloned()
                    .collect())
            }
        }
    }

    pub async fn set_assignment_status(
        &self,
        assignment_id: Uuid,
        status: TaskStatus,
        evidence_hash: Option<&str>,
        block_reason: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<TaskAssignment, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<AssignmentTuple> = sqlx::query_as(&format!(
                    "UPDATE workforce_task_assignments SET status = $2, evidence_hash = $3, block_reason = $4, \
                     updated_at = $5 WHERE assignment_id = $1 RETURNING {ASSIGNMENT_COLS}"
                ))
                .bind(assignment_id)
                .bind(status.as_str())
                .bind(evidence_hash)
                .bind(block_reason)
                .bind(now)
                .fetch_optional(pool)
                .await?;
                row.map(assignment_from_tuple).ok_or(StoreError::NotFound)
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let a = db
                    .assignments
                    .iter_mut()
                    .find(|a| a.assignment_id == assignment_id)
                    .ok_or(StoreError::NotFound)?;
                a.status = status;
                a.evidence_hash = evidence_hash.map(str::to_string);
                a.block_reason = block_reason.map(str::to_string);
                a.updated_at = now;
                Ok(a.clone())
            }
        }
    }

    // -- Performance -------------------------------------------------------

    pub async fn insert_record(&self, r: &PerformanceRecord) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_performance_records \
                     (record_id, worker_id, category, score, detail, recorded_by, recorded_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(r.record_id)
                .bind(r.worker_id)
                .bind(r.category.as_str())
                .bind(r.score)
                .bind(&r.detail)
                .bind(&r.recorded_by)
                .bind(r.recorded_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().perf.push(r.clone());
                Ok(())
            }
        }
    }

    pub async fn list_records(
        &self,
        worker_id: Uuid,
    ) -> Result<Vec<PerformanceRecord>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<PerfTuple> = sqlx::query_as(&format!(
                    "SELECT {PERF_COLS} FROM workforce_performance_records \
                     WHERE worker_id = $1 ORDER BY recorded_at"
                ))
                .bind(worker_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(perf_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .perf
                    .iter()
                    .filter(|r| r.worker_id == worker_id)
                    .cloned()
                    .collect())
            }
        }
    }

    // -- Interviews --------------------------------------------------------

    pub async fn insert_interview(&self, i: &InterviewRecord) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_interviews \
                     (interview_id, case_id, interviewer, scheduled_at, score, notes, outcome, recorded_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(i.interview_id)
                .bind(i.case_id)
                .bind(&i.interviewer)
                .bind(i.scheduled_at)
                .bind(i.score)
                .bind(&i.notes)
                .bind(i.outcome.as_str())
                .bind(i.recorded_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().interviews.push(i.clone());
                Ok(())
            }
        }
    }

    pub async fn list_interviews(&self, case_id: Uuid) -> Result<Vec<InterviewRecord>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<InterviewTuple> = sqlx::query_as(&format!(
                    "SELECT {INTERVIEW_COLS} FROM workforce_interviews WHERE case_id = $1 ORDER BY recorded_at"
                ))
                .bind(case_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(interview_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .interviews
                    .iter()
                    .filter(|i| i.case_id == case_id)
                    .cloned()
                    .collect())
            }
        }
    }

    // -- Notifications -----------------------------------------------------

    pub async fn insert_notification(&self, n: &Notification) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO workforce_notifications \
                     (notification_id, worker_id, kind, channel, subject, body, delivery_ref, created_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(n.notification_id)
                .bind(n.worker_id)
                .bind(n.kind.as_str())
                .bind(n.channel.map(|c| c.as_str().to_string()))
                .bind(&n.subject)
                .bind(&n.body)
                .bind(&n.delivery_ref)
                .bind(n.created_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().notifications.push(n.clone());
                Ok(())
            }
        }
    }

    pub async fn list_notifications(
        &self,
        worker_id: Uuid,
    ) -> Result<Vec<Notification>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<NotificationTuple> = sqlx::query_as(&format!(
                    "SELECT {NOTIFICATION_COLS} FROM workforce_notifications \
                     WHERE worker_id = $1 ORDER BY created_at"
                ))
                .bind(worker_id)
                .fetch_all(pool)
                .await?;
                Ok(rows.into_iter().map(notification_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .notifications
                    .iter()
                    .filter(|n| n.worker_id == worker_id)
                    .cloned()
                    .collect())
            }
        }
    }

    /// The worker's preferred contact point across all channels (the primary
    /// delivery target when a notification has no explicit channel).
    pub async fn preferred_contact(
        &self,
        worker_id: Uuid,
    ) -> Result<Option<ContactPoint>, StoreError> {
        let contacts = self.list_contacts(worker_id, None).await?;
        Ok(contacts
            .iter()
            .find(|c| c.preferred)
            .or_else(|| contacts.first())
            .cloned())
    }

    // -- Events (design §8) ------------------------------------------------

    /// Emit a cross-service event. On `Pg` this writes the outbox row (Debezium
    /// CDC publishes it to `envelope.topic`); on `Mem` it records the envelope
    /// so hermetic tests can assert the exact event contract.
    pub async fn emit(&self, envelope: &eel_outbox_rs::OutboxEnvelope) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let outbox = eel_outbox_rs::Outbox::new(pool.clone());
                let mut tx = pool.begin().await?;
                outbox.emit(&mut tx, envelope).await?;
                tx.commit().await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().emitted.push(envelope.clone());
                Ok(())
            }
        }
    }

    /// Envelopes emitted so far (Mem store only; empty otherwise).
    pub async fn emitted_events(&self) -> Vec<eel_outbox_rs::OutboxEnvelope> {
        match self {
            Store::Mem(mem) => mem.lock().emitted.clone(),
            Store::Pg(_) => Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Row mappers
// ---------------------------------------------------------------------------

fn worker_from_tuple(t: WorkerTuple) -> Worker {
    Worker {
        worker_id: t.0,
        legal_name: t.1,
        national_id_hash: t.2,
        home_county: t.3,
        role_eligibility: serde_json::from_value(t.4).unwrap_or_default(),
        payroll_contract_ref: t.5,
        next_of_kin_name: t.6,
        next_of_kin_phone_hash: t.7,
        created_at: t.8,
    }
}

fn contact_from_tuple(t: ContactTuple) -> ContactPoint {
    ContactPoint {
        contact_id: t.0,
        worker_id: t.1,
        channel: Channel::parse(&t.2).unwrap_or(Channel::Phone),
        value_hash_hex: t.3,
        verified: t.4,
        consented: t.5,
        preferred: t.6,
        changed_at: t.7,
    }
}

fn case_from_tuple(t: CaseTuple) -> RecruitmentCase {
    RecruitmentCase {
        case_id: t.0,
        application_ref: t.1,
        applicant_name: t.2,
        national_id_hash: t.3,
        role_code: t.4,
        county: t.5,
        stage: Stage::from_row(&t.6, t.7, t.8),
        composite_score: t.9,
        stage_history: Vec::new(),
        created_at: t.10,
    }
}

fn history_from_tuple(t: (String, String, String, DateTime<Utc>, Option<String>)) -> StageChange {
    StageChange {
        from: t.0,
        to: t.1,
        actor: t.2,
        at: t.3,
        reason: t.4,
    }
}

fn posting_from_tuple(t: PostingTuple) -> PostingOrder {
    PostingOrder {
        posting_id: t.0,
        worker_id: t.1,
        station_code: t.2,
        role_code: t.3,
        effective_from: t.4,
        effective_to: t.5,
        posted_by: t.6,
        superseded_by: t.7,
        status: t.8,
        created_at: t.9,
    }
}

fn claim_from_tuple(t: ClaimTuple) -> EquipmentClaim {
    EquipmentClaim {
        claim_id: t.0,
        posting_id: t.1,
        kit_serials: serde_json::from_value(t.2).unwrap_or_default(),
        acknowledged: t.3,
        returned: t.4,
        loss_flags: serde_json::from_value(t.5).unwrap_or_default(),
        created_at: t.6,
    }
}

fn template_from_tuple(t: TemplateTuple) -> TaskTemplate {
    TaskTemplate {
        template_id: t.0,
        role_code: t.1,
        phase: Phase::parse(&t.2).unwrap_or(Phase::PollingDay),
        title: t.3,
        critical: t.4,
        training_content_id: t.5,
        created_at: t.6,
    }
}

fn assignment_from_tuple(t: AssignmentTuple) -> TaskAssignment {
    let status = match t.5.as_str() {
        "acked" => TaskStatus::Acked,
        "done" => TaskStatus::Done,
        "blocked" => TaskStatus::Blocked,
        _ => TaskStatus::Pending,
    };
    TaskAssignment {
        assignment_id: t.0,
        posting_id: t.1,
        template_id: t.2,
        due_from: t.3,
        due_to: t.4,
        status,
        evidence_hash: t.6,
        block_reason: t.7,
        updated_at: t.8,
    }
}

fn perf_from_tuple(t: PerfTuple) -> PerformanceRecord {
    PerformanceRecord {
        record_id: t.0,
        worker_id: t.1,
        category: match t.2.as_str() {
            "punctuality" => PerfCategory::Punctuality,
            "incident" => PerfCategory::Incident,
            "evaluation" => PerfCategory::Evaluation,
            _ => PerfCategory::Attendance,
        },
        score: t.3,
        detail: t.4,
        recorded_by: t.5,
        recorded_at: t.6,
    }
}

fn interview_from_tuple(t: InterviewTuple) -> InterviewRecord {
    InterviewRecord {
        interview_id: t.0,
        case_id: t.1,
        interviewer: t.2,
        scheduled_at: t.3,
        score: t.4,
        notes: t.5,
        outcome: InterviewOutcome::parse(&t.6).unwrap_or(InterviewOutcome::Hold),
        recorded_at: t.7,
    }
}

fn notification_from_tuple(t: NotificationTuple) -> Notification {
    Notification {
        notification_id: t.0,
        worker_id: t.1,
        kind: NotificationKind::parse(&t.2).unwrap_or(NotificationKind::Message),
        channel: t.3.and_then(|c| Channel::parse(&c)),
        subject: t.4,
        body: t.5,
        delivery_ref: t.6,
        created_at: t.7,
    }
}

fn stage_reject(stage: &Stage) -> Option<String> {
    match stage {
        Stage::Rejected { reason_code } => Some(reason_code.clone()),
        _ => None,
    }
}

fn stage_rank(stage: &Stage) -> Option<i64> {
    match stage {
        Stage::Waitlist { rank } => Some(*rank),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// In-memory fake
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MemData {
    workers: Vec<Worker>,
    contacts: Vec<ContactPoint>,
    contact_log: Vec<ContactLogEntry>,
    cases: Vec<RecruitmentCase>,
    postings: Vec<PostingOrder>,
    claims: Vec<EquipmentClaim>,
    templates: Vec<TaskTemplate>,
    assignments: Vec<TaskAssignment>,
    perf: Vec<PerformanceRecord>,
    interviews: Vec<InterviewRecord>,
    notifications: Vec<Notification>,
    /// Cross-service events emitted so far (mirrors the Pg outbox for tests).
    emitted: Vec<eel_outbox_rs::OutboxEnvelope>,
}

#[derive(Clone, Default)]
pub struct MemStore(Arc<Mutex<MemData>>);

impl MemStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, MemData> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
