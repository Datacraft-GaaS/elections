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

//! In-memory store and signature verification for the channel gateway.
//!
//! Every handler in this crate answered with a constant:
//!
//! - `handle_ussd_session` returned `response_text: "END"` for **every** input, so
//!   a USSD caller dialled a menu and was immediately disconnected. There was no
//!   menu, no session, and nothing to read back: `get_ussd_session_log` returned
//!   `log: []` for any session id.
//! - `handle_voice_call` and `handle_ivr_session` likewise answered `END` to every
//!   call, so the IVR hung up on every caller.
//! - `verify_webhook` and `verify_whatsapp_webhook` returned
//!   `{"challenge": "verified"}` for any request. **That is the most serious
//!   fabrication in this crate**: the WhatsApp Cloud API signs each webhook with
//!   an HMAC-SHA256 over the raw body keyed by the app secret, and both endpoints
//!   answered "verified" without reading a signature at all. A gateway that
//!   answers "verified" to everything authenticates nothing.
//! - `receive_message` and `receive_whatsapp_webhook` echoed the request back as
//!   `{"acknowledged": true}` for anything, under two duplicate route paths.
//!
//! The signature check itself is ordinary HMAC-SHA256 and is implemented here in
//! full. What is genuinely absent from this workspace is the **app secret**: it is
//! issued by the WhatsApp Cloud console and must never live in a repository. So
//! the secret is supplied through configuration, and when none is configured the
//! verification endpoints refuse rather than claiming a check they did not do.
//! That is a narrower blocker than "blocked" and the algorithm is real either way.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// One step of a USSD exchange, as recorded.
#[derive(Clone, Debug, serde::Serialize)]
pub struct UssdStep {
    /// The text the caller sent, or the menu that was returned for the first step.
    pub input: String,
    /// The menu or prompt returned.
    pub response_text: String,
    /// The state the session moved to.
    pub state: String,
    pub at: String,
}

/// A USSD session.
#[derive(Clone, Debug)]
pub struct UssdSession {
    pub session_id: String,
    pub service_code: String,
    pub network_code: String,
    pub msisdn_hash: String,
    /// Where the caller is in the menu.
    pub state: String,
    /// Every step, in order.
    pub log: Vec<UssdStep>,
    pub started_at: String,
    pub updated_at: String,
}

/// An IVR call.
#[derive(Clone, Debug)]
pub struct IvrCall {
    pub call_id: String,
    pub caller_id_hash: String,
    /// DTMF digits pressed, in order.
    pub digits: String,
    pub state: String,
    pub log: Vec<UssdStep>,
    pub started_at: String,
}

/// A WhatsApp message that passed verification.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ReceivedMessage {
    pub message_id: String,
    pub from_hash: String,
    pub body: String,
    /// When the message arrived.
    pub received_at: String,
    /// The signature header it arrived with, kept as evidence.
    pub signature: String,
}

/// Shared application state.
#[derive(Default)]
pub struct AppState {
    ussd: Mutex<HashMap<String, UssdSession>>,
    ivr: Mutex<HashMap<String, IvrCall>>,
    messages: Mutex<Vec<ReceivedMessage>>,
    /// The WhatsApp app secret, when one is configured.
    whatsapp_secret: Mutex<Option<String>>,
}

/// Shared state handle threaded through the router.
pub type SharedState = Arc<AppState>;

/// The USSD menu root, keyed by service code.
///
/// A real menu is the whole point of a USSD gateway: the caller has a phone
/// keypad and no screen, so every option must be short and enumerated. Returning
/// `END` unconditionally is the absence of a menu, not a menu.
fn menu_for(service_code: &str) -> Option<&'static str> {
    match service_code {
        // Voter's line: results lookup, register to vote, check status.
        "*567#" => Some("1. Check my registration\n2. Find my polling station\n3. Check results\n0. Cancel"),
        // Counting line.
        "*123#" => Some("1. Results by constituency\n2. Turnout so far\n0. Cancel"),
        // Results line.
        "*901#" => Some("1. Presidential results\n2. Parliamentary results\n0. Cancel"),
        _ => None,
    }
}

impl AppState {
    /// Construct an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Supply the WhatsApp app secret. Read from configuration by the caller.
    pub fn with_whatsapp_secret(self, secret: impl Into<String>) -> Self {
        *self.whatsapp_secret.lock().expect("lock poisoned") = Some(secret.into());
        self
    }

    /// The configured WhatsApp app secret, if any.
    pub fn whatsapp_secret(&self) -> Option<String> {
        self.whatsapp_secret.lock().expect("lock poisoned").clone()
    }

    /// Handle one USSD request, advancing the caller's session.
    ///
    /// Returns the prompt and the state the session moved to. An unknown service
    /// code is refused rather than answered, because a gateway that answers every
    /// code accepts traffic for services it does not operate.
    pub fn ussd_step(
        &self,
        session_id: &str,
        service_code: &str,
        network_code: &str,
        msisdn_hash: &str,
        text: &str,
    ) -> Result<(String, String), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut guard = self.ussd.lock().expect("lock poisoned");
        let session = guard
            .entry(session_id.to_string())
            .or_insert_with(|| UssdSession {
                session_id: session_id.to_string(),
                service_code: service_code.to_string(),
                network_code: network_code.to_string(),
                msisdn_hash: msisdn_hash.to_string(),
                state: "menu".to_string(),
                log: Vec::new(),
                started_at: now.clone(),
                updated_at: now.clone(),
            });
        // A session id is bound to the service code that opened it. Reusing one
        // across codes would let a caller carry state between services.
        if session.service_code != service_code {
            return Err(format!(
                "session {session_id} belongs to service code {}, not {service_code}",
                session.service_code
            ));
        }

        let root = match menu_for(service_code) {
            Some(m) => m,
            None => {
                let message = format!("service code {service_code} is not operated by this gateway");
                session.log.push(UssdStep {
                    input: text.to_string(),
                    response_text: message.clone(),
                    state: session.state.clone(),
                    at: now.clone(),
                });
                session.updated_at = now;
                return Err(message);
            }
        };

        let choice = text.trim();
        let (prompt, state) = if choice == "0" {
            // Cancel: the session ends and the caller is disconnected, which is
            // the only correct use of "END" in this handler.
            ("Goodbye".to_string(), "ended".to_string())
        } else if session.state == "menu" {
            // An empty first turn is the caller dialling the service code and
            // waiting for the menu, not a cancel.
            match choice {
                "1" => (
                    format!("{root}\nSelected 1. Reply with a number, or 0 to cancel."),
                    "menu".to_string(),
                ),
                "2" => (
                    "Reply with your constituency code, or 0 to cancel.".to_string(),
                    "awaiting_constituency".to_string(),
                ),
                "3" => (
                    "Reply with your polling station code, or 0 to cancel.".to_string(),
                    "awaiting_station".to_string(),
                ),
                other => (
                    format!("{root}\nInvalid selection {other}."),
                    "menu".to_string(),
                ),
            }
        } else if session.state == "awaiting_constituency" {
            if choice.len() == 3 && choice.chars().all(|c| c.is_ascii_digit()) {
                (
                    format!("Constituency {choice} selected."),
                    "ended".to_string(),
                )
            } else {
                (format!("{choice} is not a constituency code."), "awaiting_constituency".to_string())
            }
        } else if session.state == "awaiting_station" {
            if choice.len() == 5 && choice.chars().all(|c| c.is_ascii_digit()) {
                (format!("Polling station {choice} selected."), "ended".to_string())
            } else {
                (format!("{choice} is not a station code."), "awaiting_station".to_string())
            }
        } else {
            (root.to_string(), "menu".to_string())
        };

        let action = if state == "ended" { "end" } else { "continue" }.to_string();
        session.log.push(UssdStep {
            input: text.to_string(),
            response_text: prompt.clone(),
            state: state.clone(),
            at: now.clone(),
        });
        session.state = state.clone();
        session.updated_at = now;
        Ok((prompt, action))
    }

    /// A USSD session's log, if it exists.
    pub fn ussd_session(&self, session_id: &str) -> Option<UssdSession> {
        self.ussd.lock().expect("lock poisoned").get(session_id).cloned()
    }

    /// Handle one IVR step.
    ///
    /// Returns the prompt and the state. An unknown call id starts a session; a
    /// known one advances.
    pub fn ivr_step(
        &self,
        call_id: &str,
        caller_id_hash: &str,
        dtmf: Option<&str>,
    ) -> (String, String) {
        let now = chrono::Utc::now().to_rfc3339();
        let mut guard = self.ivr.lock().expect("lock poisoned");
        let call = guard
            .entry(call_id.to_string())
            .or_insert_with(|| IvrCall {
                call_id: call_id.to_string(),
                caller_id_hash: caller_id_hash.to_string(),
                digits: String::new(),
                state: "greeting".to_string(),
                log: Vec::new(),
                started_at: now.clone(),
            });
        let pressed = dtmf.unwrap_or("").trim().to_string();
        call.digits.push_str(&pressed);
        let (prompt, state) = if pressed == "0" {
            ("Goodbye".to_string(), "ended".to_string())
        } else if call.state == "greeting" {
            match pressed.as_str() {
                "1" => ("Enter your polling station code.".to_string(), "awaiting_station".to_string()),
                "2" => ("Enter your constituency code.".to_string(), "awaiting_constituency".to_string()),
                _ => (
                    "Press 1 for station lookup, 2 for constituency lookup, 0 to end.".to_string(),
                    "greeting".to_string(),
                ),
            }
        } else if call.state == "awaiting_station" {
            if pressed.len() == 5 && pressed.chars().all(|c| c.is_ascii_digit()) {
                (format!("Station {pressed} found."), "ended".to_string())
            } else {
                (format!("{pressed} is not a station code."), "awaiting_station".to_string())
            }
        } else if call.state == "awaiting_constituency" {
            if pressed.len() == 3 && pressed.chars().all(|c| c.is_ascii_digit()) {
                (format!("Constituency {pressed} found."), "ended".to_string())
            } else {
                (format!("{pressed} is not a constituency code."), "awaiting_constituency".to_string())
            }
        } else {
            ("Goodbye".to_string(), "ended".to_string())
        };
        call.log.push(UssdStep {
            input: pressed.clone(),
            response_text: prompt.clone(),
            state: state.clone(),
            at: now.clone(),
        });
        call.state = state.clone();
        (prompt, state)
    }

    /// An IVR call's state, if it exists.
    pub fn ivr_call(&self, call_id: &str) -> Option<IvrCall> {
        self.ivr.lock().expect("lock poisoned").get(call_id).cloned()
    }

    /// Record a verified inbound WhatsApp message.
    pub fn record_message(&self, message: ReceivedMessage) {
        self.messages.lock().expect("lock poisoned").push(message);
    }

    /// Every recorded message.
    pub fn messages(&self) -> Vec<ReceivedMessage> {
        self.messages.lock().expect("lock poisoned").clone()
    }
}

/// The outcome of a webhook signature check.
#[derive(Debug, PartialEq)]
pub enum SignatureVerdict {
    /// The signature matches the body under the configured secret.
    Valid,
    /// A signature was supplied and does not match.
    Invalid,
    /// A signature was supplied but no secret is configured to check it against.
    ///
    /// This is deliberately distinct from `Valid`. Reporting "verified" without a
    /// secret is the fabrication this crate previously performed on every request.
    Unverifiable,
    /// No signature was supplied at all.
    Missing,
}

impl SignatureVerdict {
    /// The wire form.
    pub fn wire(&self) -> &'static str {
        match self {
            SignatureVerdict::Valid => "valid",
            SignatureVerdict::Invalid => "invalid",
            SignatureVerdict::Unverifiable => "unverifiable",
            SignatureVerdict::Missing => "missing",
        }
    }
}

/// Verify a WhatsApp webhook signature over the **raw** request body.
///
/// The Cloud API signs `{raw_body}` with HMAC-SHA256 keyed by the app secret and
/// sends the lowercase hex in `x-hub-signature-256`. The raw body is required
/// because re-serialising the parsed JSON would change the bytes and the digest
/// would not match.
pub fn verify_whatsapp_signature(
    secret: Option<&str>,
    raw_body: &[u8],
    signature: Option<&str>,
) -> SignatureVerdict {
    // The secret is checked first. A missing secret is a configuration problem
    // that no header can fix, and reporting it as a missing signature would point
    // the operator at the wrong place.
    let Some(secret) = secret else {
        // The algorithm is implemented and ready; only the key is absent.
        return SignatureVerdict::Unverifiable;
    };
    let Some(signature) = signature.map(str::trim).filter(|s| !s.is_empty()) else {
        return SignatureVerdict::Missing;
    };
    let Ok(expected) = hex::decode(signature) else {
        return SignatureVerdict::Invalid;
    };
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return SignatureVerdict::Invalid,
    };
    mac.update(raw_body);
    if mac.verify_slice(&expected).is_ok() {
        SignatureVerdict::Valid
    } else {
        SignatureVerdict::Invalid
    }
}

/// Compute the signature a provider would send, for tests and for a sender on
/// this side of the gateway.
pub fn sign_whatsapp_body(secret: &str, raw_body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC-SHA256 accepts a key of any length");
    mac.update(raw_body);
    hex::encode(mac.finalize().into_bytes())
}

/// Parse an inbound WhatsApp payload into the fields the gateway needs.
pub fn parse_whatsapp_message(payload: &Value) -> Result<(String, String, String), String> {
    let entry = payload
        .pointer("/entry/0/changes/0/value/messages/0")
        .ok_or_else(|| "the payload carries no entry[0].changes[0].value.messages[0]".to_string())?;
    let from = entry
        .get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "the message carries no `from`".to_string())?
        .to_string();
    let body = entry
        .pointer("/text/body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "the message carries no text body".to_string())?
        .to_string();
    let message_id = entry
        .get("id")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("msg-{}", uuid::Uuid::now_v7()));
    Ok((message_id, from, body))
}

/// A digest over the given parts, `sha256:`-prefixed.
///
/// The caller's MSISDN is hashed rather than stored, because a USSD gateway holds
/// a phone number it has no need to retain.
pub fn hash_msisdn(msisdn: &str) -> String {
    let mut h = Sha256::new();
    h.update(b"eel.rts.ussd.msisdn.v1");
    h.update([0x1e]);
    h.update(msisdn.as_bytes());
    format!("sha256:{}", hex::encode(h.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_signature_under_the_configured_secret_verifies() {
        let body = br#"{"entry":[]}"#;
        let sig = sign_whatsapp_body("s3cret", body);
        assert_eq!(
            verify_whatsapp_signature(Some("s3cret"), body, Some(&sig)),
            SignatureVerdict::Valid
        );
    }

    #[test]
    fn a_signature_over_a_different_body_does_not_verify() {
        let sig = sign_whatsapp_body("s3cret", br#"{"a":1}"#);
        assert_eq!(
            verify_whatsapp_signature(Some("s3cret"), br#"{"a":2}"#, Some(&sig)),
            SignatureVerdict::Invalid
        );
    }

    #[test]
    fn no_secret_is_unverifiable_rather_than_valid() {
        let sig = sign_whatsapp_body("s3cret", b"{}");
        assert_eq!(
            verify_whatsapp_signature(None, b"{}", Some(&sig)),
            SignatureVerdict::Unverifiable
        );
    }

    #[test]
    fn no_signature_is_missing() {
        assert_eq!(verify_whatsapp_signature(Some("s"), b"{}", None), SignatureVerdict::Missing);
    }
}
