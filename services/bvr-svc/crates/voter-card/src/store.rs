#![allow(missing_docs)]
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

//! Postgres persistence for voter-card API surface (TD-03-P0-01 — no in-memory).
//!
//! `Store::Pg(PgPool)` is production (sqlx against `voter_card_api_*` tables,
//! `migrations/V055__voter_card_api.sql`). `Store::Mem` is hermetic tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::spec::{
    CollectionCentre, CollectionConfirmation, VoterCardRequest, VoterCardRequestCreate,
};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Clone, Default)]
pub struct MemStore {
    pub requests: Arc<Mutex<HashMap<String, VoterCardRequest>>>,
    pub centres: Arc<Mutex<HashMap<String, CollectionCentre>>>,
}

#[derive(Clone)]
pub enum Store {
    Pg(PgPool),
    Mem(MemStore),
}

impl Store {
    pub fn pg(pool: PgPool) -> Self {
        Self::Pg(pool)
    }
    pub fn mem() -> Self {
        Self::Mem(MemStore::default())
    }

    pub async fn create_request(
        &self,
        req: VoterCardRequestCreate,
    ) -> Result<VoterCardRequest, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rec = VoterCardRequest {
                    request_id: Uuid::now_v7().to_string(),
                    status: "submitted".to_string(),
                    submitted_at: Utc::now().to_rfc3339(),
                };
                sqlx::query(
                    r#"INSERT INTO voter_card_api_requests
                       (request_id, voter_id_hash, request_type, reason, preferred_collection_centre, status, submitted_at)
                       VALUES ($1,$2,$3,$4,$5,'submitted',$6)"#,
                )
                .bind(&rec.request_id)
                .bind(&req.voter_id_hash)
                .bind(&req.request_type)
                .bind(&req.reason)
                .bind(&req.preferred_collection_centre)
                .bind(rec.submitted_at.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now()))
                .execute(pool)
                .await?;
                Ok(rec)
            }
            Self::Mem(m) => {
                let rec = VoterCardRequest {
                    request_id: Uuid::now_v7().to_string(),
                    status: "submitted".to_string(),
                    submitted_at: Utc::now().to_rfc3339(),
                };
                m.requests
                    .lock()
                    .expect("lock poisoned")
                    .insert(rec.request_id.clone(), rec.clone());
                Ok(rec)
            }
        }
    }

    pub async fn get_request(&self, id: &str) -> Result<Option<VoterCardRequest>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let row = sqlx::query_as::<_, (String, String, String)>(
                    "SELECT request_id, status, submitted_at::text FROM voter_card_api_requests WHERE request_id=$1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(
                    row.map(|(request_id, status, submitted_at)| VoterCardRequest {
                        request_id,
                        status,
                        submitted_at,
                    }),
                )
            }
            Self::Mem(m) => Ok(m.requests.lock().expect("lock poisoned").get(id).cloned()),
        }
    }

    pub async fn confirm_collection(
        &self,
        id: &str,
        req: CollectionConfirmation,
    ) -> Result<Option<VoterCardRequest>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let res = sqlx::query(
                    "UPDATE voter_card_api_requests SET status='collected', collected_at=$1, collection_centre_code=$2, officer_hash=$3 WHERE request_id=$4",
                )
                .bind(req.collected_at.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now()))
                .bind(&req.collection_centre_code)
                .bind(&req.officer_hash)
                .bind(id)
                .execute(pool)
                .await?;
                if res.rows_affected() == 0 {
                    return Ok(None);
                }
                self.get_request(id).await
            }
            Self::Mem(m) => {
                let mut guard = m.requests.lock().expect("lock poisoned");
                if let Some(r) = guard.get_mut(id) {
                    r.status = "collected".to_string();
                    Ok(Some(r.clone()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn list_centres(&self) -> Result<Vec<CollectionCentre>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, bool)>(
                    "SELECT centre_code, name, county_code, address, operating_hours, open FROM voter_card_api_centres ORDER BY centre_code",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(centre_code, name, county_code, address, operating_hours, open)| {
                            CollectionCentre {
                                centre_code,
                                name,
                                county_code,
                                address,
                                operating_hours,
                                open,
                            }
                        },
                    )
                    .collect())
            }
            Self::Mem(m) => {
                let mut v: Vec<CollectionCentre> = m
                    .centres
                    .lock()
                    .expect("lock poisoned")
                    .values()
                    .cloned()
                    .collect();
                v.sort_by(|a, b| a.centre_code.cmp(&b.centre_code));
                Ok(v)
            }
        }
    }
}
