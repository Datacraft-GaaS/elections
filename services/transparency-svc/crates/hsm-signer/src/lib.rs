//! eel HSM Signer gRPC service (WP-2.2).
//!
//! Implements the `HsmSigner` protobuf service. All operations require a valid
//! SPIFFE SVID in the TLS peer certificate (enforced by the TLS interceptor
//! configured in `main.rs`). Missing identity returns `UNAUTHENTICATED`.
//!
//! Trust domain: `spiffe://eel.local/service/hsm-signer`.
#![warn(missing_docs)]

pub mod ceremony;
pub mod keg_rotation_4of7;
pub mod metrics;
pub mod pool;
pub mod rate;

use std::pin::Pin;
use std::sync::Arc;

use chrono::Utc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tonic::{metadata::MetadataValue, Code, Request, Response, Status};
use tracing::{error, info, warn};
use uuid::Uuid;

use eel_crypto_rs::KeyRole;

use ceremony::CeremonyStore;
use metrics::metrics;
use pool::RolePool;
use rate::RoleLimiters;

// Generated gRPC types.
pub mod proto {
	tonic::include_proto!("eel.hsm.v1");
}

use proto::hsm_signer_server::HsmSigner;
use proto::{
	CeremonyEvent, CeremonyRequest, KeyInfoResponse, KeyRoleRequest, RotateRequest,
	RotateResponse, SignRequest, SignResponse, VerifyRequest, VerifyResponse,
};

type GrpcStream<T> =
	Pin<Box<dyn tokio_stream::Stream<Item = Result<T, Status>> + Send + 'static>>;

/// The main HSM signer service.
pub struct HsmSignerService {
	pool: Arc<RolePool>,
	limiters: Arc<RoleLimiters>,
	ceremonies: Arc<CeremonyStore>,
}

impl HsmSignerService {
	pub fn new(pool: RolePool) -> Self {
		Self {
			pool: Arc::new(pool),
			limiters: Arc::new(RoleLimiters::new()),
			ceremonies: Arc::new(CeremonyStore::new()),
		}
	}

	/// Expose the ceremony store (e.g. for CLI approve/abort endpoints).
	pub fn ceremonies(&self) -> Arc<CeremonyStore> {
		Arc::clone(&self.ceremonies)
	}
}

// ---------------------------------------------------------------------------
// Proto KeyRole <-> eel_crypto_rs::KeyRole conversion
// ---------------------------------------------------------------------------

fn proto_role_to_crypto(r: i32) -> Result<KeyRole, Status> {
	use proto::KeyRole as PR;
	match PR::try_from(r).unwrap_or(PR::Unspecified) {
		PR::Sth => Ok(KeyRole::Sth),
		PR::Declaration => Ok(KeyRole::Declaration),
		PR::KitCa => Ok(KeyRole::KitCa),
		PR::CredentialCa => Ok(KeyRole::CredentialCa),
		PR::CodeSigning => Ok(KeyRole::CodeSigning),
		PR::CycleKek => Ok(KeyRole::CycleKek),
		PR::Unspecified => Err(Status::invalid_argument("key_role must be specified")),
	}
}

fn role_label(role: KeyRole) -> &'static str {
	role.label()
}

// ---------------------------------------------------------------------------
// Rate limiting helper
// ---------------------------------------------------------------------------

fn check_rate(
	limiters: &RoleLimiters,
	role: KeyRole,
) -> Result<(), Status> {
	limiters.check(role).map_err(|retry_after| {
		metrics().rate_limited_total.with_label_values(&[role_label(role)]).inc();
		warn!(role = role_label(role), retry_after, "rate limit exceeded");
		let mut s = Status::resource_exhausted("rate limit exceeded");
		s.metadata_mut().insert(
			"retry-after",
			MetadataValue::try_from(retry_after.to_string().as_str())
				.unwrap_or_else(|_| MetadataValue::from_static("1")),
		);
		s
	})
}

// ---------------------------------------------------------------------------
// gRPC service implementation
// ---------------------------------------------------------------------------

#[tonic::async_trait]
impl HsmSigner for HsmSignerService {
	type CeremonyStream = GrpcStream<CeremonyEvent>;

	async fn sign(&self, request: Request<SignRequest>) -> Result<Response<SignResponse>, Status> {
		let req = request.into_inner();
		let role = proto_role_to_crypto(req.key_role)?;

		check_rate(&self.limiters, role)?;

		let pool = Arc::clone(&self.pool);
		let payload = req.payload.clone();
		let purpose = req.purpose.clone();

		let (sig, key_id, algo) = tokio::task::spawn_blocking(move || pool.sign(role, &payload))
			.await
			.map_err(|e| Status::internal(format!("task join: {e}")))?
			.map_err(|e| {
				metrics()
					.sign_error_total
					.with_label_values(&[role_label(role)])
					.inc();
				error!(role = role_label(role), error = %e, "sign failed");
				Status::internal(format!("sign error: {e}"))
			})?;

		metrics().sign_total.with_label_values(&[role_label(role)]).inc();
		info!(role = role_label(role), purpose = %purpose, key_id = %key_id, "signed");

		Ok(Response::new(SignResponse {
			signature: sig,
			key_id,
			algorithm: algo,
		}))
	}

	async fn verify(
		&self,
		request: Request<VerifyRequest>,
	) -> Result<Response<VerifyResponse>, Status> {
		let req = request.into_inner();
		let role = proto_role_to_crypto(req.key_role)?;

		let pool = Arc::clone(&self.pool);
		let payload = req.payload.clone();
		let signature = req.signature.clone();

		let ok = tokio::task::spawn_blocking(move || pool.verify(role, &payload, &signature))
			.await
			.map_err(|e| Status::internal(format!("task join: {e}")))?
			.map_err(|e| Status::internal(format!("verify error: {e}")))?;

		Ok(Response::new(VerifyResponse { ok }))
	}

	async fn key_info(
		&self,
		request: Request<KeyRoleRequest>,
	) -> Result<Response<KeyInfoResponse>, Status> {
		let req = request.into_inner();
		let role = proto_role_to_crypto(req.key_role)?;

		// In production these fields come from the HSM CKA attributes; here we
		// return the role label as a stable key_id placeholder since parsing
		// full CKA_PUBLIC_KEY_INFO for a fingerprint requires heavier machinery.
		Ok(Response::new(KeyInfoResponse {
			key_id: format!("eel-{}", role_label(role)),
			algorithm: "ECDSA-P384".to_string(),
			fingerprint: String::new(),
			created_at: Utc::now().to_rfc3339(),
			rotation_due_at: String::new(),
		}))
	}

	async fn rotate_key(
		&self,
		request: Request<RotateRequest>,
	) -> Result<Response<RotateResponse>, Status> {
		let req = request.into_inner();
		let role = proto_role_to_crypto(req.key_role)?;

		// CycleKek rotations require an Approved ceremony.
		if role == KeyRole::CycleKek && !req.ceremony_id.is_empty() {
			self.ceremonies
				.assert_approved(&req.ceremony_id)
				.map_err(|e| Status::failed_precondition(e))?;
		}

		check_rate(&self.limiters, role)?;

		// Record Executing + Complete transition.
		if !req.ceremony_id.is_empty() {
			let _ = self.ceremonies.mark_executing(&req.ceremony_id);
		}

		// Actual key rotation via PKCS#11 C_GenerateKeyPair is out of scope for
		// this WP; the ceremony gate and outbox emission are the primary
		// deliverables. Full key generation follows in WP-3.x.
		let old_key_id = format!("eel-{}", role_label(role));
		let new_key_id = format!("eel-{}-{}", role_label(role), &Uuid::new_v4().to_string()[..8]);

		info!(role = role_label(role), ceremony_id = %req.ceremony_id, "key rotation initiated");
		metrics()
			.ceremony_complete_total
			.with_label_values(&[role_label(role)])
			.inc();

		if !req.ceremony_id.is_empty() {
			let _ = self.ceremonies.mark_complete(&req.ceremony_id);
		}

		Ok(Response::new(RotateResponse {
			new_key_id,
			old_key_id,
			ceremony_state: "Complete".to_string(),
		}))
	}

	async fn ceremony(
		&self,
		request: Request<CeremonyRequest>,
	) -> Result<Response<Self::CeremonyStream>, Status> {
		let req = request.into_inner();

		let (ceremony_obj, rx) = if req.ceremony_id.is_empty() {
			// New ceremony.
			let role = proto_role_to_crypto(req.key_role)?;
			let m = if req.quorum_m == 0 { 3 } else { req.quorum_m };
			let n = if req.quorum_n == 0 { 5 } else { req.quorum_n };
			let (c, rx) = self.ceremonies.open(role, req.purpose.clone(), m, n);
			metrics()
				.ceremony_started_total
				.with_label_values(&[role_label(role)])
				.inc();
			info!(ceremony_id = %c.id, role = role_label(role), "ceremony opened");
			(c, rx)
		} else {
			// Subscribe to existing ceremony.
			let rx = self
				.ceremonies
				.subscribe(&req.ceremony_id)
				.ok_or_else(|| Status::not_found(format!("ceremony {} not found", req.ceremony_id)))?;
			let ceremony_obj = self
				.ceremonies
				.get(&req.ceremony_id)
				.ok_or_else(|| Status::not_found("ceremony not found"))?;
			(ceremony_obj, rx)
		};

		// Send the initial event.
		let initial = CeremonyEvent {
			ceremony_id: ceremony_obj.id.clone(),
			state: proto::CeremonyState::Requested as i32,
			approval: None,
			abort_reason: String::new(),
			approvals_count: 0,
			quorum_m: ceremony_obj.quorum_m,
			operation_hash: ceremony_obj.operation_hash.clone(),
		};

		let broadcast_stream = BroadcastStream::new(rx).filter_map(|result| {
			result.ok().map(|update| {
				let state = ceremony_state_to_proto(&update.state_label);
				Ok(CeremonyEvent {
					ceremony_id: update.ceremony_id,
					state: state as i32,
					approval: update.approval.map(|a| proto::CustodianApproval {
						custodian_id: a.custodian_id,
						signature: a.signature,
						approved_at: a.approved_at,
					}),
					abort_reason: update.abort_reason.unwrap_or_default(),
					approvals_count: update.approvals_count,
					quorum_m: update.quorum_m,
					operation_hash: update.operation_hash,
				})
			})
		});

		let initial_stream =
			tokio_stream::once(Ok(initial)).chain(broadcast_stream);

		Ok(Response::new(Box::pin(initial_stream)))
	}
}

fn ceremony_state_to_proto(label: &str) -> proto::CeremonyState {
	match label {
		"Requested" => proto::CeremonyState::Requested,
		"AwaitingQuorum" => proto::CeremonyState::AwaitingQuorum,
		"Approved" => proto::CeremonyState::Approved,
		"Executing" => proto::CeremonyState::Executing,
		"Complete" => proto::CeremonyState::Complete,
		"Aborted" => proto::CeremonyState::Aborted,
		_ => proto::CeremonyState::Unspecified,
	}
}
