#![allow(missing_docs)]
//! Postgres persistence for credential API surface (TD-03-P0-01 — no in-memory).
//!
//! `Store::Pg(PgPool)` is production (sqlx against `credential_api_*` tables,
//! `migrations/V054__credential_api.sql`). `Store::Mem` is hermetic tests.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::spec::{BatchSummary, Credential, CredentialProof, IssueRequest, RevocationRequest};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

#[derive(Clone, Default)]
pub struct MemStore {
    pub credentials: Arc<std::sync::Mutex<std::collections::HashMap<Uuid, Credential>>>,
    pub batches: Arc<std::sync::Mutex<std::collections::HashMap<Uuid, BatchSummary>>>,
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

    pub async fn issue_credential(&self, req: IssueRequest) -> Result<Credential, StoreError> {
        match self {
            Self::Pg(pool) => {
                let now = Utc::now();
                let cred = Credential {
                    credential_id: Uuid::now_v7(),
                    credential_type: req.credential_type.clone(),
                    holder_id: req.holder_id,
                    holder_name: "holder".to_string(),
                    issuer_id: "spiffe://eel.local/credential-issuer".to_string(),
                    issued_at: now,
                    expires_at: now + chrono::Duration::days(req.valid_for_days as i64),
                    status: "active".to_string(),
                    claims: req.claims.clone(),
                    hsm_sig: String::new(),
                };
                sqlx::query(
                    r#"INSERT INTO credential_api_credentials
                       (credential_id, credential_type, holder_id, holder_name, issuer_id, issued_at, expires_at, status, claims, hsm_sig, batch_id)
                       VALUES ($1,$2,$3,$4,$5,$6,$7,'active',$8,$9,$10)"#,
                )
                .bind(cred.credential_id)
                .bind(&cred.credential_type)
                .bind(cred.holder_id)
                .bind(&cred.holder_name)
                .bind(&cred.issuer_id)
                .bind(cred.issued_at)
                .bind(cred.expires_at)
                .bind(&cred.claims)
                .bind(&cred.hsm_sig)
                .bind(req.batch_id)
                .execute(pool)
                .await?;
                if let Some(bid) = req.batch_id {
                    sqlx::query(
                        r#"INSERT INTO credential_api_batches (batch_id, credential_type, total_issued, total_failed, total_revoked, status, created_at, completed_at)
                           VALUES ($1,$2,1,0,0,'completed',$3,$3)
                           ON CONFLICT (batch_id) DO UPDATE SET total_issued = credential_api_batches.total_issued + 1"#,
                    )
                    .bind(bid)
                    .bind(&cred.credential_type)
                    .bind(now)
                    .execute(pool)
                    .await?;
                }
                Ok(cred)
            }
            Self::Mem(m) => {
                let now = Utc::now();
                let cred = Credential {
                    credential_id: Uuid::now_v7(),
                    credential_type: req.credential_type.clone(),
                    holder_id: req.holder_id,
                    holder_name: "holder".to_string(),
                    issuer_id: "spiffe://eel.local/credential-issuer".to_string(),
                    issued_at: now,
                    expires_at: now + chrono::Duration::days(req.valid_for_days as i64),
                    status: "active".to_string(),
                    claims: req.claims.clone(),
                    hsm_sig: String::new(),
                };
                let id = cred.credential_id;
                m.credentials.lock().expect("lock poisoned").insert(id, cred.clone());
                if let Some(bid) = req.batch_id {
                    let mut b = m.batches.lock().expect("lock poisoned");
                    b.entry(bid).or_insert_with(|| BatchSummary {
                        batch_id: bid,
                        credential_type: cred.credential_type.clone(),
                        total_issued: 0,
                        total_failed: 0,
                        total_revoked: 0,
                        status: "completed".to_string(),
                        created_at: now,
                        completed_at: Some(now),
                    });
                    if let Some(entry) = b.get_mut(&bid) {
                        entry.total_issued += 1;
                    }
                }
                Ok(cred)
            }
        }
    }

    pub async fn get_credential(&self, id: Uuid) -> Result<Option<Credential>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let row = sqlx::query_as::<_, (Uuid, String, Uuid, String, String, DateTime<Utc>, DateTime<Utc>, String, Value, String)>(
                    "SELECT credential_id, credential_type, holder_id, holder_name, issuer_id, issued_at, expires_at, status, claims, hsm_sig FROM credential_api_credentials WHERE credential_id=$1",
                )
                .bind(id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(|(cid, ct, hid, hname, iid, issued, expires, status, claims, sig)| Credential {
                    credential_id: cid,
                    credential_type: ct,
                    holder_id: hid,
                    holder_name: hname,
                    issuer_id: iid,
                    issued_at: issued,
                    expires_at: expires,
                    status,
                    claims,
                    hsm_sig: sig,
                }))
            }
            Self::Mem(m) => Ok(m.credentials.lock().expect("lock poisoned").get(&id).cloned()),
        }
    }

    pub async fn revoke_credential(&self, id: Uuid, _req: RevocationRequest) -> Result<Option<Credential>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let res = sqlx::query("UPDATE credential_api_credentials SET status='revoked' WHERE credential_id=$1")
                    .bind(id)
                    .execute(pool)
                    .await?;
                if res.rows_affected() == 0 {
                    return Ok(None);
                }
                // bump batch revoked if needed (lookup batch_id)
                let batch: Option<(Option<Uuid>,)> = sqlx::query_as("SELECT batch_id FROM credential_api_credentials WHERE credential_id=$1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await?;
                if let Some((Some(bid),)) = batch {
                    let _ = sqlx::query("UPDATE credential_api_batches SET total_revoked = total_revoked + 1 WHERE batch_id=$1")
                        .bind(bid)
                        .execute(pool)
                        .await;
                }
                self.get_credential(id).await
            }
            Self::Mem(m) => {
                let mut guard = m.credentials.lock().expect("lock poisoned");
                if let Some(v) = guard.get_mut(&id) {
                    v.status = "revoked".to_string();
                    Ok(Some(v.clone()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn get_proof(&self, id: Uuid) -> Result<Option<CredentialProof>, StoreError> {
        if let Some(c) = self.get_credential(id).await? {
            Ok(Some(CredentialProof {
                credential_id: c.credential_id,
                proof_type: "EcdsaSecp384r1Signature2023".to_string(),
                proof_value: c.hsm_sig.clone(),
                verification_method: "https://keys.eel.local/credential-hsm#key-2026".to_string(),
                created_at: c.issued_at,
                transparency_leaf_hash: None,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_batch(&self, batch_id: Uuid) -> Result<Option<BatchSummary>, StoreError> {
        match self {
            Self::Pg(pool) => {
                let row = sqlx::query_as::<_, (Uuid, String, i32, i32, i32, String, DateTime<Utc>, Option<DateTime<Utc>>)>(
                    "SELECT batch_id, credential_type, total_issued, total_failed, total_revoked, status, created_at, completed_at FROM credential_api_batches WHERE batch_id=$1",
                )
                .bind(batch_id)
                .fetch_optional(pool)
                .await?;
                Ok(row.map(|(bid, ct, issued, failed, revoked, status, created, completed)| BatchSummary {
                    batch_id: bid,
                    credential_type: ct,
                    total_issued: issued,
                    total_failed: failed,
                    total_revoked: revoked,
                    status,
                    created_at: created,
                    completed_at: completed,
                }))
            }
            Self::Mem(m) => Ok(m.batches.lock().expect("lock poisoned").get(&batch_id).cloned()),
        }
    }
}
