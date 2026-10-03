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

//! Persistence for risk-limiting audits (M21): runs, drawn samples,
//! interpretations, entropy provenance, witness sign-offs and reports.
//!
//! [`Store`] is an enum over the two backends so handlers stay dyn-free:
//! `Pg` runs sqlx runtime queries against the governance-svc schema;
//! `Mem` is an in-process fake used by hermetic tests and mirrors exactly
//! the same state transitions and uniqueness semantics.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AuditManifest, AuditReport, AuditRun, EntropySource, SampledBallot, WitnessRecord,
};

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

/// A run plus everything needed to continue the audit offline: the reported
/// totals that rebuild its pairwise assertion, and the seed + manifest that
/// reproduce (and extend) the seeded draw at adaptive-expansion time.
#[derive(Debug, Clone)]
pub struct RunRow {
    pub run: AuditRun,
    pub winner_id: String,
    pub loser_id: String,
    pub winner_votes: u64,
    pub loser_votes: u64,
    pub seed: String,
    pub manifest: AuditManifest,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("resource not found")]
    NotFound,
    #[error("ballot already has an audited interpretation")]
    DuplicateSampleInterpretation,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

type RunTuple = (
    String,
    String,
    String,
    f64,
    String,
    String,
    String,
    String,
    i64,
    i64,
    String,
    String,
);
type SampleTuple = (String, String, i32, String, Option<String>, Option<String>);
type InterpretedTuple = (String, i32, String, Option<String>, Option<String>);

#[derive(Clone)]
pub enum Store {
    Pg(PgPool),
    Mem(MemStore),
}

impl Store {
    pub async fn create_run(
        &self,
        row: &RunRow,
        entropy: &EntropySource,
    ) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO rla_audit_runs \
                     (run_id, election_id, contest_id, risk_limit, status, created_at, \
                       winner_id, loser_id, winner_votes, loser_votes, seed, manifest) \
                      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                )
                .bind(&row.run.run_id)
                .bind(&row.run.election_id)
                .bind(&row.run.contest_id)
                .bind(row.run.risk_limit)
                .bind(&row.run.status)
                .bind(&row.run.created_at)
                .bind(&row.winner_id)
                .bind(&row.loser_id)
                .bind(row.winner_votes as i64)
                .bind(row.loser_votes as i64)
                .bind(&row.seed)
                .bind(serde_json::to_string(&row.manifest).unwrap_or_default())
                .execute(pool)
                .await?;
                self.save_entropy_source(entropy).await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                db.runs.insert(row.run.run_id.clone(), row.clone());
                db.entropy
                    .insert(entropy.source_id.clone(), entropy.clone());
                Ok(())
            }
        }
    }

    pub async fn get_run(&self, run_id: &str) -> Result<Option<RunRow>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<RunTuple> = sqlx::query_as(
                    "SELECT run_id, election_id, contest_id, risk_limit, status, created_at, \
                                winner_id, loser_id, winner_votes, loser_votes, seed, manifest \
                          FROM rla_audit_runs WHERE run_id = $1",
                )
                .bind(run_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(|r| RunRow {
                    run: AuditRun {
                        run_id: r.0,
                        election_id: r.1,
                        contest_id: r.2,
                        risk_limit: r.3,
                        status: r.4,
                        created_at: r.5,
                    },
                    winner_id: r.6,
                    loser_id: r.7,
                    winner_votes: r.8 as u64,
                    loser_votes: r.9 as u64,
                    seed: r.10,
                    manifest: serde_json::from_str(&r.11).unwrap_or_default(),
                }))
            }
            Store::Mem(mem) => Ok(mem.lock().runs.get(run_id).cloned()),
        }
    }

    pub async fn list_runs(&self) -> Result<Vec<AuditRun>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<(String, String, String, f64, String, String)> = sqlx::query_as(
                    "SELECT run_id, election_id, contest_id, risk_limit, status, created_at \
                     FROM rla_audit_runs ORDER BY created_at",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(run_id, election_id, contest_id, risk_limit, status, created_at)| {
                            AuditRun {
                                run_id,
                                election_id,
                                contest_id,
                                risk_limit,
                                status,
                                created_at,
                            }
                        },
                    )
                    .collect())
            }
            Store::Mem(mem) => {
                let mut runs: Vec<AuditRun> =
                    mem.lock().runs.values().map(|r| r.run.clone()).collect();
                runs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                Ok(runs)
            }
        }
    }

    /// Forward-only status transition (initialised -> sampling -> auditing ->
    /// finalised); the caller owns the legality of the hop.
    pub async fn update_status(&self, run_id: &str, status: &str) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let result = sqlx::query("UPDATE rla_audit_runs SET status = $2 WHERE run_id = $1")
                    .bind(run_id)
                    .bind(status)
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
                let row = db.runs.get_mut(run_id).ok_or(StoreError::NotFound)?;
                row.run.status = status.to_string();
                Ok(())
            }
        }
    }

    pub async fn insert_samples(
        &self,
        run_id: &str,
        ballots: &[SampledBallot],
    ) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                for b in ballots {
                    sqlx::query(
                        "INSERT INTO rla_sampled_ballots \
                         (run_id, ballot_id, batch_id, position, manifest_entry, reported_vote, audited_vote) \
                         VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    )
                    .bind(run_id)
                    .bind(&b.ballot_id)
                    .bind(&b.batch_id)
                    .bind(b.position as i32)
                    .bind(&b.manifest_entry)
                    .bind(&b.reported_vote)
                    .bind(&b.audited_vote)
                    .execute(pool)
                    .await?;
                }
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let entry = db.samples.entry(run_id.to_string()).or_default();
                entry.extend(ballots.iter().cloned());
                Ok(())
            }
        }
    }

    pub async fn list_samples(&self, run_id: &str) -> Result<Vec<SampledBallot>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<SampleTuple> = sqlx::query_as(
                    "SELECT ballot_id, batch_id, position, manifest_entry, reported_vote, audited_vote \
                     FROM rla_sampled_ballots WHERE run_id = $1 ORDER BY batch_id, position",
                )
                .bind(run_id)
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(
                            ballot_id,
                            batch_id,
                            position,
                            manifest_entry,
                            reported_vote,
                            audited_vote,
                        )| SampledBallot {
                            ballot_id,
                            batch_id,
                            position: position as u32,
                            manifest_entry,
                            reported_vote,
                            audited_vote,
                        },
                    )
                    .collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                let mut rows = db.samples.get(run_id).cloned().unwrap_or_default();
                rows.sort_by(|a, b| (&a.batch_id, a.position).cmp(&(&b.batch_id, b.position)));
                Ok(rows)
            }
        }
    }

    pub async fn get_sample(
        &self,
        run_id: &str,
        ballot_id: &str,
    ) -> Result<Option<SampledBallot>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<SampleTuple> = sqlx::query_as(
                    "SELECT ballot_id, batch_id, position, manifest_entry, reported_vote, audited_vote \
                     FROM rla_sampled_ballots WHERE run_id = $1 AND ballot_id = $2",
                )
                .bind(run_id)
                .bind(ballot_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(
                        ballot_id,
                        batch_id,
                        position,
                        manifest_entry,
                        reported_vote,
                        audited_vote,
                    )| SampledBallot {
                        ballot_id,
                        batch_id,
                        position: position as u32,
                        manifest_entry,
                        reported_vote,
                        audited_vote,
                    },
                ))
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                Ok(db
                    .samples
                    .get(run_id)
                    .and_then(|s| s.iter().find(|b| b.ballot_id == ballot_id))
                    .cloned())
            }
        }
    }

    /// Record the human interpretation of one sampled ballot. A ballot may be
    /// interpreted exactly once; a second attempt is `DuplicateSampleInterpretation`
    /// regardless of whether the new reading agrees with the first.
    pub async fn record_interpretation(
        &self,
        run_id: &str,
        ballot_id: &str,
        audited_vote: &str,
    ) -> Result<SampledBallot, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<InterpretedTuple> = sqlx::query_as(
                    "UPDATE rla_sampled_ballots SET audited_vote = $3 \
                     WHERE run_id = $1 AND ballot_id = $2 AND audited_vote IS NULL \
                     RETURNING batch_id, position, manifest_entry, reported_vote, audited_vote",
                )
                .bind(run_id)
                .bind(ballot_id)
                .bind(audited_vote)
                .fetch_optional(pool)
                .await?;
                if let Some((batch_id, position, manifest_entry, reported_vote, audited)) = row {
                    return Ok(SampledBallot {
                        ballot_id: ballot_id.to_string(),
                        batch_id,
                        position: position as u32,
                        manifest_entry,
                        reported_vote,
                        audited_vote: audited,
                    });
                }
                match self.get_sample(run_id, ballot_id).await? {
                    Some(_) => Err(StoreError::DuplicateSampleInterpretation),
                    None => Err(StoreError::NotFound),
                }
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let ballot = db
                    .samples
                    .get_mut(run_id)
                    .and_then(|s| s.iter_mut().find(|b| b.ballot_id == ballot_id))
                    .ok_or(StoreError::NotFound)?;
                if ballot.audited_vote.is_some() {
                    return Err(StoreError::DuplicateSampleInterpretation);
                }
                ballot.audited_vote = Some(audited_vote.to_string());
                Ok(ballot.clone())
            }
        }
    }

    pub async fn save_entropy_source(&self, source: &EntropySource) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO rla_entropy_sources \
                     (source_id, beacon_uri, pulse_index, output_value, fetched_at) \
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(&source.source_id)
                .bind(&source.beacon_uri)
                .bind(source.pulse_index as i64)
                .bind(&source.output_value)
                .bind(&source.fetched_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                db.entropy.insert(source.source_id.clone(), source.clone());
                Ok(())
            }
        }
    }

    pub async fn save_witness_record(&self, witness: &WitnessRecord) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO rla_witness_records \
                     (witness_id, run_id, witness_name, affiliation, signature, recorded_at) \
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(&witness.witness_id)
                .bind(&witness.run_id)
                .bind(&witness.witness_name)
                .bind(&witness.affiliation)
                .bind(&witness.signature)
                .bind(&witness.recorded_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                db.witnesses
                    .insert(witness.witness_id.clone(), witness.clone());
                Ok(())
            }
        }
    }

    /// Store the final report for a run; one report per run (the finalise
    /// gate guarantees this is only reached once per run).
    pub async fn save_report(&self, report: &AuditReport) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO rla_audit_reports \
                     (report_id, run_id, risk_achieved, ballots_audited, outcome, generated_at) \
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(&report.report_id)
                .bind(&report.run_id)
                .bind(report.risk_achieved)
                .bind(report.ballots_audited as i32)
                .bind(&report.outcome)
                .bind(&report.generated_at)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                db.reports.insert(report.run_id.clone(), report.clone());
                Ok(())
            }
        }
    }

    pub async fn get_report(&self, run_id: &str) -> Result<Option<AuditReport>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<(Uuid, f64, i32, String, String)> = sqlx::query_as(
                    "SELECT report_id, risk_achieved, ballots_audited, outcome, generated_at \
                     FROM rla_audit_reports WHERE run_id = $1",
                )
                .bind(run_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(report_id, risk_achieved, ballots_audited, outcome, generated_at)| {
                        AuditReport {
                            report_id: report_id.to_string(),
                            run_id: run_id.to_string(),
                            risk_achieved,
                            ballots_audited: ballots_audited as u32,
                            outcome,
                            generated_at,
                        }
                    },
                ))
            }
            Store::Mem(mem) => Ok(mem.lock().reports.get(run_id).cloned()),
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory fake
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MemData {
    runs: HashMap<String, RunRow>,
    samples: HashMap<String, Vec<SampledBallot>>,
    entropy: HashMap<String, EntropySource>,
    witnesses: HashMap<String, WitnessRecord>,
    reports: HashMap<String, AuditReport>,
}

#[derive(Clone, Default)]
pub struct MemStore(Arc<std::sync::Mutex<MemData>>);

impl MemStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, MemData> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
