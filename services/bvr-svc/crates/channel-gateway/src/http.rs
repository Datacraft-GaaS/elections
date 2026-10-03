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

//! USSD/IVR/WhatsApp channel gateway webhooks.
//!
//! Every handler reads or advances [`crate::store::AppState`]. The one that
//! mattered most was verification: `verify_webhook` and `verify_whatsapp_webhook`
//! both returned `{"challenge": "verified"}` for any request at all, without
//! reading a signature. The WhatsApp Cloud API signs each webhook with an
//! HMAC-SHA256 over the **raw** body keyed by the app secret, and this handler now
//! performs that check.
//!
//! The algorithm is fully implemented. What is absent from this workspace is the
//! app secret itself, which is issued by the provider and must never live in a
//! repository. So with no secret configured the verification endpoints report
//! `unverifiable` and the message endpoints refuse — they do not report "verified"
//! for a check that did not happen.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tracing::info;

use crate::spec::{IvrVoiceRequest, UssdSessionRequest, UssdSessionResponse};
use crate::store::{
    hash_msisdn, parse_whatsapp_message, sign_whatsapp_body, verify_whatsapp_signature,
    ReceivedMessage, SharedState, SignatureVerdict,
};

/// The header the WhatsApp Cloud API signs with.
const SIGNATURE_HEADER: &str = "x-hub-signature-256";

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/ussd/sessions", post(handle_ussd_session))
        .route("/ussd/sessions/{session_id}", get(get_ussd_session_log))
        .route("/ivr/voice", post(handle_voice_call))
        .route("/ivr/sessions", post(handle_ivr_session))
        .route(
            "/webhook/whatsapp",
            get(verify_webhook).post(receive_message),
        )
        .route(
            "/whatsapp/webhook",
            get(verify_whatsapp_webhook).post(receive_whatsapp_webhook),
        )
        .with_state(state)
}

async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

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

/// The signature header, if one was sent.
fn signature_of(headers: &HeaderMap) -> Option<String> {
    headers
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

async fn handle_ussd_session(
    State(state): State<SharedState>,
    Json(req): Json<UssdSessionRequest>,
) -> axum::response::Response {
    if req.session_id.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "session_id is required");
    }
    // The gateway holds a hashed MSISDN rather than the number: a USSD gateway has
    // no need to retain a phone number.
    let msisdn_hash = if req.msisdn_hash.starts_with("sha256:") {
        req.msisdn_hash.clone()
    } else {
        hash_msisdn(&req.msisdn_hash)
    };
    match state.ussd_step(
        &req.session_id,
        &req.service_code,
        &req.network_code,
        &msisdn_hash,
        &req.text,
    ) {
        Ok((response_text, action)) => {
            let body = UssdSessionResponse {
                response_text,
                action: action.clone(),
            };
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(detail) => problem(StatusCode::UNPROCESSABLE_ENTITY, detail),
    }
}

async fn get_ussd_session_log(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
) -> axum::response::Response {
    // Previously any session id returned `log: []`, so a caller's whole exchange
    // was unreadable.
    let Some(session) = state.ussd_session(&session_id) else {
        return problem(
            StatusCode::NOT_FOUND,
            format!("no USSD session with id {session_id}"),
        );
    };
    Json(json!({
        "session_id": session.session_id,
        "service_code": session.service_code,
        "network_code": session.network_code,
        "msisdn_hash": session.msisdn_hash,
        "state": session.state,
        "started_at": session.started_at,
        "updated_at": session.updated_at,
        "log": session.log,
    }))
    .into_response()
}

async fn handle_voice_call(
    State(state): State<SharedState>,
    Json(req): Json<IvrVoiceRequest>,
) -> axum::response::Response {
    if req.call_id.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "call_id is required");
    }
    let (prompt, stage) = state.ivr_step(&req.call_id, &req.caller_id_hash, req.dtmf.as_deref());
    Json(json!({
        "response_text": prompt,
        "call_id": req.call_id,
        "state": stage,
    }))
    .into_response()
}

async fn handle_ivr_session(
    State(state): State<SharedState>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    // A named session form: { call_id, caller_id_hash, dtmf }. Previously this
    // echoed the request back as `END` and recorded nothing.
    let call_id = req
        .get("call_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if call_id.trim().is_empty() {
        return problem(StatusCode::UNPROCESSABLE_ENTITY, "call_id is required");
    }
    let caller = req
        .get("caller_id_hash")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let dtmf = req.get("dtmf").and_then(|v| v.as_str());
    let (prompt, stage) = state.ivr_step(&call_id, caller, dtmf);
    Json(json!({
        "response_text": prompt,
        "call_id": call_id,
        "state": stage,
    }))
    .into_response()
}

/// Shared verification handler for both webhook paths.
async fn verify(headers: &HeaderMap, secret: Option<String>) -> axum::response::Response {
    let signature = signature_of(headers);
    // A GET verification carries no body, so there is nothing to sign over. The
    // provider's webhook verification is a POST with the signed body; a GET here
    // can only report that the endpoint exists, not that a signature holds.
    let verdict = verify_whatsapp_signature(secret.as_deref(), &[], signature.as_deref());
    match verdict {
        SignatureVerdict::Valid => Json(json!({ "challenge": "valid" })).into_response(),
        SignatureVerdict::Unverifiable => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "challenge": "unverifiable",
                "detail": "no WhatsApp app secret is configured, so no signature can be \
                           checked; the verification algorithm is implemented but the key \
                           is issued by the provider and must not live in this repository",
            })),
        )
            .into_response(),
        SignatureVerdict::Invalid => problem(
            StatusCode::UNAUTHORIZED,
            "the supplied signature does not match the body under the configured secret",
        ),
        SignatureVerdict::Missing => problem(
            StatusCode::UNAUTHORIZED,
            format!("no {SIGNATURE_HEADER} header was sent"),
        ),
    }
}

async fn verify_webhook(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> axum::response::Response {
    verify(&headers, state.whatsapp_secret()).await
}

async fn verify_whatsapp_webhook(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> axum::response::Response {
    verify(&headers, state.whatsapp_secret()).await
}

/// Shared receive handler for both webhook paths.
async fn receive(state: SharedState, headers: HeaderMap, body: Bytes) -> axum::response::Response {
    let secret = state.whatsapp_secret();
    let signature = signature_of(&headers);
    let verdict = verify_whatsapp_signature(secret.as_deref(), &body, signature.as_deref());
    match verdict {
        SignatureVerdict::Valid => {}
        SignatureVerdict::Unverifiable => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "no WhatsApp app secret is configured, so an inbound message cannot be \
                 authenticated; recording it would mean accepting unauthenticated traffic",
            )
        }
        SignatureVerdict::Invalid => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "the supplied signature does not match the body under the configured secret",
            )
        }
        SignatureVerdict::Missing => {
            return problem(
                StatusCode::UNAUTHORIZED,
                format!("no {SIGNATURE_HEADER} header was sent"),
            )
        }
    }

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return problem(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("the body is not JSON: {e}"),
            )
        }
    };
    let (message_id, from, text) = match parse_whatsapp_message(&payload) {
        Ok(m) => m,
        Err(detail) => return problem(StatusCode::UNPROCESSABLE_ENTITY, detail),
    };
    let message = ReceivedMessage {
        message_id: message_id.clone(),
        // The sender's number is hashed: the gateway has no need to retain it.
        from_hash: hash_msisdn(&from),
        body: text,
        received_at: chrono::Utc::now().to_rfc3339(),
        signature: signature.unwrap_or_default(),
    };
    state.record_message(message.clone());
    info!(message_id = %message_id, "whatsapp: message received");
    (
        StatusCode::OK,
        Json(json!({
            "acknowledged": true,
            "message_id": message.message_id,
            "from_hash": message.from_hash,
            "received_at": message.received_at,
        })),
    )
        .into_response()
}

async fn receive_message(
    State(state): State<SharedState>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    receive(state, headers, body).await
}

async fn receive_whatsapp_webhook(
    State(state): State<SharedState>,
    headers: HeaderMap,
    body: Bytes,
) -> axum::response::Response {
    receive(state, headers, body).await
}

/// Compute the signature a provider would send. Exposed so a sender on this side
/// of the gateway, and the tests, can produce one.
pub fn sign(secret: &str, body: &[u8]) -> String {
    sign_whatsapp_body(secret, body)
}
