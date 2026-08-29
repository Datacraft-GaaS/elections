#![allow(missing_docs)]
//! Postgres persistence for printing API surface (TD-03-P0-01 — no in-memory).
//!
//! `Store::Pg(PgPool)` is production (sqlx against `printing_api_*` tables,
//! `migrations/V053__printing_api_surface.sql`). `Store::Mem` is hermetic
//! tests (SharedMap, mirrors custody pattern).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::spec::{AuditEvent, DeliveryRecord, DeliveryRecordRequest, PrintOrder, PrintOrderRequest, QCDecision};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Clone, Default)]
pub struct MemStore {
    pub orders: eel_spec_helpers_rs::SharedMap<Uuid, PrintOrder>,
    pub deliveries: eel_spec_helpers_rs::SharedMap<Uuid, DeliveryRecord>,
    pub audit: Arc<std::sync::Mutex<Vec<AuditEvent>>>,
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

    pub async fn create_order(&self, req: PrintOrderRequest) -> Result<PrintOrder, StoreError> {
        match self {
            Self::Pg(pool) => {
                let now = Utc::now();
                let order = PrintOrder {
                    order_id: Uuid::now_v7(),
                    election_id: req.election_id,
                    position_code: req.position_code.clone(),
                    constituency_code: req.constituency_code.clone(),
                    ballot_quantity: req.ballot_quantity,
                    proof_id: req.proof_id,
                    security_features: req.security_features.clone(),
                    status: "ordered".to_string(),
                    printer_reference: format!("PRN-{}", Uuid::now_v7()),
                    created_at: now,
                    updated_at: now,
                };
                sqlx::query(
                    r#"INSERT INTO printing_api_orders
                       (order_id, election_id, position_code, constituency_code, ballot_quantity, proof_id, security_features, status, printer_reference, created_at, updated_at)
                       VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$10)"#,
                )
                .bind(order.order_id)
                .bind(order.election_id)
                .bind(&order.position_code)
                .bind(&order.constituency_code)
                .bind(order.ballot_quantity)
                .bind(order.proof_id)
                .bind(serde_json::to_value(&order.security_features).unwrap_or(json!([])))
                .bind(&order.status)
                .bind(&order.printer_reference)
                .bind(now)
                .execute(pool)
                .await?;
                sqlx::query(
                    "INSERT INTO printing_api_audit (event_id, event_type, order_id, payload_summary, created_at) VALUES ($1,'order_created',$2,$3,$4)",
                )
                .bind(Uuid::now_v7())
                .bind(order.order_id)
                .bind(json!({ "order_id": order.order_id }))
                .bind(now)
                .execute(pool)
                .await?;
                Ok(order)
            }
            Self::Mem(m) => {
                let now = Utc::now();
                let order = PrintOrder {
                    order_id: Uuid::now_v7(),
                    election_id: req.election_id,
                    position_code: req.position_code,
                    constituency_code: req.constituency_code,
                    ballot_quantity: req.ballot_quantity,
                    proof_id: req.proof_id,
                    security_features: req.security_features,
                    status: "ordered".to_string(),
                    printer_reference: format!("PRN-{}", Uuid::now_v7()),
                    created_at: now,
                    updated_at: now,
                };
                m.orders.insert(order.order_id, order.clone());
                m.audit.lock().expect("lock poisoned").push(AuditEvent {
                    event_id: Uuid::now_v7(),
                    event_type: "order_created".to_string(),
                    order_id: Some(order.order_id),
                    actor_role: None,
                    payload_summary: json!({ "order_id": order.order_id }),
                    created_at: now,
                });
                Ok(order)
            }
        }
    }

    pub async fn list_orders(&self) -> Result<Vec<PrintOrder>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, i32, Uuid, serde_json::Value, String, String, DateTime<Utc>, DateTime<Utc>)>(
                    "SELECT order_id, election_id, position_code, constituency_code, ballot_quantity, proof_id, security_features, status, printer_reference, created_at, updated_at FROM printing_api_orders ORDER BY created_at",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(|(oid, eid, pos, cc, qty, proof, sec, status, pref, created, updated)| PrintOrder {
                        order_id: oid,
                        election_id: eid,
                        position_code: pos,
                        constituency_code: cc,
                        ballot_quantity: qty,
                        proof_id: proof,
                        security_features: serde_json::from_value(sec).unwrap_or_default(),
                        status,
                        printer_reference: pref,
                        created_at: created,
                        updated_at: updated,
                    })
                    .collect())
            }
            Self::Mem(m) => {
                let mut v = m.orders.values();
                v.sort_by(|a, b| a.order_id.cmp(&b.order_id));
                Ok(v)
            }
        }
    }

    pub async fn get_order(&self, id: Uuid) -> Result<Option<PrintOrder>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let row = sqlx::query_as::<_, (Uuid, Uuid, String, String, i32, Uuid, serde_json::Value, String, String, DateTime<Utc>, DateTime<Utc>)>(
                    "SELECT order_id, election_id, position_code, constituency_code, ballot_quantity, proof_id, security_features, status, printer_reference, created_at, updated_at FROM printing_api_orders WHERE order_id=$1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(|(oid, eid, pos, cc, qty, proof, sec, status, pref, created, updated)| PrintOrder {
                    order_id: oid,
                    election_id: eid,
                    position_code: pos,
                    constituency_code: cc,
                    ballot_quantity: qty,
                    proof_id: proof,
                    security_features: serde_json::from_value(sec).unwrap_or_default(),
                    status,
                    printer_reference: pref,
                    created_at: created,
                    updated_at: updated,
                }))
            }
            Self::Mem(m) => Ok(m.orders.get(&id)),
        }
    }

    pub async fn submit_qc(&self, id: Uuid, decision: QCDecision) -> Result<Option<PrintOrder>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let status = if decision.passed { "qc" } else { "ordered" };
                let now = Utc::now();
                let res = sqlx::query(
                    "UPDATE printing_api_orders SET status=$1, updated_at=$2 WHERE order_id=$3",
                )
                .bind(status)
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?;
                if res.rows_affected() == 0 {
                    return Ok(None);
                }
                self.get_order(id).await
            }
            Self::Mem(m) => {
                let res = m.orders.with_lock(|map| {
                    if let Some(o) = map.get_mut(&id) {
                        o.status = if decision.passed { "qc".to_string() } else { "ordered".to_string() };
                        o.updated_at = Utc::now();
                        Some(o.clone())
                    } else {
                        None
                    }
                });
                Ok(res)
            }
        }
    }

    pub async fn record_delivery(&self, req: DeliveryRecordRequest) -> Result<DeliveryRecord, StoreError> {
        match self {
            Self::Pg(pool) => {
                // ensure order exists
                if self.get_order(req.order_id).await?.is_none() {
                    return Err(StoreError::NotFound);
                }
                let now = Utc::now();
                let delivery = DeliveryRecord {
                    delivery_id: Uuid::now_v7(),
                    order_id: req.order_id,
                    delivered_quantity: req.delivered_quantity,
                    serial_range_start: req.serial_range_start.clone(),
                    serial_range_end: req.serial_range_end.clone(),
                    received_by_hash: req.received_by_hash.clone(),
                    received_at: req.received_at,
                    status: "received".to_string(),
                    created_at: now,
                };
                sqlx::query(
                    "INSERT INTO printing_api_deliveries (delivery_id, order_id, delivered_quantity, serial_range_start, serial_range_end, received_by_hash, received_at, status, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,'received',$8)",
                )
                .bind(delivery.delivery_id)
                .bind(delivery.order_id)
                .bind(delivery.delivered_quantity)
                .bind(&delivery.serial_range_start)
                .bind(&delivery.serial_range_end)
                .bind(&delivery.received_by_hash)
                .bind(delivery.received_at)
                .bind(now)
                .execute(pool)
                .await?;
                sqlx::query(
                    "INSERT INTO printing_api_audit (event_id, event_type, order_id, payload_summary, created_at) VALUES ($1,'delivery_recorded',$2,$3,$4)",
                )
                .bind(Uuid::now_v7())
                .bind(delivery.order_id)
                .bind(serde_json::to_value(&delivery).unwrap_or(json!({})))
                .bind(now)
                .execute(pool)
                .await?;
                Ok(delivery)
            }
            Self::Mem(m) => {
                let now = Utc::now();
                let delivery = DeliveryRecord {
                    delivery_id: Uuid::now_v7(),
                    order_id: req.order_id,
                    delivered_quantity: req.delivered_quantity,
                    serial_range_start: req.serial_range_start,
                    serial_range_end: req.serial_range_end,
                    received_by_hash: req.received_by_hash,
                    received_at: req.received_at,
                    status: "received".to_string(),
                    created_at: now,
                };
                m.deliveries.insert(delivery.delivery_id, delivery.clone());
                m.audit.lock().expect("lock poisoned").push(AuditEvent {
                    event_id: Uuid::now_v7(),
                    event_type: "delivery_recorded".to_string(),
                    order_id: Some(delivery.order_id),
                    actor_role: None,
                    payload_summary: serde_json::to_value(&delivery).unwrap_or(json!({})),
                    created_at: now,
                });
                Ok(delivery)
            }
        }
    }

    pub async fn list_audit(&self) -> Result<Vec<AuditEvent>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rows = sqlx::query_as::<_, (Uuid, String, Option<Uuid>, Option<String>, serde_json::Value, DateTime<Utc>)>(
                    "SELECT event_id, event_type, order_id, actor_role, payload_summary, created_at FROM printing_api_audit ORDER BY created_at",
                )
                .fetch_all(pool)
                .await?;
                Ok(rows
                    .into_iter()
                    .map(|(eid, et, oid, role, payload, created)| AuditEvent {
                        event_id: eid,
                        event_type: et,
                        order_id: oid,
                        actor_role: role,
                        payload_summary: payload,
                        created_at: created,
                    })
                    .collect())
            }
            Self::Mem(m) => Ok(m.audit.lock().expect("lock poisoned").clone()),
        }
    }
}
