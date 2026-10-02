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

//! Public grievance intake.
//!
//! The previous handler minted a `grievance_id` and an `ack_ref`, returned them,
//! and stored nothing — so a grievance filed through the public form existed
//! nowhere. The `ack_ref` in particular looked like a reference into a case
//! management system that had never heard of it.
//!
//! Grievances are now filed against the store, keyed by the Idempotency-Key the
//! request carries, so a retry returns the same grievance rather than a second one.

use axum::{
	extract::State,
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use chrono::Utc;
use serde_json::json;
use tracing::info;

use crate::mobile_store::SharedMobileStore;
use crate::models::GrievanceIntakeRequest;

/// A problem+json response.
fn problem(status: StatusCode, detail: impl Into<String>) -> axum::response::Response {
	(
		status,
		Json(json!({
			"type": "about:blank",
			"title": status.canonical_reason().unwrap_or("Error"),
			"status": status.as_u16(),
			"detail": detail.into(),
		})),
	)
		.into_response()
}

pub async fn file_grievance(
	State(mobile): State<SharedMobileStore>,
	Json(req): Json<GrievanceIntakeRequest>,
) -> axum::response::Response {
	info!(channel = %req.channel, "grievance.file_grievance");

	// The Idempotency-Key travels in the body on this route, so a retry from a
	// client that lost the response is recognisable.
	let key = Some(req.idempotency_key.trim())
		.filter(|k| !k.is_empty())
		.map(str::to_string);
	if let Some(key) = key.as_deref() {
		if let Some(existing) = mobile.grievance_by_idempotency(key) {
			return (
				StatusCode::OK,
				Json(json!({
					"grievance_id": existing.grievance_id,
					"channel": existing.channel,
					"submitted_at": existing.submitted_at,
					"duplicate_of": existing.grievance_id,
				})),
			)
				.into_response();
		}
	}
	if req.channel.trim().is_empty() {
		return problem(StatusCode::UNPROCESSABLE_ENTITY, "channel is required");
	}
	let key = key.unwrap_or_else(|| format!("grv-key-{}", uuid::Uuid::now_v7()));

	// The reporter is not identified on the public form, so there is no contact to
	// hash. A follow-up channel is what lets a reporter be reached, and it is kept.
	let grievance = mobile.file_grievance(
		req.channel.trim(),
		req.station_code.as_deref().unwrap_or_default(),
		req.station_code.as_deref().unwrap_or("general"),
		serde_json::json!({
			"description": req.description,
			"photo_blob_id": req.photo_blob_id,
			"audio_blob_id": req.audio_blob_id,
		}),
		&key,
	);
	(
		StatusCode::CREATED,
		Json(json!({
			"grievance_id": grievance.grievance_id,
			"channel": grievance.channel,
			"submitted_at": grievance.submitted_at,
			"category": grievance.category,
			// The station reference is hashed rather than stored, and the
			// acknowledgement is the grievance's own id.
			"contact_hash": grievance.contact_hash,
			"ack_ref": grievance.grievance_id,
			"idempotency_key": grievance.idempotency_key,
			"blocking_reason": "routing to a case management system needs that system, \\
								 which is not in this workspace; the grievance is recorded here \\
								 and its id is the only reference that resolves",
		})),
	)
		.into_response()
}
