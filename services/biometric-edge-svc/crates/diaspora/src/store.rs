#![allow(missing_docs)]
//! Postgres persistence for diaspora API surface (TD-03-P0-01 — no in-memory).
//!
//! `Store::Pg(PgPool)` is production (sqlx against `diaspora_api_*` tables,
//! `migrations/V5__diaspora_api.sql`). `Store::Mem` is hermetic tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Clone, Default)]
pub struct MemStore {
    pub embassies: Arc<Mutex<HashMap<String, Value>>>,
    pub ballot_dispatches: Arc<Mutex<HashMap<String, Value>>>,
    pub return_dispatches: Arc<Mutex<HashMap<String, Value>>>,
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

    pub async fn list_embassies(&self) -> Result<Vec<Value>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rows = sqlx::query_as::<_, (Value,)>("SELECT data FROM diaspora_api_embassies ORDER BY embassy_code")
                    .fetch_all(pool)
                    .await?;
                Ok(rows.into_iter().map(|(v,)| v).collect())
            }
            Self::Mem(m) => {
                let guard = m.embassies.lock().expect("lock poisoned");
                let mut v: Vec<Value> = guard.values().cloned().collect();
                v.sort_by(|a, b| a["embassy_code"].as_str().cmp(&b["embassy_code"].as_str()));
                Ok(v)
            }
        }
    }

    pub async fn get_embassy(&self, code: &str) -> Result<Option<Value>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let row = sqlx::query_as::<_, (Value,)>("SELECT data FROM diaspora_api_embassies WHERE embassy_code=$1")
                    .bind(code)
                    .fetch_optional(pool)
                    .await?;
                Ok(row.map(|(v,)| v))
            }
            Self::Mem(m) => Ok(m.embassies.lock().expect("lock poisoned").get(code).cloned()),
        }
    }

    pub async fn create_ballot_dispatch(&self, id: &str, data: Value) -> Result<Value, StoreError> {
        match self {
            Self::Pg(pool) => {
                sqlx::query("INSERT INTO diaspora_api_ballot_dispatches (dispatch_id, data) VALUES ($1,$2) ON CONFLICT (dispatch_id) DO UPDATE SET data=$2")
                    .bind(id)
                    .bind(&data)
                    .execute(pool)
                    .await?;
                Ok(data)
            }
            Self::Mem(m) => {
                m.ballot_dispatches.lock().expect("lock poisoned").insert(id.to_string(), data.clone());
                Ok(data)
            }
        }
    }

    pub async fn list_ballot_dispatches(&self) -> Result<Vec<Value>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rows = sqlx::query_as::<_, (Value,)>("SELECT data FROM diaspora_api_ballot_dispatches ORDER BY dispatch_id")
                    .fetch_all(pool)
                    .await?;
                Ok(rows.into_iter().map(|(v,)| v).collect())
            }
            Self::Mem(m) => {
                let guard = m.ballot_dispatches.lock().expect("lock poisoned");
                let mut v: Vec<Value> = guard.values().cloned().collect();
                v.sort_by(|a, b| a["dispatch_id"].as_str().cmp(&b["dispatch_id"].as_str()));
                Ok(v)
            }
        }
    }

    pub async fn get_ballot_dispatch(&self, id: &str) -> Result<Option<Value>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let row = sqlx::query_as::<_, (Value,)>("SELECT data FROM diaspora_api_ballot_dispatches WHERE dispatch_id=$1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?;
                Ok(row.map(|(v,)| v))
            }
            Self::Mem(m) => Ok(m.ballot_dispatches.lock().expect("lock poisoned").get(id).cloned()),
        }
    }

    pub async fn confirm_ballot_delivery(&self, id: &str, confirmation: Value) -> Result<Option<Value>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let existing = self.get_ballot_dispatch(id).await?;
                if let Some(mut v) = existing {
                    v["status"] = Value::String("delivered".to_string());
                    v["delivery_confirmation"] = confirmation;
                    sqlx::query("UPDATE diaspora_api_ballot_dispatches SET data=$1 WHERE dispatch_id=$2")
                        .bind(&v)
                        .bind(id)
                        .execute(pool)
                        .await?;
                    Ok(Some(v))
                } else {
                    Ok(None)
                }
            }
            Self::Mem(m) => {
                let mut guard = m.ballot_dispatches.lock().expect("lock poisoned");
                if let Some(v) = guard.get_mut(id) {
                    v["status"] = Value::String("delivered".to_string());
                    v["delivery_confirmation"] = confirmation;
                    Ok(Some(v.clone()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn list_return_dispatches(&self) -> Result<Vec<Value>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let rows = sqlx::query_as::<_, (Value,)>("SELECT data FROM diaspora_api_return_dispatches ORDER BY return_id")
                    .fetch_all(pool)
                    .await?;
                Ok(rows.into_iter().map(|(v,)| v).collect())
            }
            Self::Mem(m) => {
                let guard = m.return_dispatches.lock().expect("lock poisoned");
                let mut v: Vec<Value> = guard.values().cloned().collect();
                v.sort_by(|a, b| a["return_id"].as_str().cmp(&b["return_id"].as_str()));
                Ok(v)
            }
        }
    }
}
