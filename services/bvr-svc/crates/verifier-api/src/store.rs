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

//! In-memory state for the public voter-verification surface.
//!
//! Every handler here read or built its answer, and four of them were worse than
//! dead:
//!
//! - `verify_whatsapp_webhook` returned `{"challenge": "verified"}` for **any**
//!   request. The contract (whatsapp.yaml) says the handler must echo
//!   `hub.challenge` as plain text, and only when `hub.verify_token` matches the
//!   configured token -- otherwise 403. It compared nothing.
//! - `handle_whatsapp_webhook` answered `{"acknowledged": true}` for any body,
//!   signed or not. Meta signs with HMAC-SHA256 over the raw body; the endpoint
//!   read no signature.
//! - `lookup_voter_registration` returned `registered: false` for every request.
//! - `get_polling_station_detail` returned `name: null, constituency: null` for
//!   any station, which reads as "not on the register" rather than "not held here".
//! - `get_result_summary` returned `results: []`, which reads as "no results
//!   published" -- a claim about an election.
//! - `handle_ussd_session` answered `END` to every caller: the line hung up on the
//!   first menu.
//!
//! The response shapes below are taken from the contract rather than invented:
//! `StationDetail` and `ResultSummary` from verify.yaml, `USSDResponse` from
//! ussd.yaml. The USSD request carries **accumulated** `*`-delimited input, so the
//! menu state is derived from the input itself and the endpoint is idempotent
//! without a server-side session -- which also means there is no lock held across
//! the store lookup.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use eel_crypto_rs::constant_time::ct_eq;

type HmacSha256 = Hmac<Sha256>;

/// GPS coordinates, WGS84. Required by `StationDetail`.
#[derive(Clone, Debug, Serialize)]
pub struct Coordinates {
    pub lat: f64,
    pub lng: f64,
}

/// `StationDetail` from verify.yaml. All fields class C; no PII.
#[derive(Clone, Debug, Serialize)]
pub struct StationDetail {
    pub station_code: String,
    pub station_name: String,
    pub county_code: String,
    pub county_name: String,
    pub constituency_code: String,
    pub constituency_name: String,
    pub ward_name: String,
    pub registered_voters: u64,
    pub coordinates: Coordinates,
    pub streams: Option<u32>,
    pub address: Option<String>,
    pub accessibility_features: Option<Vec<String>>,
    pub opening_time: Option<String>,
    pub closing_time: Option<String>,
}

/// The leading candidate in a `ResultSummary`.
#[derive(Clone, Debug, Serialize)]
pub struct LeadingCandidate {
    pub display_name: String,
    pub party_acronym: String,
    pub votes: u64,
    pub vote_pct: f64,
}

/// One row of `top_candidates`.
#[derive(Clone, Debug, Serialize)]
pub struct TopCandidate {
    pub display_name: String,
    pub party_acronym: String,
    pub votes: u64,
    pub vote_pct: f64,
    pub position: u32,
}

/// `ResultSummary` from verify.yaml. All class C; no PII.
#[derive(Clone, Debug, Serialize)]
pub struct ResultSummary {
    pub election_id: String,
    pub election_name: String,
    /// One of polling, tallying, provisional, certified, gazetted.
    pub status: String,
    pub total_valid_votes: u64,
    pub total_registered_voters: Option<u64>,
    pub turnout_pct: f64,
    pub leading_candidate: LeadingCandidate,
    pub top_candidates: Option<Vec<TopCandidate>>,
    pub as_of: String,
}

/// One USSD exchange, kept for audit.
#[derive(Clone, Debug, Serialize)]
pub struct UssdAudit {
    pub session_id: String,
    pub service_code: String,
    pub input: String,
    pub response_text: String,
    pub action: String,
    pub at: String,
}

/// Shared state for the verification surface.
#[derive(Default)]
pub struct VerifierStore {
    stations: Mutex<HashMap<String, StationDetail>>,
    results: Mutex<Vec<ResultSummary>>,
    ussd_audit: Mutex<Vec<UssdAudit>>,
    webhooks: Mutex<Vec<serde_json::Value>>,
    /// Meta's `hub.verify_token`, compared in constant time.
    whatsapp_verify_token: Mutex<Option<String>>,
    /// Meta's app secret, which keys the `X-Hub-Signature-256` HMAC.
    whatsapp_app_secret: Mutex<Option<String>>,
    /// The telco USSD webhook secret, which keys `X-Eel-Webhook-Sig`.
    ussd_webhook_secret: Mutex<Option<String>>,
}

/// The handle threaded through the router.
pub type SharedVerifierStore = Arc<VerifierStore>;

impl VerifierStore {
    /// Construct an unconfigured store.
    ///
    /// Returns the value rather than the handle so a caller can configure it
    /// before sharing it.
    pub fn new() -> Self {
        Self::default()
    }

    /// The shared handle for a configured store.
    pub fn shared(self) -> SharedVerifierStore {
        Arc::new(self)
    }

    /// Set Meta's webhook verification token.
    pub fn with_whatsapp_verify_token(self, token: impl Into<String>) -> Self {
        *self.whatsapp_verify_token.lock().expect("lock poisoned") = Some(token.into());
        self
    }

    /// Set Meta's app secret, for `X-Hub-Signature-256`.
    pub fn with_whatsapp_app_secret(self, secret: impl Into<String>) -> Self {
        *self.whatsapp_app_secret.lock().expect("lock poisoned") = Some(secret.into());
        self
    }

    /// Set the telco USSD webhook secret, for `X-Eel-Webhook-Sig`.
    pub fn with_ussd_webhook_secret(self, secret: impl Into<String>) -> Self {
        *self.ussd_webhook_secret.lock().expect("lock poisoned") = Some(secret.into());
        self
    }

    /// The configured Meta verification token, if any.
    pub fn whatsapp_verify_token(&self) -> Option<String> {
        self.whatsapp_verify_token.lock().expect("lock poisoned").clone()
    }

    /// The configured Meta app secret, if any.
    pub fn whatsapp_app_secret(&self) -> Option<String> {
        self.whatsapp_app_secret.lock().expect("lock poisoned").clone()
    }

    /// The configured telco USSD webhook secret, if any.
    pub fn ussd_webhook_secret(&self) -> Option<String> {
        self.ussd_webhook_secret.lock().expect("lock poisoned").clone()
    }

    /// Register a polling station.
    pub fn register_station(&self, station: StationDetail) -> StationDetail {
        self.stations
            .lock()
            .expect("lock poisoned")
            .insert(station.station_code.clone(), station.clone());
        station
    }

    /// A station by code.
    pub fn station(&self, code: &str) -> Option<StationDetail> {
        self.stations.lock().expect("lock poisoned").get(code).cloned()
    }

    /// Every station, by code.
    pub fn stations(&self) -> Vec<StationDetail> {
        let guard = self.stations.lock().expect("lock poisoned");
        let mut items: Vec<StationDetail> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.station_code.cmp(&b.station_code));
        items
    }

    /// Record a published result summary.
    pub fn record_results(&self, summary: ResultSummary) {
        self.results.lock().expect("lock poisoned").push(summary);
    }

    /// The most recently published summary for a constituency, if there is one.
    pub fn latest_results_for(&self, constituency_code: &str) -> Option<ResultSummary> {
        self.results
            .lock()
            .expect("lock poisoned")
            .iter()
            .rev()
            .find(|r| r.election_id.eq_ignore_ascii_case(constituency_code))
            .cloned()
    }

    /// Every recorded inbound webhook, oldest first.
    pub fn webhooks(&self) -> Vec<serde_json::Value> {
        self.webhooks.lock().expect("lock poisoned").clone()
    }

    /// Record an authenticated inbound webhook.
    pub fn record_webhook(&self, value: serde_json::Value) {
        self.webhooks.lock().expect("lock poisoned").push(value);
    }

    /// Record one USSD exchange for audit.
    pub fn record_ussd_step(
        &self,
        session_id: &str,
        service_code: &str,
        input: &str,
        response_text: &str,
        action: &str,
    ) {
        self.ussd_audit.lock().expect("lock poisoned").push(UssdAudit {
            session_id: session_id.to_string(),
            service_code: service_code.to_string(),
            input: input.to_string(),
            response_text: response_text.to_string(),
            action: action.to_string(),
            at: chrono::Utc::now().to_rfc3339(),
        });
    }

    /// Every USSD exchange, oldest first.
    pub fn ussd_audit(&self) -> Vec<UssdAudit> {
        self.ussd_audit.lock().expect("lock poisoned").clone()
    }
}

/// The outcome of a webhook signature check.
#[derive(Debug, PartialEq, Eq)]
pub enum SignatureVerdict {
    /// The signature matches the body under the configured secret.
    Valid,
    /// A signature was supplied and does not match.
    Invalid,
    /// A signature was supplied but no secret is configured to check it against.
    ///
    /// Deliberately distinct from `Valid`. Reporting "verified" without a secret is
    /// the fabrication the previous handler performed on every request.
    Unverifiable,
    /// No signature was supplied at all.
    Missing,
}

/// Verify an `X-Hub-Signature-256` header over the **raw** body.
///
/// The header format is `sha256=<lowercase hex>`; the digest is
/// HMAC-SHA256(raw_body) keyed by the app secret. The raw body is required
/// because re-serialising parsed JSON would change the bytes and the digest would
/// no longer match.
pub fn verify_hub_signature(
    secret: Option<&str>,
    raw_body: &[u8],
    header: Option<&str>,
) -> SignatureVerdict {
    // The secret is checked first: a missing secret is a configuration problem
    // that no header can fix.
    let Some(secret) = secret else {
        return SignatureVerdict::Unverifiable;
    };
    let Some(header) = header.map(str::trim).filter(|h| !h.is_empty()) else {
        return SignatureVerdict::Missing;
    };
    let Some(digest) = header.strip_prefix("sha256=") else {
        return SignatureVerdict::Invalid;
    };
    let Ok(expected) = hex::decode(digest) else {
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

/// Verify an `X-Eel-Webhook-Sig` header, which uses the same `sha256=<hex>` form.
pub fn verify_ussd_signature(
    secret: Option<&str>,
    raw_body: &[u8],
    header: Option<&str>,
) -> SignatureVerdict {
    verify_hub_signature(secret, raw_body, header)
}

/// Compute the signature Meta would send, for tests and for a sender here.
pub fn sign_body(secret: &str, raw_body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC-SHA256 accepts a key of any length");
    mac.update(raw_body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

/// Compare a presented `hub.verify_token` against the configured one.
///
/// Constant time, so a caller cannot learn the token by measuring. Returns `false`
/// when no token is configured, which is why an unconfigured deployment answers
/// 403 rather than accepting every challenge.
pub fn verify_token_matches(configured: Option<&str>, presented: &str) -> bool {
    match configured {
        Some(expected) => ct_eq(expected.as_bytes(), presented.trim().as_bytes()),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_signature_under_the_configured_secret_verifies() {
        let body = br#"{"entry":[]}"#;
        let sig = sign_body("s3cret", body);
        assert_eq!(verify_hub_signature(Some("s3cret"), body, Some(&sig)), SignatureVerdict::Valid);
    }

    #[test]
    fn a_signature_over_a_different_body_does_not_verify() {
        let sig = sign_body("s3cret", br#"{"a":1}"#);
        assert_eq!(
            verify_hub_signature(Some("s3cret"), br#"{"a":2}"#, Some(&sig)),
            SignatureVerdict::Invalid
        );
    }

    #[test]
    fn no_secret_is_unverifiable_rather_than_valid() {
        let sig = sign_body("s3cret", b"{}");
        assert_eq!(verify_hub_signature(None, b"{}", Some(&sig)), SignatureVerdict::Unverifiable);
    }

    #[test]
    fn no_signature_is_missing() {
        assert_eq!(verify_hub_signature(Some("s"), b"{}", None), SignatureVerdict::Missing);
    }

    #[test]
    fn a_header_without_the_sha256_prefix_does_not_verify() {
        let body = b"{}";
        let bare = sign_body("s", body).trim_start_matches("sha256=").to_string();
        assert_eq!(verify_hub_signature(Some("s"), body, Some(&bare)), SignatureVerdict::Invalid);
    }

    #[test]
    fn the_token_must_match_and_one_must_be_configured() {
        assert!(verify_token_matches(Some("tok"), "tok"));
        assert!(!verify_token_matches(Some("tok"), "other"));
        // An unconfigured deployment must not accept every challenge.
        assert!(!verify_token_matches(None, "anything"));
    }
}
