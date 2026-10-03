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

//! Postgres persistence for custody API surface (TD-03-P0-01 — no in-memory).
//!
//! `Store` is an enum over `Pg` (sqlx) and `Mem` (hermetic tests). Handlers
//! are `Store`-agnostic; production wires `Pg(PgPool)` via `AppState`.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::spec::{
    Asset, AssetCreate, CustodyTransfer, CustodyTransferCreate, Discrepancy, ReconcileCreate,
    ReconcileReport, Reserve, Seal, SealBreak, SealBreakCreate, SealCreate,
};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

// ---------------------------------------------------------------------------
// Store enum
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub enum Store {
    Pg(PgPool),
    Mem(MemStore),
}

#[derive(Clone, Default)]
pub struct MemStore {
    pub assets: Arc<eel_spec_helpers_rs::SharedMap<String, Asset>>,
    pub transfers: Arc<eel_spec_helpers_rs::SharedMap<String, Vec<CustodyTransfer>>>,
    pub seals: Arc<eel_spec_helpers_rs::SharedMap<String, Seal>>,
    pub seal_breaks: Arc<eel_spec_helpers_rs::SharedMap<String, Vec<SealBreak>>>,
    pub reconciles: Arc<eel_spec_helpers_rs::SharedMap<String, ReconcileReport>>,
}

impl Store {
    pub fn pg(pool: PgPool) -> Self {
        Self::Pg(pool)
    }
    pub fn mem() -> Self {
        Self::Mem(MemStore::default())
    }

    // -----------------------------------------------------------------------
    // Assets
    // -----------------------------------------------------------------------

    pub async fn create_asset(&self, req: AssetCreate) -> Result<Asset, StoreError> {
        match self {
            Self::Pg(pool) => {
                let id = Uuid::now_v7();
                let now: DateTime<Utc> = Utc::now();
                sqlx::query(
                    r#"INSERT INTO custody_api_assets
                       (asset_id, asset_type, serial_number, current_custodian_id, current_location, status, condition, created_at, updated_at)
                       VALUES ($1,$2,$3,$4,$5,'in_custody',$6,$7,$7)"#,
                )
                .bind(id)
                .bind(&req.asset_type)
                .bind(&req.serial_number)
                .bind(&req.current_custodian_id)
                .bind(&req.current_location)
                .bind(&req.condition)
                .bind(now)
                .execute(pool)
                .await?;
                Ok(Asset {
                    asset_id: id.to_string(),
                    asset_type: req.asset_type,
                    serial_number: req.serial_number,
                    current_custodian_id: req.current_custodian_id,
                    current_location: req.current_location,
                    status: "in_custody".to_string(),
                    condition: req.condition,
                    created_at: now.to_rfc3339(),
                    updated_at: now.to_rfc3339(),
                })
            }
            Self::Mem(m) => {
                let now = Utc::now().to_rfc3339();
                let asset = Asset {
                    asset_id: Uuid::now_v7().to_string(),
                    asset_type: req.asset_type,
                    serial_number: req.serial_number,
                    current_custodian_id: req.current_custodian_id,
                    current_location: req.current_location,
                    status: "in_custody".to_string(),
                    condition: req.condition,
                    created_at: now.clone(),
                    updated_at: now,
                };
                m.assets.insert(asset.asset_id.clone(), asset.clone());
                Ok(asset)
            }
        }
    }

    pub async fn list_assets(&self) -> Result<Vec<Asset>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rows = sqlx::query_as::<_, (Uuid, String, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>)>(
                    "SELECT asset_id, asset_type, serial_number, current_custodian_id, current_location, status, condition, created_at, updated_at FROM custody_api_assets ORDER BY created_at",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(id, asset_type, serial, cust, loc, status, cond, created, updated)| {
                            Asset {
                                asset_id: id.to_string(),
                                asset_type,
                                serial_number: serial,
                                current_custodian_id: cust,
                                current_location: loc,
                                status,
                                condition: cond,
                                created_at: created.to_rfc3339(),
                                updated_at: updated.to_rfc3339(),
                            }
                        },
                    )
                    .collect())
            }
            Self::Mem(m) => {
                let mut v = m.assets.values();
                v.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));
                Ok(v)
            }
        }
    }

    pub async fn get_asset(&self, id: &str) -> Result<Option<Asset>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let uuid = Uuid::parse_str(id).map_err(|_| StoreError::NotFound)?;
                let row = sqlx::query_as::<_, (Uuid, String, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>)>(
                    "SELECT asset_id, asset_type, serial_number, current_custodian_id, current_location, status, condition, created_at, updated_at FROM custody_api_assets WHERE asset_id=$1",
                )
                .bind(uuid)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(
                    |(id, asset_type, serial, cust, loc, status, cond, created, updated)| Asset {
                        asset_id: id.to_string(),
                        asset_type,
                        serial_number: serial,
                        current_custodian_id: cust,
                        current_location: loc,
                        status,
                        condition: cond,
                        created_at: created.to_rfc3339(),
                        updated_at: updated.to_rfc3339(),
                    },
                ))
            }
            Self::Mem(m) => Ok(m.assets.get(&id.to_string())),
        }
    }

    pub async fn transfer(
        &self,
        asset_id: &str,
        req: CustodyTransferCreate,
    ) -> Result<CustodyTransfer, StoreError> {
        match self {
            Self::Pg(pool) => {
                let asset_uuid = Uuid::parse_str(asset_id).map_err(|_| StoreError::NotFound)?;
                let asset = self
                    .get_asset(asset_id)
                    .await?
                    .ok_or(StoreError::NotFound)?;
                let tid = Uuid::now_v7();
                let now = Utc::now();
                sqlx::query(
                    "INSERT INTO custody_api_transfers (transfer_id, asset_id, from_custodian_id, to_custodian_id, from_location, to_location, reason, transferred_at, officer_id, witness_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
                )
                .bind(tid)
                .bind(asset_uuid)
                .bind(&asset.current_custodian_id)
                .bind(&req.to_custodian_id)
                .bind(&asset.current_location)
                .bind(&req.to_location)
                .bind(&req.reason)
                .bind(now)
                .bind(&req.officer_id)
                .bind(&req.witness_id)
                .execute(pool)
                .await?;
                sqlx::query(
                    "UPDATE custody_api_assets SET current_custodian_id=$1, current_location=$2, status='in_transit', updated_at=$3 WHERE asset_id=$4",
                )
                .bind(&req.to_custodian_id)
                .bind(&req.to_location)
                .bind(now)
                .bind(asset_uuid)
                .execute(pool)
                .await?;
                Ok(CustodyTransfer {
                    transfer_id: tid.to_string(),
                    asset_id: asset_id.to_string(),
                    from_custodian_id: asset.current_custodian_id,
                    to_custodian_id: req.to_custodian_id,
                    from_location: asset.current_location,
                    to_location: req.to_location,
                    reason: req.reason,
                    transferred_at: now.to_rfc3339(),
                    officer_id: req.officer_id,
                    witness_id: req.witness_id,
                })
            }
            Self::Mem(m) => {
                let asset = m
                    .assets
                    .get(&asset_id.to_string())
                    .ok_or(StoreError::NotFound)?;
                let t = CustodyTransfer {
                    transfer_id: Uuid::now_v7().to_string(),
                    asset_id: asset_id.to_string(),
                    from_custodian_id: asset.current_custodian_id.clone(),
                    to_custodian_id: req.to_custodian_id.clone(),
                    from_location: asset.current_location.clone(),
                    to_location: req.to_location.clone(),
                    reason: req.reason,
                    transferred_at: Utc::now().to_rfc3339(),
                    officer_id: req.officer_id,
                    witness_id: req.witness_id,
                };
                // update asset
                let mut updated = asset.clone();
                updated.current_custodian_id = t.to_custodian_id.clone();
                updated.current_location = t.to_location.clone();
                updated.status = "in_transit".to_string();
                updated.updated_at = Utc::now().to_rfc3339();
                m.assets.insert(asset_id.to_string(), updated);
                m.transfers
                    .with_lock(|map| map.entry(asset_id.to_string()).or_default().push(t.clone()));
                Ok(t)
            }
        }
    }

    pub async fn asset_history(&self, asset_id: &str) -> Result<Vec<CustodyTransfer>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let uuid = Uuid::parse_str(asset_id).map_err(|_| StoreError::NotFound)?;
                let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, String, String, Option<String>, DateTime<Utc>, String, Option<String>)>(
                    "SELECT transfer_id, asset_id, from_custodian_id, to_custodian_id, from_location, to_location, reason, transferred_at, officer_id, witness_id FROM custody_api_transfers WHERE asset_id=$1 ORDER BY transferred_at",
                )
                .bind(uuid)
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(tid, aid, from_c, to_c, from_l, to_l, reason, at, off, wit)| {
                            CustodyTransfer {
                                transfer_id: tid.to_string(),
                                asset_id: aid.to_string(),
                                from_custodian_id: from_c,
                                to_custodian_id: to_c,
                                from_location: from_l,
                                to_location: to_l,
                                reason: reason.unwrap_or_default(),
                                transferred_at: at.to_rfc3339(),
                                officer_id: off,
                                witness_id: wit,
                            }
                        },
                    )
                    .collect())
            }
            Self::Mem(m) => Ok(m.transfers.get(&asset_id.to_string()).unwrap_or_default()),
        }
    }

    // -----------------------------------------------------------------------
    // Seals
    // -----------------------------------------------------------------------

    pub async fn create_seal(&self, req: SealCreate) -> Result<Seal, StoreError> {
        match self {
            Self::Pg(pool) => {
                let asset_uuid =
                    Uuid::parse_str(&req.asset_id).map_err(|_| StoreError::NotFound)?;
                // ensure asset exists
                self.get_asset(&req.asset_id)
                    .await?
                    .ok_or(StoreError::NotFound)?;
                let sid = Uuid::now_v7();
                let now = Utc::now();
                sqlx::query(
                    "INSERT INTO custody_api_seals (seal_id, asset_id, seal_number, sealed_by_id, sealed_at, status) VALUES ($1,$2,$3,$4,$5,'intact')",
                )
                .bind(sid)
                .bind(asset_uuid)
                .bind(&req.seal_number)
                .bind(&req.sealed_by_id)
                .bind(now)
                .execute(pool)
                .await?;
                Ok(Seal {
                    seal_id: sid.to_string(),
                    asset_id: req.asset_id,
                    seal_number: req.seal_number,
                    sealed_by_id: req.sealed_by_id,
                    sealed_at: now.to_rfc3339(),
                    status: "intact".to_string(),
                })
            }
            Self::Mem(m) => {
                let seal = Seal {
                    seal_id: Uuid::now_v7().to_string(),
                    asset_id: req.asset_id,
                    seal_number: req.seal_number,
                    sealed_by_id: req.sealed_by_id,
                    sealed_at: Utc::now().to_rfc3339(),
                    status: "intact".to_string(),
                };
                m.seals.insert(seal.seal_id.clone(), seal.clone());
                Ok(seal)
            }
        }
    }

    pub async fn break_seal(
        &self,
        seal_id: &str,
        req: SealBreakCreate,
    ) -> Result<SealBreak, StoreError> {
        match self {
            Self::Pg(pool) => {
                let sid = Uuid::parse_str(seal_id).map_err(|_| StoreError::NotFound)?;
                let exists: Option<(Uuid,)> =
                    sqlx::query_as("SELECT seal_id FROM custody_api_seals WHERE seal_id=$1")
                        .bind(sid)
                        .fetch_optional(pool)
                        .await?;
                if exists.is_none() {
                    return Err(StoreError::NotFound);
                }
                sqlx::query("UPDATE custody_api_seals SET status='broken' WHERE seal_id=$1")
                    .bind(sid)
                    .execute(pool)
                    .await?;
                let bid = Uuid::now_v7();
                let now = Utc::now();
                sqlx::query(
                    "INSERT INTO custody_api_seal_breaks (break_id, seal_id, broken_by_id, countersign_id, reason, broken_at) VALUES ($1,$2,$3,$4,$5,$6)",
                )
                .bind(bid)
                .bind(sid)
                .bind(&req.broken_by_id)
                .bind(&req.countersign_id)
                .bind(&req.reason)
                .bind(now)
                .execute(pool)
                .await?;
                Ok(SealBreak {
                    break_id: bid.to_string(),
                    seal_id: seal_id.to_string(),
                    broken_by_id: req.broken_by_id,
                    countersign_id: req.countersign_id,
                    reason: req.reason,
                    broken_at: now.to_rfc3339(),
                })
            }
            Self::Mem(m) => {
                let ok = m.seals.with_lock(|map| {
                    if let Some(s) = map.get_mut(seal_id) {
                        s.status = "broken".to_string();
                        true
                    } else {
                        false
                    }
                });
                if !ok {
                    return Err(StoreError::NotFound);
                }
                let brk = SealBreak {
                    break_id: Uuid::now_v7().to_string(),
                    seal_id: seal_id.to_string(),
                    broken_by_id: req.broken_by_id,
                    countersign_id: req.countersign_id,
                    reason: req.reason,
                    broken_at: Utc::now().to_rfc3339(),
                };
                m.seal_breaks.with_lock(|map| {
                    map.entry(seal_id.to_string())
                        .or_default()
                        .push(brk.clone())
                });
                Ok(brk)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Reserves / summary / reconcile
    // -----------------------------------------------------------------------

    pub async fn list_reserves(&self) -> Result<Vec<Reserve>, StoreError> {
        match self {
            Self::Pg(pool) => {
                // custody_api_reserves does not have all Reserve fields; synthesize for contract
                let rows = sqlx::query_as::<_, (Uuid, String, i32, String, DateTime<Utc>)>(
                    "SELECT reserve_id, asset_type, count, location, created_at FROM custody_api_reserves ORDER BY created_at",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(|(id, at, cnt, loc, _)| Reserve {
                        reserve_id: id.to_string(),
                        asset_type: at,
                        location: loc,
                        quantity: cnt as u32,
                        min_threshold: 0,
                        status: "ok".to_string(),
                    })
                    .collect())
            }
            Self::Mem(_) => Ok(vec![]),
        }
    }

    pub async fn custody_summary(&self) -> Result<crate::spec::CustodySummary, StoreError> {
        match self {
            Self::Pg(pool) => {
                let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM custody_api_assets")
                    .fetch_one(pool)
                    .await?;
                let in_transit: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM custody_api_assets WHERE status='in_transit'",
                )
                .fetch_one(pool)
                .await?;
                let in_custody = total.0 - in_transit.0;
                Ok(crate::spec::CustodySummary {
                    total_assets: total.0 as u32,
                    assets_in_custody: in_custody as u32,
                    assets_in_transit: in_transit.0 as u32,
                    broken_seals_today: 0,
                    discrepancy_count: 0,
                    generated_at: Utc::now().to_rfc3339(),
                })
            }
            Self::Mem(m) => {
                let assets = m.assets.values();
                let total = assets.len() as u32;
                let in_transit = assets.iter().filter(|a| a.status == "in_transit").count() as u32;
                let in_custody = total - in_transit;
                Ok(crate::spec::CustodySummary {
                    total_assets: total,
                    assets_in_custody: in_custody,
                    assets_in_transit: in_transit,
                    broken_seals_today: 0,
                    discrepancy_count: 0,
                    generated_at: Utc::now().to_rfc3339(),
                })
            }
        }
    }

    pub async fn create_reconcile(
        &self,
        req: ReconcileCreate,
    ) -> Result<ReconcileReport, StoreError> {
        let status = if req.discrepancies.is_empty() {
            "reconciled"
        } else {
            "discrepancy_found"
        };
        match self {
            Self::Pg(pool) => {
                let rid = Uuid::now_v7();
                let discs =
                    serde_json::to_value(&req.discrepancies).unwrap_or(Value::Array(vec![]));
                sqlx::query(
                    "INSERT INTO custody_api_reconciles (reconcile_id, station_code, election_id, assets_expected, assets_found, discrepancies, status, officer_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
                )
                .bind(rid)
                .bind(&req.station_code)
                .bind(&req.election_id)
                .bind(req.assets_expected as i32)
                .bind(req.assets_found as i32)
                .bind(discs)
                .bind(status)
                .bind(&req.officer_id)
                .execute(pool)
                .await?;
                Ok(ReconcileReport {
                    reconcile_id: rid.to_string(),
                    station_code: req.station_code,
                    election_id: req.election_id,
                    assets_expected: req.assets_expected,
                    assets_found: req.assets_found,
                    discrepancies: req.discrepancies,
                    status: status.to_string(),
                    officer_id: req.officer_id,
                })
            }
            Self::Mem(m) => {
                let report = ReconcileReport {
                    reconcile_id: Uuid::now_v7().to_string(),
                    station_code: req.station_code,
                    election_id: req.election_id,
                    assets_expected: req.assets_expected,
                    assets_found: req.assets_found,
                    discrepancies: req.discrepancies,
                    status: status.to_string(),
                    officer_id: req.officer_id,
                };
                m.reconciles
                    .insert(report.reconcile_id.clone(), report.clone());
                Ok(report)
            }
        }
    }

    pub async fn get_reconcile(&self, id: &str) -> Result<Option<ReconcileReport>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let uuid = Uuid::parse_str(id).map_err(|_| StoreError::NotFound)?;
                let row = sqlx::query_as::<
                    _,
                    (Uuid, String, String, i32, i32, Value, String, String),
                >(
                    "SELECT reconcile_id, station_code, election_id, assets_expected, assets_found, discrepancies, status, officer_id FROM custody_api_reconciles WHERE reconcile_id=$1",
                )
                .bind(uuid)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(|(rid, sc, eid, exp, found, discs, status, off)| {
                    let discrepancies: Vec<Discrepancy> =
                        serde_json::from_value(discs).unwrap_or_default();
                    ReconcileReport {
                        reconcile_id: rid.to_string(),
                        station_code: sc,
                        election_id: eid,
                        assets_expected: exp as u32,
                        assets_found: found as u32,
                        discrepancies,
                        status,
                        officer_id: off,
                    }
                }))
            }
            Self::Mem(m) => Ok(m.reconciles.get(&id.to_string())),
        }
    }
}
