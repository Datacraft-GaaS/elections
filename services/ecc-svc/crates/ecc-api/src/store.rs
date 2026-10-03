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

//! Persistence for speech integrity (M17) and Section 44A confirmations.
//!
//! [`Store`] is an enum over the two backends so handlers stay dyn-free:
//! `Pg` runs sqlx runtime queries against the ecc-svc schema
//! (`migrations/V1__speech_integrity.sql`, `V2__s44a_confirmations.sql`);
//! `Mem` is an in-process fake used by hermetic tests and mirrors exactly
//! the same state transitions and uniqueness semantics.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct MediaItemRow {
    pub item_id: Uuid,
    pub url: String,
    pub url_sha256: String,
    pub media_type: String,
    pub language_code: String,
    pub source_platform: String,
    pub captured_at: DateTime<Utc>,
    pub status: String,
    /// Highest-confidence finding category once analysis has run; None while queued.
    pub top_category: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FindingRow {
    pub category: String,
    pub confidence: f64,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdjudicationRow {
    pub adjudication_id: Uuid,
    pub item_id: Uuid,
    pub verdict: String,
    pub referral_agency: String,
    pub analyst_notes: Option<String>,
    pub decided_at: DateTime<Utc>,
}

/// Aggregate counts backing GET /speech/reports/summary.
#[derive(Debug, Default, Clone)]
pub struct SummaryCounts {
    pub total_items: i64,
    pub by_status: HashMap<String, i64>,
    pub by_category: HashMap<String, i64>,
    pub by_verdict: HashMap<String, i64>,
    pub referrals_by_agency: HashMap<String, i64>,
    pub earliest_submitted_at: Option<DateTime<Utc>>,
    pub latest_submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct S44aRow {
    pub confirmation_id: Uuid,
    pub county_code: String,
    pub station_code: String,
    pub officer1_id: String,
    pub officer1_confirmed_at: DateTime<Utc>,
    pub officer2_id: Option<String>,
    pub officer2_confirmed_at: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct S44aCountyStatus {
    pub total_stations: i64,
    pub officer1_confirmed: i64,
    pub complete: i64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("media item with the same content hash already exists")]
    DuplicateMediaItem,
    #[error("media item already adjudicated")]
    AdjudicationExists,
    #[error("s44a confirmation already exists for station")]
    S44aConfirmationExists,
    #[error("no s44a confirmation awaiting officer2 for station")]
    S44aNotAwaitingOfficer2,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum Store {
    Pg(PgPool),
    Mem(MemStore),
}

/// Fingerprint snapshot stored per (operation, Idempotency-Key).
pub type Snapshot = (String, i32, serde_json::Value);

impl Store {
    pub async fn lookup_fingerprint(
        &self,
        operation: &str,
        key: &str,
    ) -> Result<Option<Snapshot>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<(String, i32, serde_json::Value)> = sqlx::query_as(
                    "SELECT request_sha256, response_status, response_body \
                     FROM ecc_request_fingerprints WHERE operation = $1 AND idempotency_key = $2",
                )
                .bind(operation)
                .bind(key)
                .fetch_optional(pool)
                .await?;
                Ok(row)
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .fingerprints
                .get(&(operation.to_string(), key.to_string()))
                .cloned()),
        }
    }

    pub async fn save_fingerprint(
        &self,
        fingerprint_id: Uuid,
        operation: &str,
        key: &str,
        request_sha256: &str,
        status: i32,
        body: serde_json::Value,
    ) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                sqlx::query(
                    "INSERT INTO ecc_request_fingerprints \
                     (fingerprint_id, operation, idempotency_key, request_sha256, response_status, response_body) \
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(fingerprint_id)
                .bind(operation)
                .bind(key)
                .bind(request_sha256)
                .bind(status)
                .bind(body)
                .execute(pool)
                .await?;
                Ok(())
            }
            Store::Mem(mem) => {
                mem.lock().fingerprints.insert(
                    (operation.to_string(), key.to_string()),
                    (request_sha256.to_string(), status, body),
                );
                Ok(())
            }
        }
    }

    /// Insert a media item. Returns `DuplicateMediaItem` when the same URL
    /// content hash was already ingested (contract 409).
    pub async fn insert_media_item(&self, row: &MediaItemRow) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let result = sqlx::query(
                    "INSERT INTO speech_media_items \
                     (item_id, url, url_sha256, media_type, language_code, source_platform, captured_at, status, submitted_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
                     ON CONFLICT (url_sha256) DO NOTHING",
                )
                .bind(row.item_id)
                .bind(&row.url)
                .bind(&row.url_sha256)
                .bind(&row.media_type)
                .bind(&row.language_code)
                .bind(&row.source_platform)
                .bind(row.captured_at)
                .bind(&row.status)
                .bind(row.submitted_at)
                .execute(pool)
                .await?;
                if result.rows_affected() == 0 {
                    Err(StoreError::DuplicateMediaItem)
                } else {
                    Ok(())
                }
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                if db
                    .media_items
                    .iter()
                    .any(|m| m.url_sha256 == row.url_sha256)
                {
                    return Err(StoreError::DuplicateMediaItem);
                }
                db.media_items.push(row.clone());
                Ok(())
            }
        }
    }

    pub async fn fetch_media_item(
        &self,
        item_id: Uuid,
    ) -> Result<Option<MediaItemRow>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<(Uuid, String, String, String, String, Option<String>, DateTime<Utc>, String, DateTime<Utc>)> =
                    sqlx::query_as(
                        "SELECT item_id, url, url_sha256, media_type, language_code, top_category, captured_at, status, submitted_at \
                         FROM speech_media_items WHERE item_id = $1",
                    )
                    .bind(item_id)
                    .fetch_optional(pool)
                    .await?;
                Ok(row.map(
                    |(
                        item_id,
                        url,
                        url_sha256,
                        media_type,
                        language_code,
                        top_category,
                        captured_at,
                        status,
                        submitted_at,
                    )| MediaItemRow {
                        item_id,
                        url,
                        url_sha256,
                        media_type,
                        language_code,
                        source_platform: String::new(),
                        captured_at,
                        status,
                        top_category,
                        submitted_at,
                    },
                ))
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .media_items
                .iter()
                .find(|m| m.item_id == item_id)
                .cloned()),
        }
    }

    pub async fn list_media_items(
        &self,
        status: Option<&str>,
        media_type: Option<&str>,
        from_date: Option<chrono::NaiveDate>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MediaItemRow>, i64), StoreError> {
        match self {
            Store::Pg(pool) => {
                // Runtime-composed filters; every dynamic value is bound, never inlined.
                let mut sql = String::from(
                    "SELECT item_id, media_type, language_code, source_platform, status, top_category, captured_at, submitted_at, \
                     COUNT(*) OVER () AS total FROM speech_media_items WHERE TRUE",
                );
                let mut idx = 1;
                if status.is_some() {
                    sql.push_str(&format!(" AND status = ${idx}"));
                    idx += 1;
                }
                if media_type.is_some() {
                    sql.push_str(&format!(" AND media_type = ${idx}"));
                    idx += 1;
                }
                if from_date.is_some() {
                    sql.push_str(&format!(" AND submitted_at >= ${idx}::date"));
                    idx += 1;
                }
                sql.push_str(&format!(
                    " ORDER BY submitted_at DESC LIMIT ${idx} OFFSET ${}",
                    idx + 1
                ));
                let mut q = sqlx::query_as::<
                    _,
                    (
                        Uuid,
                        String,
                        String,
                        String,
                        String,
                        Option<String>,
                        DateTime<Utc>,
                        DateTime<Utc>,
                        i64,
                    ),
                >(&sql);
                if let Some(v) = status {
                    q = q.bind(v);
                }
                if let Some(v) = media_type {
                    q = q.bind(v);
                }
                if let Some(v) = from_date {
                    q = q.bind(v);
                }
                q = q.bind(limit).bind(offset);
                let rows = q.fetch_all(pool).await?;
                let total = rows.first().map(|r| r.8).unwrap_or(0);
                let items = rows
                    .into_iter()
                    .map(
                        |(
                            item_id,
                            media_type,
                            language_code,
                            source_platform,
                            status,
                            top_category,
                            captured_at,
                            submitted_at,
                            _total,
                        )| MediaItemRow {
                            item_id,
                            url: String::new(),
                            url_sha256: String::new(),
                            media_type,
                            language_code,
                            source_platform,
                            captured_at,
                            status,
                            top_category,
                            submitted_at,
                        },
                    )
                    .collect();
                Ok((items, total))
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                let mut items: Vec<MediaItemRow> = db
                    .media_items
                    .iter()
                    .filter(|m| status.map_or(true, |s| m.status == s))
                    .filter(|m| media_type.map_or(true, |t| m.media_type == t))
                    .filter(|m| from_date.map_or(true, |d| m.submitted_at.date_naive() >= d))
                    .cloned()
                    .collect();
                items.sort_by(|a, b| b.submitted_at.cmp(&a.submitted_at));
                let total = items.len() as i64;
                let start = offset.max(0) as usize;
                let page: Vec<MediaItemRow> = items
                    .into_iter()
                    .skip(start)
                    .take(limit.max(0) as usize)
                    .collect();
                Ok((page, total))
            }
        }
    }

    pub async fn fetch_findings(&self, item_id: Uuid) -> Result<Vec<FindingRow>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<(String, f64, Option<String>)> = sqlx::query_as(
                    "SELECT category, confidence, excerpt FROM speech_analysis_findings \
                     WHERE item_id = $1 ORDER BY created_at",
                )
                .bind(item_id)
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(|(category, confidence, excerpt)| FindingRow {
                        category,
                        confidence,
                        excerpt,
                    })
                    .collect())
            }
            Store::Mem(mem) => Ok(mem
                .lock()
                .findings
                .get(&item_id)
                .cloned()
                .unwrap_or_default()),
        }
    }

    /// Record an adjudication and transition the item status atomically.
    /// Returns `AdjudicationExists` when the item already has a verdict.
    pub async fn record_adjudication(
        &self,
        row: &AdjudicationRow,
        grounds: &str,
        next_status: &str,
    ) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let mut tx = pool.begin().await?;
                // Lock the parent row to serialise concurrent adjudications.
                let existing: Option<(Uuid,)> =
                    sqlx::query_as("SELECT adjudication_id FROM speech_adjudications WHERE item_id = $1 FOR UPDATE")
                        .bind(row.item_id)
                        .fetch_optional(&mut *tx)
                        .await?;
                if existing.is_some() {
                    return Err(StoreError::AdjudicationExists);
                }
                sqlx::query(
                    "INSERT INTO speech_adjudications \
                     (adjudication_id, item_id, verdict, grounds, referral_agency, analyst_notes, decided_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                )
                .bind(row.adjudication_id)
                .bind(row.item_id)
                .bind(&row.verdict)
                .bind(grounds)
                .bind(&row.referral_agency)
                .bind(&row.analyst_notes)
                .bind(row.decided_at)
                .execute(&mut *tx)
                .await?;
                sqlx::query("UPDATE speech_media_items SET status = $2 WHERE item_id = $1")
                    .bind(row.item_id)
                    .bind(next_status)
                    .execute(&mut *tx)
                    .await?;
                tx.commit().await?;
                Ok(())
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                if db.adjudications.contains_key(&row.item_id) {
                    return Err(StoreError::AdjudicationExists);
                }
                db.adjudications.insert(row.item_id, row.clone());
                if let Some(item) = db.media_items.iter_mut().find(|m| m.item_id == row.item_id) {
                    item.status = next_status.to_string();
                }
                Ok(())
            }
        }
    }

    pub async fn fetch_adjudication(
        &self,
        item_id: Uuid,
    ) -> Result<Option<AdjudicationRow>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<(Uuid, Uuid, String, String, Option<String>, DateTime<Utc>)> = sqlx::query_as(
                    "SELECT adjudication_id, item_id, verdict, referral_agency, analyst_notes, decided_at \
                     FROM speech_adjudications WHERE item_id = $1",
                )
                .bind(item_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(
                        adjudication_id,
                        item_id,
                        verdict,
                        referral_agency,
                        analyst_notes,
                        decided_at,
                    )| AdjudicationRow {
                        adjudication_id,
                        item_id,
                        verdict,
                        referral_agency,
                        analyst_notes,
                        decided_at,
                    },
                ))
            }
            Store::Mem(mem) => Ok(mem.lock().adjudications.get(&item_id).cloned()),
        }
    }

    pub async fn summary_counts(
        &self,
        from: Option<chrono::NaiveDate>,
        to: Option<chrono::NaiveDate>,
    ) -> Result<SummaryCounts, StoreError> {
        match self {
            Store::Pg(pool) => {
                let window = "WHERE ($1::date IS NULL OR submitted_at >= $1::date) \
                              AND ($2::date IS NULL OR submitted_at < ($2::date + INTERVAL '1 day'))";
                let by_status_rows: Vec<(String, i64)> = sqlx::query_as(&format!(
                    "SELECT status, COUNT(*) FROM speech_media_items {window} GROUP BY status"
                ))
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await?;
                let by_category_rows: Vec<(String, i64)> = sqlx::query_as(&format!(
                    "SELECT f.category, COUNT(DISTINCT f.item_id) \
                     FROM speech_analysis_findings f \
                     JOIN speech_media_items m ON m.item_id = f.item_id {window} \
                     GROUP BY f.category"
                ))
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await?;
                let by_verdict_rows: Vec<(String, i64)> = sqlx::query_as(&format!(
                    "SELECT a.verdict, COUNT(*) FROM speech_adjudications a \
                     JOIN speech_media_items m ON m.item_id = a.item_id {window} GROUP BY a.verdict"
                ))
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await?;
                let referrals_rows: Vec<(String, i64)> = sqlx::query_as(&format!(
                    "SELECT a.referral_agency, COUNT(*) FROM speech_adjudications a \
                     JOIN speech_media_items m ON m.item_id = a.item_id {window} \
                     GROUP BY a.referral_agency HAVING a.referral_agency <> 'none'"
                ))
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await?;
                let bounds_row: (Option<DateTime<Utc>>, Option<DateTime<Utc>>) =
                    sqlx::query_as(&format!(
                    "SELECT MIN(submitted_at), MAX(submitted_at) FROM speech_media_items {window}"
                ))
                    .bind(from)
                    .bind(to)
                    .fetch_one(pool)
                    .await?;
                let bounds = match bounds_row {
                    (lo, hi) if lo.is_some() || hi.is_some() => Some((lo, hi)),
                    _ => None,
                };
                Ok(SummaryCounts {
                    total_items: by_status_rows.iter().map(|(_, c)| c).sum(),
                    by_status: by_status_rows.into_iter().collect(),
                    by_category: by_category_rows.into_iter().collect(),
                    by_verdict: by_verdict_rows.into_iter().collect(),
                    referrals_by_agency: referrals_rows.into_iter().collect(),
                    earliest_submitted_at: bounds.and_then(|(lo, _)| lo),
                    latest_submitted_at: bounds.and_then(|(_, hi)| hi),
                })
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                let in_window = |m: &MediaItemRow| {
                    from.map_or(true, |f| m.submitted_at.date_naive() >= f)
                        && to.map_or(true, |t| m.submitted_at.date_naive() <= t)
                };
                let mut counts = SummaryCounts::default();
                for m in db.media_items.iter().filter(|m| in_window(m)) {
                    *counts.by_status.entry(m.status.clone()).or_insert(0) += 1;
                    counts.total_items += 1;
                    counts.earliest_submitted_at = Some(match counts.earliest_submitted_at {
                        Some(lo) => lo.min(m.submitted_at),
                        None => m.submitted_at,
                    });
                    counts.latest_submitted_at = Some(match counts.latest_submitted_at {
                        Some(hi) => hi.max(m.submitted_at),
                        None => m.submitted_at,
                    });
                }
                for (item_id, findings) in db.findings.iter() {
                    let Some(m) = db.media_items.iter().find(|m| m.item_id == *item_id) else {
                        continue;
                    };
                    if !in_window(m) {
                        continue;
                    }
                    for f in findings {
                        *counts.by_category.entry(f.category.clone()).or_insert(0) += 1;
                    }
                }
                for (item_id, a) in db.adjudications.iter() {
                    let Some(m) = db.media_items.iter().find(|m| m.item_id == *item_id) else {
                        continue;
                    };
                    if !in_window(m) {
                        continue;
                    }
                    *counts.by_verdict.entry(a.verdict.clone()).or_insert(0) += 1;
                    if a.referral_agency != "none" {
                        *counts
                            .referrals_by_agency
                            .entry(a.referral_agency.clone())
                            .or_insert(0) += 1;
                    }
                }
                Ok(counts)
            }
        }
    }

    // -- Section 44A -------------------------------------------------------

    pub async fn insert_s44a_officer1(&self, row: &S44aRow) -> Result<(), StoreError> {
        match self {
            Store::Pg(pool) => {
                let result = sqlx::query(
                    "INSERT INTO s44a_confirmations \
                     (confirmation_id, county_code, station_code, officer1_id, officer1_confirmed_at, status) \
                     VALUES ($1, $2, $3, $4, $5, $6) \
                     ON CONFLICT (station_code) DO NOTHING",
                )
                .bind(row.confirmation_id)
                .bind(&row.county_code)
                .bind(&row.station_code)
                .bind(&row.officer1_id)
                .bind(row.officer1_confirmed_at)
                .bind(&row.status)
                .execute(pool)
                .await?;
                if result.rows_affected() == 0 {
                    Err(StoreError::S44aConfirmationExists)
                } else {
                    Ok(())
                }
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                if db.s44a.iter().any(|c| c.station_code == row.station_code) {
                    return Err(StoreError::S44aConfirmationExists);
                }
                db.s44a.push(row.clone());
                Ok(())
            }
        }
    }

    pub async fn complete_s44a_officer2(
        &self,
        station_code: &str,
        officer2_id: &str,
        confirmed_at: DateTime<Utc>,
    ) -> Result<Option<S44aRow>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: Option<(Uuid, String, String, String, DateTime<Utc>, Option<String>, Option<DateTime<Utc>>, String)> =
                    sqlx::query_as(
                        "UPDATE s44a_confirmations \
                         SET officer2_id = $2, officer2_confirmed_at = $3, status = 'complete', updated_at = NOW() \
                         WHERE station_code = $1 AND status = 'officer1_confirmed' \
                         RETURNING confirmation_id, county_code, station_code, officer1_id, \
                                   officer1_confirmed_at, officer2_id, officer2_confirmed_at, status",
                    )
                    .bind(station_code)
                    .bind(officer2_id)
                    .bind(confirmed_at)
                    .fetch_optional(pool)
                    .await?;
                Ok(row.map(s44a_from_tuple))
            }
            Store::Mem(mem) => {
                let mut db = mem.lock();
                let entry = db
                    .s44a
                    .iter_mut()
                    .find(|c| c.station_code == station_code && c.status == "officer1_confirmed")
                    .ok_or(StoreError::S44aNotAwaitingOfficer2)?;
                entry.officer2_id = Some(officer2_id.to_string());
                entry.officer2_confirmed_at = Some(confirmed_at);
                entry.status = "complete".to_string();
                Ok(Some(entry.clone()))
            }
        }
    }

    pub async fn list_s44a_by_county(&self, county_code: &str) -> Result<Vec<S44aRow>, StoreError> {
        match self {
            Store::Pg(pool) => {
                let rows: Vec<(Uuid, String, String, String, DateTime<Utc>, Option<String>, Option<DateTime<Utc>>, String)> =
                    sqlx::query_as(
                        "SELECT confirmation_id, county_code, station_code, officer1_id, \
                                officer1_confirmed_at, officer2_id, officer2_confirmed_at, status \
                         FROM s44a_confirmations WHERE county_code = $1 ORDER BY officer1_confirmed_at DESC",
                    )
                    .bind(county_code)
                    .fetch_all(pool)
                    .await?;
                Ok(rows.into_iter().map(s44a_from_tuple).collect())
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                let mut rows: Vec<S44aRow> = db
                    .s44a
                    .iter()
                    .filter(|c| c.county_code == county_code)
                    .cloned()
                    .collect();
                rows.sort_by(|a, b| b.officer1_confirmed_at.cmp(&a.officer1_confirmed_at));
                Ok(rows)
            }
        }
    }

    pub async fn s44a_county_status(
        &self,
        county_code: &str,
    ) -> Result<S44aCountyStatus, StoreError> {
        match self {
            Store::Pg(pool) => {
                let row: (i64, i64, i64) = sqlx::query_as(
                    "SELECT COUNT(*), \
                            COUNT(*) FILTER (WHERE status = 'officer1_confirmed'), \
                            COUNT(*) FILTER (WHERE status = 'complete') \
                     FROM s44a_confirmations WHERE county_code = $1",
                )
                .bind(county_code)
                .fetch_one(pool)
                .await?;
                Ok(S44aCountyStatus {
                    total_stations: row.0,
                    officer1_confirmed: row.1,
                    complete: row.2,
                })
            }
            Store::Mem(mem) => {
                let db = mem.lock();
                let rows: Vec<&S44aRow> = db
                    .s44a
                    .iter()
                    .filter(|c| c.county_code == county_code)
                    .collect();
                Ok(S44aCountyStatus {
                    total_stations: rows.len() as i64,
                    officer1_confirmed: rows
                        .iter()
                        .filter(|c| c.status == "officer1_confirmed")
                        .count() as i64,
                    complete: rows.iter().filter(|c| c.status == "complete").count() as i64,
                })
            }
        }
    }
}

fn s44a_from_tuple(
    t: (
        Uuid,
        String,
        String,
        String,
        DateTime<Utc>,
        Option<String>,
        Option<DateTime<Utc>>,
        String,
    ),
) -> S44aRow {
    S44aRow {
        confirmation_id: t.0,
        county_code: t.1,
        station_code: t.2,
        officer1_id: t.3,
        officer1_confirmed_at: t.4,
        officer2_id: t.5,
        officer2_confirmed_at: t.6,
        status: t.7,
    }
}

// ---------------------------------------------------------------------------
// In-memory fake
// ---------------------------------------------------------------------------

#[derive(Default)]
struct MemData {
    fingerprints: HashMap<(String, String), Snapshot>,
    media_items: Vec<MediaItemRow>,
    findings: HashMap<Uuid, Vec<FindingRow>>,
    adjudications: HashMap<Uuid, AdjudicationRow>,
    s44a: Vec<S44aRow>,
}

#[derive(Clone, Default)]
pub struct MemStore(std::sync::Arc<Mutex<MemData>>);

impl MemStore {
    fn lock(&self) -> std::sync::MutexGuard<'_, MemData> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
