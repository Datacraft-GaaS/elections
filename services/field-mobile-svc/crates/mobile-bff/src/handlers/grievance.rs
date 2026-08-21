use axum::Json;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::models::{GrievanceIntakeRequest, GrievanceRef};

pub async fn file_grievance(
	Json(req): Json<GrievanceIntakeRequest>,
) -> Json<GrievanceRef> {
	let grievance_id = Uuid::now_v7().to_string();
	info!(
		grievance_id,
		channel = %req.channel,
		idempotency_key = %req.idempotency_key,
		"grievance.file_grievance"
	);
	Json(GrievanceRef {
		grievance_id,
		channel: req.channel,
		submitted_at: Utc::now(),
		ack_ref: format!("ack-{}", Uuid::now_v7()),
	})
}
