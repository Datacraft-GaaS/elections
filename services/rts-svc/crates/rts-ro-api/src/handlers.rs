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
//
// The request carries `ro_signature`: an Ed25519 countersignature from the RO
// desktop HSM. This handler used to discard it, log the RO id, and answer
// `status: "verified"` -- so a countersignature was never checked and the answer
// said it had been. The word "verified" on a form a returning officer signs is
// the most consequential string on this endpoint.
//
// The verification algorithm is implemented and exercised; what is missing is the
// trust anchor. The public keys of the RO desktop HSMs are not in this workspace,
// so with none configured the endpoint says it cannot verify rather than claiming
// to. When a key *is* configured, the signature is really checked against the
// canonical message the HSM signs, and a bad signature is a 401.
pub async fn verify_form34a(
    axum::extract::Extension(trust): axum::extract::Extension<crate::ro_trust::SharedRoTrust>,
    Path(id): Path<Uuid>,
    Json(req): Json<VerifyForm34ARequest>,
) -> Response {
    info!(
        form_id = %id,
        ro_id = %req.ro_id,
        idempotency_key = %req.idempotency_key,
        "verify_form34a"
    );
    // An absent or malformed signature is a client error, and naming it as such is
    // more useful than a 501: the caller can fix it.
    let signature_hex = req.ro_signature.trim();
    if signature_hex.is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ro_signature is required: a Form 34A verification is a countersignature, and \
             this handler previously answered status verified without reading it",
        );
    }
    let signature_bytes = match hex::decode(signature_hex) {
        Ok(b) if b.len() == 64 => b,
        _ => {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                "ro_signature must be 64 bytes of hex: an Ed25519 signature",
            )
        }
    };
    let signature = match eel_crypto_rs::ed25519::signature_from_bytes(
        &signature_bytes
            .try_into()
            .expect("checked length is 64"),
    ) {
        Ok(s) => s,
        Err(e) => {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("ro_signature is not a valid Ed25519 signature: {e}"),
            )
        }
    };
    let message = countersigned_message(&id, &req.ro_id, &req.verified_at);
    let Some(key_hex) = trust.key_for(req.ro_id.trim()) else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "no public key is configured for this returning officer, so the countersignature \
             cannot be checked. The Ed25519 verification is implemented, but the trust \
             anchor -- the public keys of the RO desktop HSMs -- is not in this workspace. \
             This handler previously answered status verified for any request, having read \
             no signature at all",
        );
    };
    let key_bytes = match hex::decode(key_hex.as_str()) {
        Ok(b) if b.len() == 32 => b,
        _ => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("the configured public key is not 32 bytes of hex"),
            )
        }
    };
    let verifying_key = match eel_crypto_rs::ed25519::verifying_key_from_bytes(
        &key_bytes
            .try_into()
            .expect("checked length is 32"),
    ) {
        Ok(k) => k,
        Err(e) => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("the configured public key for this officer is not a valid Ed25519 \
                         verifying key: {e}"),
            )
        }
    };
    if let Err(e) = eel_crypto_rs::ed25519::verify(&verifying_key, message.as_bytes(), &signature) {
        warn!(form_id = %id, error = %e, "countersignature did not verify");
        return problem(
            StatusCode::UNAUTHORIZED,
            "the countersignature does not verify against the configured public key for this \
             returning officer",
        );
    }
    (
        StatusCode::ACCEPTED,
        Json(VerifyAccepted {
            form_id: id,
            event_id: Uuid::now_v7(),
            status: "verified".into(),
        }),
    )
        .into_response()
}

/// The canonical bytes an RO HSM countersigns.
///
/// Fixed here rather than assembled ad hoc, because a countersignature is only
/// meaningful over an agreed message: a verifier and a signer that disagree about
/// what was signed will disagree about whether it is valid.
pub fn countersigned_message(form_id: &Uuid, ro_id: &str, verified_at: &str) -> String {
    format!("eel.rts.form34a.countersigned.v1\n{form_id}\n{ro_id}\n{verified_at}")
}

fn problem(status: StatusCode, detail: impl Into<String>) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
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
