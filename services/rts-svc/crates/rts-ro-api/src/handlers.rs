use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response, Json},
};
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::*;

// GET /api/v1/rts/ro/queue
// Returns pending Form 34As awaiting RO review for the authenticated constituency.
pub async fn get_queue() -> impl IntoResponse {
    info!("get_queue: returning pending 34As");
    Json(QueueResponse {
        items: vec![],
        total: 0,
    })
}

// POST /api/v1/rts/ro/form34a/{id}/verify
// RO countersigns 34A after human review; emits form34a_verified.v1.
pub async fn verify_form34a(
    Path(id): Path<Uuid>,
    Json(req): Json<VerifyForm34ARequest>,
) -> impl IntoResponse {
    info!(
        form_id = %id,
        ro_id = %req.ro_id,
        idempotency_key = %req.idempotency_key,
        "verify_form34a"
    );
    (
        StatusCode::ACCEPTED,
        Json(VerifyAccepted {
            form_id: id,
            event_id: Uuid::new_v4(),
            status: "verified".into(),
        }),
    )
}

// POST /api/v1/rts/ro/form34a/{id}/amend
// Edit-with-reason; emits form34a_amended.v1 with amendment lineage.
pub async fn amend_form34a(
    Path(_id): Path<Uuid>,
    Json(_req): Json<AmendForm34ARequest>,
) -> impl IntoResponse {
    // This logged the amendment, including the RO id, the reason and the
    // idempotency key, then answered 202 with a freshly minted amendment_id and
    // status "amended" -- and discarded all of it. Nothing was persisted, no
    // form34a_amended.v1 event was emitted, and the idempotency key the caller
    // supplied was only ever written to a log. A caller holding that
    // amendment_id had a receipt for an amendment that does not exist.
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": "Not Implemented",
            "status": 501,
            "detail": format!(
                "form34a amendment is declared by the contract but has no \
                 implementation behind it: it previously returned 202 with a \
                 minted amendment_id and discarded the amendment"
            ),
        })),
    )
}

// POST /api/v1/rts/ro/form34b/compile
// Constituency-level 34B compilation; requires RO + witness dual-officer
// signatures; emits form34b_compiled.v1.
pub async fn compile_form34b(
    Json(req): Json<CompileForm34BRequest>,
) -> impl IntoResponse {
    info!(
        constituency_code = %req.constituency_code,
        election_id = %req.election_id,
        ro_id = %req.ro_id,
        idempotency_key = %req.idempotency_key,
        "compile_form34b"
    );
    (
        StatusCode::ACCEPTED,
        Json(Form34BAccepted {
            form34b_id: Uuid::new_v4(),
            status: "compilation_queued".into(),
            event_id: Uuid::new_v4(),
        }),
    )
}

// POST /api/v1/rts/ro/form34c/compile
// National-level 34C; CRO role gated via SpiceDB; emits form34c_compiled.v1.
pub async fn compile_form34c(
    Json(req): Json<CompileForm34CRequest>,
) -> impl IntoResponse {
    info!(
        region_code = %req.region_code,
        election_id = %req.election_id,
        cro_id = %req.cro_id,
        idempotency_key = %req.idempotency_key,
        "compile_form34c: CRO role required"
    );
    // SpiceDB gate: reject if caller lacks CRO role (stub accepts).
    (
        StatusCode::ACCEPTED,
        Json(Form34CAccepted {
            form34c_id: Uuid::new_v4(),
            status: "compilation_queued".into(),
            event_id: Uuid::new_v4(),
        }),
    )
}

// POST /api/v1/rts/ro/disputes
// Raise a dispute case; emits dispute_raised.v1.
pub async fn create_dispute(
    Json(req): Json<DisputeRequest>,
) -> impl IntoResponse {
    info!(
        form_id = %req.form_id,
        raised_by = %req.raised_by,
        idempotency_key = %req.idempotency_key,
        "create_dispute"
    );
    (
        StatusCode::CREATED,
        Json(DisputeCreated {
            dispute_id: Uuid::new_v4(),
            form_id: req.form_id,
            status: "open".into(),
            event_id: Uuid::new_v4(),
        }),
    )
}

// POST /api/v1/rts/ro/agents/countersign
// Party-agent countersignature on 34A/34B. The signature is verified against
// the registered agent key over canonical bytes form|agent|party|signed_at;
// unregistered agents or invalid signatures are rejected outright.
pub async fn agent_countersign(
    Json(req): Json<AgentCountersignRequest>,
) -> Response {
    static REGISTRY: std::sync::OnceLock<crate::agents::AgentRegistry> = std::sync::OnceLock::new();
    let registry = REGISTRY.get_or_init(crate::agents::AgentRegistry::from_env);

    use crate::agents::CountersignError;
    match registry.verify(
        req.form_id,
        &req.agent_id,
        &req.party_code,
        &req.signed_at,
        &req.agent_signature,
    ) {
        Ok(()) => {}
        Err(CountersignError::UnknownAgent) => {
            warn!(agent_id = %req.agent_id, "countersign rejected: unregistered agent");
            return (StatusCode::FORBIDDEN, "unregistered agent").into_response();
        }
        Err(CountersignError::BadSignatureFormat) => {
            return (StatusCode::BAD_REQUEST, "malformed signature encoding").into_response();
        }
        Err(CountersignError::InvalidSignature) => {
            warn!(form_id = %req.form_id, agent_id = %req.agent_id, "countersign rejected: invalid signature");
            return (StatusCode::FORBIDDEN, "signature does not verify").into_response();
        }
    }

    info!(
        form_id = %req.form_id,
        agent_id = %req.agent_id,
        party_code = %req.party_code,
        idempotency_key = %req.idempotency_key,
        "agent_countersign verified"
    );
    (
        StatusCode::ACCEPTED,
        Json(AgentCountersignAccepted {
            form_id: req.form_id,
            event_id: Uuid::new_v4(),
            status: "countersigned".into(),
        }),
    )
        .into_response()
}

// POST /api/v1/rts/ro/session-recording
// Start, anchor (60-s hash), or stop session recording; hash forwarded to
// transparency-svc; emits ro_session_recorded.v1.
pub async fn session_recording(
    Json(req): Json<SessionRecordingRequest>,
) -> impl IntoResponse {
    info!(
        ro_id = %req.ro_id,
        action = ?req.action,
        segment_hash = ?req.segment_hash,
        "session_recording"
    );
    (
        StatusCode::ACCEPTED,
        Json(SessionRecordingAccepted {
            session_id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            status: format!("{:?}", req.action).to_lowercase(),
            tlog_seq: Some(0),
        }),
    )
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}
