use axum::Json;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::models::{
	BinaryIntegrityEvent, BinaryIntegrityRequest, CloneReport, CloneReportRequest,
};

pub async fn report_clone(
	Json(req): Json<CloneReportRequest>,
) -> Json<CloneReport> {
	let report_id = Uuid::now_v7().to_string();
	info!(
		report_id,
		package_name = %req.package_name,
		platform = %req.platform,
		"clone_detect.report_clone"
	);
	Json(CloneReport {
		report_id,
		app_id: req.app_id,
		package_name: req.package_name,
		reported_signature: req.reported_signature,
		platform: req.platform,
		reported_at: Utc::now(),
	})
}

pub async fn binary_integrity_check(
	Json(req): Json<BinaryIntegrityRequest>,
) -> Json<BinaryIntegrityEvent> {
	let event_id = Uuid::now_v7().to_string();
	info!(
		event_id,
		app_id = %req.app_id,
		"clone_detect.binary_integrity_check"
	);
	Json(BinaryIntegrityEvent {
		event_id,
		app_id: req.app_id,
		build_fingerprint: req.build_fingerprint,
		attestation_token: req.attestation_token,
		passed: true,
		failure_reason: None,
		emitted_at: Utc::now(),
	})
}
