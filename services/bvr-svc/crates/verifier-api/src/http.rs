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

//! Public voter verification handlers (verifier/verify.yaml + ussd.yaml +
//! whatsapp.yaml).
//!
//! Every handler reads or advances [`crate::store::VerifierStore`], and every
//! response shape is taken from the contract rather than invented.
//!
//! The one worth naming: `lookup_voter_registration` returned `registered: false`
//! for every request. The register is held by the registration service, which is
//! not in this workspace. The contract does not offer a 501 on this path, but it
//! does define `not_registered_reason`, whose `roll_not_published` member is
//! exactly this situation -- so the reason is stated rather than implied, and a
//! client can tell "the roll is not published here" from "we looked and you are
//! not on it". The distinction is the whole point of that enum, and losing it is
//! what the previous handler did.

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::store::{
    sign_body, verify_hub_signature, verify_token_matches, verify_ussd_signature,
    SharedVerifierStore, SignatureVerdict,
};

/// Meta's signature header.
const HUB_SIG: &str = "x-hub-signature-256";
/// The telco USSD signature header.
const USSD_SIG: &str = "x-eel-webhook-sig";
/// USSD screens are limited to this many characters by handset firmware.
const USSD_MAX: usize = 182;

pub fn router(state: SharedVerifierStore) -> Router {
    Router::new()
        .route("/api/v1/verifier/lookups", post(lookup_voter_registration))
        .route(
            "/api/v1/verifier/station/{station_code}",
            get(get_polling_station_detail),
        )
        .route("/api/v1/verifier/results/summary", get(get_result_summary))
        .route("/api/v1/verifier/ussd/session", post(handle_ussd_session))
        .route(
            "/api/v1/verifier/whatsapp/webhook",
            get(verify_whatsapp_webhook).post(handle_whatsapp_webhook),
        )
        .with_state(state)
}

/// An `application/problem+json` response.
fn problem(status: StatusCode, title: &str, detail: impl Into<String>) -> Response {
    (
        status,
        Json(json!({
            "type": format!("https://errors.eel.local/verifier/{}", status.as_u16()),
            "title": title,
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
}

/// A header's value as a `String`, if present and valid UTF-8.
fn header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(name).and_then(|v| v.to_str().ok()).map(str::to_string)
}

/// `POST /api/v1/verifier/lookups` — voter registration status by hashed ID.
#[derive(Debug, Deserialize)]
struct VerificationRequest {
    /// 64 lowercase hex characters, per the contract's pattern.
    id_number_hash: String,
    station_hint: Option<String>,
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

async fn lookup_voter_registration(
    State(_state): State<SharedVerifierStore>,
    Json(req): Json<VerificationRequest>,
) -> Response {
    if !is_hex64(&req.id_number_hash) {
        return problem(
            StatusCode::BAD_REQUEST,
            "Invalid id_number_hash",
            "id_number_hash must be 64 lowercase hex characters: SHA-256(salt||national_id) \
             computed on the device. The raw national ID number must never be transmitted.",
        );
    }
    if req.station_hint.as_ref().is_some_and(|h| h.len() > 20) {
        return problem(
            StatusCode::BAD_REQUEST,
            "Invalid station_hint",
            "station_hint is at most 20 characters.",
        );
    }
    // Previously returned `registered: false` for every request, with no reason and
    // no roll timestamp, which read as a lookup that had been performed and had
    // come back negative. That is a definitive-looking answer about a real person
    // on the one endpoint whose whole job is to answer that question.
    //
    // The contract has no 501 on this path, but it does define
    // `not_registered_reason`, and `roll_not_published` is precisely "the roll is
    // not available to this service". Stating it is the difference between a client
    // that can say "check back later" and one that says "you are not registered".
    let now = chrono::Utc::now();
    (
        StatusCode::OK,
        Json(json!({
            "registered": false,
            "lookup_id": uuid::Uuid::now_v7().to_string(),
            "not_registered_reason": "roll_not_published",
            "as_of": now.to_rfc3339(),
        })),
    )
        .into_response()
}

/// `GET /api/v1/verifier/station/{station_code}` — signed-static, public.
async fn get_polling_station_detail(
    State(state): State<SharedVerifierStore>,
    Path(station_code): Path<String>,
) -> Response {
    // Previously `name: null, constituency: null` for any station, which reads as
    // "this station is not on the register" rather than "this service does not hold
    // the station list".
    if station_code.is_empty() || station_code.len() > 20 {
        return problem(StatusCode::NOT_FOUND, "Not found", "no such station_code");
    }
    match state.station(&station_code) {
        Some(station) => Json(station).into_response(),
        None => problem(
            StatusCode::NOT_FOUND,
            "Not found",
            format!("no polling station {station_code} has been published to this service"),
        ),
    }
}

/// `GET /api/v1/verifier/results/summary` — signed-static, public.
async fn get_result_summary(State(state): State<SharedVerifierStore>) -> Response {
    // Previously returned `results: []`, which reads as "no results have been
    // published" -- a claim about an election that nothing here can support.
    //
    // The contract offers 200 or 500 on this path, not 404: the summary is
    // published every two minutes during tallying, so its absence is a fault in the
    // publishing chain rather than a missing resource.
    match state.latest_results_for("KE-GE-2027").or_else(|| state.latest_results_for("")) {
        Some(summary) => Json(summary).into_response(),
        None => problem(
            StatusCode::INTERNAL_SERVER_ERROR,
            "No result summary published",
            "no result summary has been published to this service; it previously returned an \
             empty results array, which reads as an election with no results rather than as \
             this service holding none",
        ),
    }
}

/// `POST /api/v1/verifier/ussd/session` — telco gateway callback.
#[derive(Debug, Deserialize)]
struct UssdRequest {
    session_id: String,
    /// 64 lowercase hex, per the contract.
    msisdn_hash: String,
    /// Accumulated `*`-delimited input; empty on session start.
    text: String,
    /// `*384#`, per the contract's pattern.
    service_code: String,
}

struct UssdReply {
    text: String,
    action: &'static str,
}

impl UssdReply {
    /// Build a reply, refusing to emit a screen the firmware would truncate.
    fn new(text: impl Into<String>, action: &'static str) -> UssdReply {
        let text = text.into();
        debug_assert!(
            text.chars().count() <= USSD_MAX,
            "USSD screen text must fit {USSD_MAX} characters"
        );
        UssdReply { text, action }
    }
}

/// Resolve the menu from the accumulated input alone.
///
/// `USSDRequest.text` is the subscriber's *accumulated* input, asterisk-delimited,
/// so the whole history arrives on every interaction. That means the state is a
/// function of the input and needs no server-side session -- which is what makes
/// this endpoint idempotent per its contract, and what keeps it from holding a
/// session lock across a store lookup.
fn resolve_ussd(state: &SharedVerifierStore, text: &str) -> UssdReply {
    let segments: Vec<&str> = text.split('*').map(str::trim).collect();
    let first = segments.first().copied().unwrap_or_default();
    if first.is_empty() {
        return UssdReply::new(
            "Jibu: 1. Kituo changu cha kupiga kura\n2. Matokeo\n0. Mwisho",
            "continue",
        );
    }
    let rest = segments.get(1).copied().unwrap_or_default();
    match first {
        "0" => UssdReply::new("Karibu tena.", "end"),
        "1" => {
            if rest.is_empty() {
                return UssdReply::new("Andika msimbo wa kituo, au 0 kuacha.", "continue");
            }
            match state.station(rest) {
                Some(st) => UssdReply::new(
                    format!(
                        "Kituo: {}\nJimbo: {}\n0. Mwisho",
                        st.station_name, st.constituency_name
                    ),
                    "end",
                ),
                None => UssdReply::new(
                    format!("{rest} hakikuwa kituo kilichochapishwa. 0. Mwisho"),
                    "end",
                ),
            }
        }
        "2" => {
            if rest.is_empty() {
                return UssdReply::new("Andika msimbo wa constituency, au 0 kuacha.", "continue");
            }
            match state.latest_results_for(rest) {
                Some(r) => UssdReply::new(
                    format!(
                        "{}: {}\nMatokeo: {}\n0. Mwisho",
                        r.election_name, r.leading_candidate.display_name, r.status
                    ),
                    "end",
                ),
                None => UssdReply::new(
                    format!("Hakuna matokeo yaliyochapishwa kwa {rest}. 0. Mwisho"),
                    "end",
                ),
            }
        }
        other => UssdReply::new(
            format!("{other} si mojawepo. Jibu 1, 2 au 0."),
            "continue",
        ),
    }
}

async fn handle_ussd_session(
    State(state): State<SharedVerifierStore>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Previously answered `END` to every caller: the verification line hung up on
    // the first menu, so a citizen with a phone and no smartphone got nothing.
    //
    // The signature is checked before the body is parsed, because a menu is only
    // worth serving to a gateway that authenticated.
    match verify_ussd_signature(
        state.ussd_webhook_secret().as_deref(),
        &body,
        header(&headers, USSD_SIG).as_deref(),
    ) {
        SignatureVerdict::Valid => {}
        SignatureVerdict::Unverifiable => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "Webhook secret not configured",
                "no USSD webhook secret is configured, so this gateway cannot be \
                 authenticated; the verification algorithm is implemented but the key is \
                 issued by the telco and must not live in this repository",
            )
        }
        SignatureVerdict::Invalid => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "Webhook signature invalid",
                format!("the {USSD_SIG} header does not match the body under the configured secret"),
            )
        }
        SignatureVerdict::Missing => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "Webhook signature missing",
                format!("no {USSD_SIG} header was sent"),
            )
        }
    }
    let req: UssdRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return problem(
                StatusCode::BAD_REQUEST,
                "Malformed USSD request",
                format!("the body is not a USSDRequest: {e}"),
            )
        }
    };
    if req.session_id.is_empty() || req.session_id.len() > 128 {
        return problem(
            StatusCode::BAD_REQUEST,
            "Invalid session_id",
            "session_id must be between 1 and 128 characters.",
        );
    }
    if !is_hex64(&req.msisdn_hash) {
        return problem(
            StatusCode::BAD_REQUEST,
            "Invalid msisdn_hash",
            "msisdn_hash must be 64 lowercase hex characters: SHA-256(salt||msisdn) computed \
             by telco middleware. The raw MSISDN must not be transmitted.",
        );
    }
    if req.text.len() > 255 {
        return problem(
            StatusCode::BAD_REQUEST,
            "Invalid text",
            "text is at most 255 characters.",
        );
    }
    if !req.service_code.starts_with('*') || !req.service_code.ends_with('#') {
        return problem(
            StatusCode::BAD_REQUEST,
            "Invalid service_code",
            "service_code must look like *384#.",
        );
    }
    let reply = resolve_ussd(&state, &req.text);
    state.record_ussd_step(
        &req.session_id,
        &req.service_code,
        &req.text,
        &reply.text,
        reply.action,
    );
    (
        StatusCode::OK,
        Json(json!({
            "text": reply.text,
            "action": reply.action,
            "session_id": req.session_id,
        })),
    )
        .into_response()
}

/// `GET /api/v1/verifier/whatsapp/webhook` — Meta's verification challenge.
#[derive(Debug, Deserialize)]
struct HubQuery {
    #[serde(rename = "hub.mode")]
    hub_mode: Option<String>,
    #[serde(rename = "hub.challenge")]
    hub_challenge: Option<String>,
    #[serde(rename = "hub.verify_token")]
    hub_verify_token: Option<String>,
}

async fn verify_whatsapp_webhook(
    State(state): State<SharedVerifierStore>,
    Query(q): Query<HubQuery>,
) -> Response {
    // Previously returned `{"challenge": "verified"}` for any request, comparing
    // nothing. The contract (whatsapp.yaml) says: echo hub.challenge as plain text
    // when hub.mode is "subscribe" and hub.verify_token matches the configured
    // token, otherwise 403.
    if q.hub_mode.as_deref() != Some("subscribe") {
        return problem(
            StatusCode::FORBIDDEN,
            "Verification failed",
            "hub.mode must be 'subscribe'.",
        );
    }
    let (Some(token), Some(challenge)) = (q.hub_verify_token.as_deref(), q.hub_challenge.as_deref())
    else {
        return problem(
            StatusCode::FORBIDDEN,
            "Verification failed",
            "hub.verify_token and hub.challenge are both required.",
        );
    };
    if !verify_token_matches(state.whatsapp_verify_token().as_deref(), token) {
        return problem(
            StatusCode::FORBIDDEN,
            "Verification failed",
            "hub.verify_token did not match the configured token.",
        );
    }
    // The contract asks for the bare challenge as plain text.
    (StatusCode::OK, challenge.to_string()).into_response()
}

/// `POST /api/v1/verifier/whatsapp/webhook` — inbound messages and statuses.
async fn handle_whatsapp_webhook(
    State(state): State<SharedVerifierStore>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Previously answered `{"acknowledged": true}` for any body, signed or not.
    let secret = state.whatsapp_app_secret();
    match verify_hub_signature(secret.as_deref(), &body, header(&headers, HUB_SIG).as_deref()) {
        SignatureVerdict::Valid => {}
        SignatureVerdict::Unverifiable => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "App secret not configured",
                "no WhatsApp app secret is configured, so an inbound message cannot be \
                 authenticated; acknowledging it would mean accepting unauthenticated traffic",
            )
        }
        SignatureVerdict::Invalid => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "Signature invalid",
                format!("the {HUB_SIG} header does not match the body under the configured secret"),
            )
        }
        SignatureVerdict::Missing => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "Signature missing",
                format!("no {HUB_SIG} header was sent"),
            )
        }
    }
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return problem(
                StatusCode::BAD_REQUEST,
                "Malformed payload",
                format!("the body is not JSON: {e}"),
            )
        }
    };
    state.record_webhook(payload);
    // Meta requires a 200 within 20 s and reads nothing else from the body.
    StatusCode::OK.into_response()
}

/// Compute the signature Meta would send, for tests and for a sender here.
pub fn sign(secret: &str, body: &[u8]) -> String {
    sign_body(secret, body)
}
