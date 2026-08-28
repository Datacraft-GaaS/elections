//! ADR-0011 idempotency support.
//!
//! Mutating operations deduplicate on (operation, Idempotency-Key) via the
//! `eel_idempotency_keys` table: the first accepted response is snapshotted
//! and replayed verbatim on repeat submissions with the same request
//! fingerprint. Reuse of a key with a different request body is a 409.

use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::error::ApiError;

/// Stable fingerprint for an (operation, key) pair, stored as `key_hash`
/// so raw keys never sit in the table in plaintext.
fn key_hash(operation: &str, key: &str) -> Vec<u8> {
	Sha256::digest(format!("{operation}:{key}")).to_vec()
}

pub struct Replay {
	pub status: u16,
	pub body: Vec<u8>,
}

/// Look up a prior response snapshot for (operation, key).
///
/// Returns `Ok(Some(replay))` when the key was already accepted for this
/// operation with the same request fingerprint, `Ok(None)` when unseen,
/// and a 409 conflict when the key was reused with a different request.
pub async fn lookup(
	db: &PgPool,
	operation: &str,
	key: &str,
	request_fingerprint: &[u8],
) -> Result<Option<Replay>, ApiError> {
	let hash = key_hash(operation, key);
	let row: Option<(i16, Vec<u8>, Vec<u8>)> = sqlx::query_as(
		"SELECT response_status, response_body, request_body_hash \
		 FROM eel_idempotency_keys WHERE key_hash = $1",
	)
	.bind(&hash)
	.fetch_optional(db)
	.await?;

	match row {
		None => Ok(None),
		Some((status, body, stored_fp)) => {
			if stored_fp.as_slice() != request_fingerprint {
				return Err(ApiError::Conflict(format!(
					"Idempotency-Key was already used for {operation} with a different request"
				)));
			}
			sqlx::query(
				"UPDATE eel_idempotency_keys \
				 SET last_hit_at = NOW(), hit_count = hit_count + 1 WHERE key_hash = $1",
			)
			.bind(&hash)
			.execute(db)
			.await?;
			Ok(Some(Replay {
				status: status as u16,
				body,
			}))
		}
	}
}

/// Snapshot a freshly accepted response for later replay.
pub async fn record(
	db: &PgPool,
	operation: &str,
	key: &str,
	subject: &str,
	request_fingerprint: &[u8],
	status: u16,
	body: &[u8],
) -> Result<(), ApiError> {
	let hash = key_hash(operation, key);
	sqlx::query(
		"INSERT INTO eel_idempotency_keys \
		     (key_hash, subject_id, endpoint, request_body_hash, response_status, response_body) \
		 VALUES ($1, $2, $3, $4, $5, $6) \
		 ON CONFLICT (key_hash) DO NOTHING",
	)
	.bind(&hash)
	.bind(subject)
	.bind(operation)
	.bind(request_fingerprint)
	.bind(status as i16)
	.bind(body)
	.execute(db)
	.await?;
	Ok(())
}

/// Extract the required `Idempotency-Key` header value.
pub fn require_key(headers: &axum::http::HeaderMap) -> Result<String, ApiError> {
	headers
		.get("idempotency-key")
		.and_then(|v| v.to_str().ok())
		.map(|s| s.trim().to_string())
		.filter(|s| !s.is_empty())
		.ok_or_else(|| ApiError::Validation("Idempotency-Key header is required".to_string()))
}

/// Build a replayed response from a stored snapshot.
pub fn replay_response(replay: Replay) -> axum::response::Response {
	use axum::{
		http::{header::CONTENT_TYPE, StatusCode},
		response::IntoResponse,
	};
	let status =
		StatusCode::from_u16(replay.status).unwrap_or(StatusCode::OK);
	(
		status,
		[(CONTENT_TYPE, "application/json")],
		replay.body,
	)
		.into_response()
}

/// SHA-256 fingerprint of the exact request bytes, used to detect key reuse.
pub fn fingerprint_request(bytes: &[u8]) -> Vec<u8> {
	Sha256::digest(bytes).to_vec()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn key_hash_is_deterministic_and_operation_scoped() {
		let a = key_hash("POST /x", "k1");
		let b = key_hash("POST /x", "k1");
		let c = key_hash("POST /y", "k1");
		assert_eq!(a, b);
		assert_ne!(a, c);
		assert_eq!(a.len(), 32);
	}

	#[test]
	fn request_fingerprint_differs_by_body() {
		assert_ne!(fingerprint_request(b"a"), fingerprint_request(b"b"));
	}

	#[test]
	fn require_key_rejects_missing_and_blank() {
		let mut map = axum::http::HeaderMap::new();
		assert!(require_key(&map).is_err());
		map.insert("idempotency-key", "  ".parse().unwrap());
		assert!(require_key(&map).is_err());
		map.insert("idempotency-key", "abc".parse().unwrap());
		assert_eq!(require_key(&map).unwrap(), "abc");
	}
}
