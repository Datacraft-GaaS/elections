use axum::{
	Json,
	extract::Path,
};
use chrono::Utc;
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

use crate::models::{TokenVerifyRequest, TokenVerifyResponse};

pub async fn get_results() -> Json<Value> {
	Json(serde_json::json!({
		"status": "ok",
		"data": [],
		"note": "relay from M11; CDN-cached"
	}))
}

pub async fn get_results_by_scope(
	Path((scope, code)): Path<(String, String)>,
) -> Json<Value> {
	info!(scope, code, "verify.get_results_by_scope");
	Json(serde_json::json!({
		"scope": scope,
		"code": code,
		"data": null,
		"note": "constituency/regional results relay"
	}))
}

pub async fn verify_token(
	Json(req): Json<TokenVerifyRequest>,
) -> Json<TokenVerifyResponse> {
	let _log_query_id = Uuid::now_v7().to_string();
	info!(token_sha256 = %req.token_sha256, "verify.verify_token");
	Json(TokenVerifyResponse {
		query_id: _log_query_id,
		token_sha256: req.token_sha256,
		included: true,
		inclusion_proof_ref: Some("stub-proof-ref".to_string()),
		verified_at: Utc::now(),
	})
}

pub async fn get_token_proof(
	Path(sha): Path<String>,
) -> Json<Value> {
	info!(sha, "verify.get_token_proof");
	Json(serde_json::json!({
		"sha": sha,
		"proof": null,
		"note": "inclusion proof JSON stub"
	}))
}

pub async fn get_anomalies() -> Json<Value> {
	Json(serde_json::json!({
		"anomalies": [],
		"note": "public anomaly/fraud flags relay from M26"
	}))
}
