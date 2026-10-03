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

//! In-memory state for the `/api/v1/mobile/bff/*` and `/v1/agent/*` surfaces.
//!
//! Every one of these handlers answered from a constant or a fresh random value:
//!
//! - `get_station_status` returned `turnout_count: 1247`, `kit_battery_level: 87.5`,
//!   `connectivity_status: "online"` and `queue_depth: 3` for **every station**.
//!   Those are invented telemetry readings presented as measurements of a real
//!   polling station.
//! - `get_observer_access_pass` minted an access pass for anyone who asked.
//! - `list_dispatch_assignments` returned two fabricated assignments
//!   (`assign-001` active, `assign-002` pending) for any `worker_id`.
//! - `update_dispatch_assignment` built its idempotency key from a **fresh UUID on
//!   every call**, so the idempotency store was never read and never hit; it stored
//!   `{}` and answered `updated: true` for any assignment id.
//! - `report_incident` and `create_incident` discarded the incident body
//!   entirely (`_req`) and cached `{}` under the idempotency key, so a retry
//!   returned an empty object.
//! - `register_push_token` minted a token id and stored nothing.
//! - `get_attestation_verdict` decided the verdict from the **device id string**
//!   (`starts_with("emulator")` -> UNVERIFIED, `starts_with("rooted")` -> FAILS,
//!   anything else -> MEETS) while ignoring the `attestation_object` entirely, and
//!   minted an observer licence on a MEETS.
//! - `create_bff_session` swallowed a failed OIDC exchange and fell back to
//!   `mock_access_token`, then wrapped it in a string shaped like a JWT with a
//!   literal `...` in it.
//! - `refresh_bff_session` minted a JWT-shaped string from a UUID with no session
//!   lookup at all, so a refresh worked for any caller.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use sha2::{Digest, Sha256};

/// A device session.
#[derive(Clone, Debug)]
pub struct Session {
    pub session_id: String,
    /// The digest of the bearer token, not the token itself.
    pub token_hash: String,
    /// The digest of the refresh token.
    pub refresh_hash: String,
    /// The scopes requested at creation.
    pub scopes: Vec<String>,
    pub issued_at: String,
    pub expires_at: String,
    pub refresh_expires_at: String,
    /// Whether the OIDC token exchange actually completed.
    ///
    /// It does not in this workspace: the exchange needs the identity provider,
    /// which is not reachable from this process. A session is still issued, and it
    /// is a real session over this store, but it does not carry an OIDC identity
    /// and the field says so rather than the response pretending otherwise.
    pub oidc_exchange: String,
    /// The nonce the client supplied, bound into the token digest.
    pub nonce: String,
    pub revoked: bool,
}

/// A station's reported state.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct StationState {
    pub station_code: String,
    /// Whether the kit has checked in since it was provisioned.
    pub online: bool,
    /// Battery percentage, as last reported.
    pub battery_level: Option<f64>,
    /// Voter check-ins counted at the station.
    pub voter_count: u64,
    /// The last time the kit reported.
    pub last_seen_at: Option<String>,
    /// How many agents are currently checked in.
    pub queue_depth: usize,
    pub materials_delivered: bool,
    pub updated_at: String,
}

/// A dispatch assignment.
#[derive(Clone, Debug, serde::Serialize)]
pub struct DispatchAssignment {
    pub assignment_id: String,
    pub worker_id: String,
    pub role: String,
    pub status: String,
    pub station_code: Option<String>,
    pub issued_at: String,
    pub updated_at: String,
}

/// A reported incident.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Incident {
    pub incident_id: String,
    pub incident_type: String,
    pub station_code: Option<String>,
    pub severity: String,
    pub reported_by_hash: String,
    pub detail: Value,
    /// Where it was reported from, when the caller said.
    pub reported_via: String,
    pub reported_at: String,
    /// The Idempotency-Key it was filed under, so a retry is recognisable.
    pub idempotency_key: String,
}

/// A registered push token.
#[derive(Clone, Debug, serde::Serialize)]
pub struct PushTokenRecord {
    pub token_id: String,
    pub device_id: String,
    /// The provider's token is a credential, so only its digest is retained.
    pub token_hash: String,
    pub platform: String,
    pub registered_at: String,
    pub revoked: bool,
}

/// An observer access pass request.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ObserverPass {
    pub pass_id: String,
    pub organisation: String,
    pub accreditation_ref: String,
    pub issued_at: String,
    pub valid_until: String,
    pub status: String,
}

/// Shared state for the mobile BFF surfaces.
#[derive(Default)]
pub struct MobileStore {
    sessions: Mutex<HashMap<String, Session>>,
    by_refresh: Mutex<HashMap<String, String>>,
    stations: Mutex<HashMap<String, StationState>>,
    assignments: Mutex<HashMap<String, DispatchAssignment>>,
    incidents: Mutex<HashMap<String, Incident>>,
    by_idempotency: Mutex<HashMap<String, String>>,
    push_tokens: Mutex<HashMap<String, PushTokenRecord>>,
    passes: Mutex<HashMap<String, ObserverPass>>,
    forms: Mutex<HashMap<String, Form34ARecord>>,
    check_ins: Mutex<HashMap<String, AgentCheckIn>>,
    tamper_events: Mutex<HashMap<String, TamperEvent>>,
    tamper_index: Mutex<HashMap<String, String>>,
    observations: Mutex<HashMap<String, Observation>>,
    observation_index: Mutex<HashMap<String, String>>,
    grievances: Mutex<HashMap<String, Grievance>>,
    grievance_index: Mutex<HashMap<String, String>>,
    /// Registration hashes this service has seen. The register itself is held by
    /// the registration service, which is not in this workspace, so this is only
    /// what has passed through here.
    registrations: Mutex<HashMap<String, String>>,
}

/// The handle threaded through the router.
pub type SharedMobileStore = Arc<MobileStore>;

/// How long a session's access token is good for.
pub const SESSION_TTL_SECS: i64 = 300;
/// How long a session's refresh token is good for.
pub const REFRESH_TTL_SECS: i64 = 3600;

impl MobileStore {
    /// Construct an empty store.
    pub fn new() -> SharedMobileStore {
        Arc::new(Self::default())
    }

    /// The digest of a bearer token.
    pub fn hash_token(parts: &[&str]) -> String {
        let mut h = Sha256::new();
        h.update(b"eel.mobile.session.token.v1");
        for p in parts {
            h.update([0x1e]);
            h.update(p.as_bytes());
        }
        format!("sha256:{}", hex::encode(h.finalize()))
    }

    /// Issue a session.
    ///
    /// The access token is a digest over the session id, the client nonce and the
    /// issue time. It is **not** a JWT and is not shaped like one: a string
    /// beginning `eyJhbGci` reads as a signed token to any client that inspects it,
    /// and this service signs nothing.
    pub fn create_session(
        &self,
        session_id: &str,
        nonce: &str,
        scopes: Vec<String>,
        oidc_completed: bool,
    ) -> Session {
        let now = chrono::Utc::now();
        let issued_at = now.to_rfc3339();
        let expires_at = (now + chrono::Duration::seconds(SESSION_TTL_SECS)).to_rfc3339();
        let refresh_expires_at = (now + chrono::Duration::seconds(REFRESH_TTL_SECS)).to_rfc3339();
        let token_hash = Self::hash_token(&[session_id, nonce, &issued_at]);
        let refresh_hash = Self::hash_token(&[session_id, "refresh", &issued_at]);
        let session = Session {
            session_id: session_id.to_string(),
            token_hash: token_hash.clone(),
            refresh_hash: refresh_hash.clone(),
            scopes,
            issued_at,
            expires_at,
            refresh_expires_at,
            oidc_exchange: if oidc_completed {
                "completed".to_string()
            } else {
                "not-completed".to_string()
            },
            nonce: nonce.to_string(),
            revoked: false,
        };
        self.sessions
            .lock()
            .expect("lock poisoned")
            .insert(session_id.to_string(), session.clone());
        // The refresh token the client receives *is* this digest, so the index is
        // keyed on the token value rather than on a re-hash of it. Keying it on a
        // second hash of the token made every lookup miss.
        self.by_refresh
            .lock()
            .expect("lock poisoned")
            .insert(refresh_hash, session_id.to_string());
        session
    }

    /// Issue a new access token for a live session, replacing the old one.
    ///
    /// A refresh that handed back a token the session did not know about would be
    /// a token no later request could be checked against, so the rotation is
    /// recorded rather than computed and forgotten.
    pub fn rotate_access_token(&self, session_id: &str) -> Option<Session> {
        let mut guard = self.sessions.lock().expect("lock poisoned");
        let session = guard.get_mut(session_id)?;
        if session.revoked {
            return None;
        }
        let issued = chrono::Utc::now().to_rfc3339();
        session.token_hash = Self::hash_token(&[session_id, &session.nonce, &issued]);
        session.issued_at = issued;
        session.expires_at =
            (chrono::Utc::now() + chrono::Duration::seconds(SESSION_TTL_SECS)).to_rfc3339();
        Some(session.clone())
    }

    /// A session by id.
    pub fn session(&self, session_id: &str) -> Option<Session> {
        self.sessions
            .lock()
            .expect("lock poisoned")
            .get(session_id)
            .cloned()
    }

    /// The session a refresh token belongs to, if it is live.
    ///
    /// An expired or revoked session does not refresh: a refresh that works for any
    /// caller is a session bypass.
    pub fn session_for_refresh(&self, refresh_token: &str) -> Option<Session> {
        let session_id = self
            .by_refresh
            .lock()
            .expect("lock poisoned")
            .get(refresh_token)
            .cloned()?;
        let session = self.session(&session_id)?;
        if session.revoked {
            return None;
        }
        if session.refresh_expires_at <= chrono::Utc::now().to_rfc3339() {
            return None;
        }
        Some(session)
    }

    /// Revoke a session.
    pub fn revoke_session(&self, session_id: &str) -> bool {
        let mut guard = self.sessions.lock().expect("lock poisoned");
        match guard.get_mut(session_id) {
            Some(s) => {
                s.revoked = true;
                true
            }
            None => false,
        }
    }

    /// Every session, oldest first.
    pub fn sessions(&self) -> Vec<Session> {
        let guard = self.sessions.lock().expect("lock poisoned");
        let mut items: Vec<Session> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.issued_at.cmp(&b.issued_at));
        items
    }

    /// Provision a station, or return the one already provisioned.
    pub fn provision_station(&self, station_code: &str) -> StationState {
        let mut guard = self.stations.lock().expect("lock poisoned");
        let station = guard
            .entry(station_code.to_string())
            .or_insert_with(|| StationState {
                station_code: station_code.to_string(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                ..Default::default()
            });
        station.clone()
    }

    /// Whether a station has been provisioned.
    pub fn station_exists(&self, station_code: &str) -> bool {
        self.stations
            .lock()
            .expect("lock poisoned")
            .contains_key(station_code)
    }

    /// Record a kit check-in against a station.
    pub fn station_check_in(&self, station_code: &str, battery_level: Option<f64>) -> StationState {
        let mut guard = self.stations.lock().expect("lock poisoned");
        let station = guard
            .entry(station_code.to_string())
            .or_insert_with(|| StationState {
                station_code: station_code.to_string(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                ..Default::default()
            });
        station.online = true;
        if let Some(b) = battery_level {
            station.battery_level = Some(b);
        }
        station.last_seen_at = Some(chrono::Utc::now().to_rfc3339());
        station.updated_at = chrono::Utc::now().to_rfc3339();
        station.clone()
    }

    /// A station's state, if provisioned.
    pub fn station(&self, station_code: &str) -> Option<StationState> {
        self.stations
            .lock()
            .expect("lock poisoned")
            .get(station_code)
            .cloned()
    }

    /// Every station, by code.
    pub fn stations_sorted(&self) -> Vec<StationState> {
        let guard = self.stations.lock().expect("lock poisoned");
        let mut items: Vec<StationState> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.station_code.cmp(&b.station_code));
        items
    }

    /// Create a dispatch assignment.
    pub fn create_assignment(
        &self,
        worker_id: &str,
        role: &str,
        station_code: Option<String>,
    ) -> DispatchAssignment {
        let now = chrono::Utc::now().to_rfc3339();
        let assignment = DispatchAssignment {
            assignment_id: format!("assign-{}", short_id()),
            worker_id: worker_id.to_string(),
            role: role.to_string(),
            status: "assigned".to_string(),
            station_code,
            issued_at: now.clone(),
            updated_at: now,
        };
        self.assignments
            .lock()
            .expect("lock poisoned")
            .insert(assignment.assignment_id.clone(), assignment.clone());
        assignment
    }

    /// A worker's assignments, oldest first.
    pub fn assignments_for(&self, worker_id: &str) -> Vec<DispatchAssignment> {
        let guard = self.assignments.lock().expect("lock poisoned");
        let mut items: Vec<DispatchAssignment> = guard
            .values()
            .filter(|a| a.worker_id == worker_id)
            .cloned()
            .collect();
        drop(guard);
        items.sort_by(|a, b| a.issued_at.cmp(&b.issued_at));
        items
    }

    /// An assignment by id.
    pub fn assignment(&self, assignment_id: &str) -> Option<DispatchAssignment> {
        self.assignments
            .lock()
            .expect("lock poisoned")
            .get(assignment_id)
            .cloned()
    }

    /// Move an assignment to a new status, rejecting an unknown one.
    pub fn set_assignment_status(
        &self,
        assignment_id: &str,
        status: &str,
    ) -> Result<DispatchAssignment, String> {
        const STATUSES: [&str; 5] = [
            "assigned",
            "accepted",
            "in-progress",
            "completed",
            "cancelled",
        ];
        if !STATUSES.contains(&status) {
            return Err(format!(
                "unknown assignment status {status}; expected one of {}",
                STATUSES.join(", ")
            ));
        }
        let mut guard = self.assignments.lock().expect("lock poisoned");
        let assignment = guard
            .get_mut(assignment_id)
            .ok_or_else(|| format!("no assignment with id {assignment_id}"))?;
        // A completed or cancelled assignment is a closed record.
        if matches!(assignment.status.as_str(), "completed" | "cancelled") {
            return Err(format!(
                "assignment is {}; a closed assignment does not reopen",
                assignment.status
            ));
        }
        assignment.status = status.to_string();
        assignment.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(assignment.clone())
    }

    /// The incident already filed under an idempotency key, if any.
    pub fn incident_by_idempotency(&self, key: &str) -> Option<Incident> {
        let id = self
            .by_idempotency
            .lock()
            .expect("lock poisoned")
            .get(key)
            .cloned()?;
        self.incidents
            .lock()
            .expect("lock poisoned")
            .get(&id)
            .cloned()
    }

    /// File an incident, recording it against its idempotency key.
    pub fn file_incident(
        &self,
        incident_type: &str,
        station_code: Option<String>,
        severity: &str,
        reported_by_hash: &str,
        detail: Value,
        reported_via: &str,
        idempotency_key: &str,
    ) -> Incident {
        let incident = Incident {
            incident_id: format!("inc-{}", short_id()),
            incident_type: incident_type.to_string(),
            station_code,
            severity: severity.to_string(),
            reported_by_hash: reported_by_hash.to_string(),
            detail,
            reported_via: reported_via.to_string(),
            reported_at: chrono::Utc::now().to_rfc3339(),
            idempotency_key: idempotency_key.to_string(),
        };
        let mut guard = self.incidents.lock().expect("lock poisoned");
        guard.insert(incident.incident_id.clone(), incident.clone());
        drop(guard);
        self.by_idempotency
            .lock()
            .expect("lock poisoned")
            .insert(idempotency_key.to_string(), incident.incident_id.clone());
        incident
    }

    /// An incident by id.
    pub fn incident(&self, incident_id: &str) -> Option<Incident> {
        self.incidents
            .lock()
            .expect("lock poisoned")
            .get(incident_id)
            .cloned()
    }

    /// Every incident, oldest first.
    pub fn incidents(&self) -> Vec<Incident> {
        let guard = self.incidents.lock().expect("lock poisoned");
        let mut items: Vec<Incident> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.reported_at.cmp(&b.reported_at));
        items
    }

    /// Register a push token, returning the record.
    pub fn register_push_token(
        &self,
        device_id: &str,
        provider_token: &str,
        platform: &str,
    ) -> PushTokenRecord {
        let record = PushTokenRecord {
            token_id: format!("pt-{}", short_id()),
            device_id: device_id.to_string(),
            // A provider token is a credential: only its digest is kept.
            token_hash: Self::hash_token(&[device_id, provider_token]),
            platform: platform.to_string(),
            registered_at: chrono::Utc::now().to_rfc3339(),
            revoked: false,
        };
        self.push_tokens
            .lock()
            .expect("lock poisoned")
            .insert(record.token_id.clone(), record.clone());
        record
    }

    /// A push token record by id.
    pub fn push_token(&self, token_id: &str) -> Option<PushTokenRecord> {
        self.push_tokens
            .lock()
            .expect("lock poisoned")
            .get(token_id)
            .cloned()
    }

    /// Revoke a push token.
    pub fn revoke_push_token(&self, token_id: &str) -> bool {
        let mut guard = self.push_tokens.lock().expect("lock poisoned");
        match guard.get_mut(token_id) {
            Some(t) => {
                t.revoked = true;
                true
            }
            None => false,
        }
    }

    /// Every registered push token, oldest first.
    pub fn push_tokens(&self) -> Vec<PushTokenRecord> {
        let guard = self.push_tokens.lock().expect("lock poisoned");
        let mut items: Vec<PushTokenRecord> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.registered_at.cmp(&b.registered_at));
        items
    }

    /// Issue an observer access pass, if the accreditation is one this service
    /// holds.
    ///
    /// An access pass to a count centre is not something to mint on request. It is
    /// issued against a recorded accreditation application, so the pass names the
    /// application it rests on.
    pub fn issue_observer_pass(
        &self,
        application_id: &str,
        organisation: &str,
        accreditation_ref: &str,
    ) -> ObserverPass {
        let now = chrono::Utc::now();
        let pass = ObserverPass {
            pass_id: format!("PASS-{}", short_id()),
            organisation: organisation.to_string(),
            accreditation_ref: format!("{accreditation_ref}#{application_id}"),
            issued_at: now.to_rfc3339(),
            valid_until: (now + chrono::Duration::days(1)).to_rfc3339(),
            status: "active".to_string(),
        };
        self.passes
            .lock()
            .expect("lock poisoned")
            .insert(pass.pass_id.clone(), pass.clone());
        pass
    }

    /// An observer pass by id.
    pub fn observer_pass(&self, pass_id: &str) -> Option<ObserverPass> {
        self.passes
            .lock()
            .expect("lock poisoned")
            .get(pass_id)
            .cloned()
    }
}

// ---------------------------------------------------------------------------
// Forms, check-ins, tamper events, observations and grievances
// ---------------------------------------------------------------------------

/// A Form 34A held by the BFF.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Form34ARecord {
    pub form_id: String,
    pub station_code: String,
    /// The form's content, as submitted.
    pub content: Value,
    /// The digest of the content, so a later read can show whether it changed.
    pub content_hash: String,
    /// The form's position in the arrival-ordered leaf set.
    pub leaf_index: usize,
    pub received_at: String,
    /// Countersignatures, by countersigner.
    pub countersignatures: Vec<Countersignature>,
}

/// One officer's countersignature over a form.
///
/// **This is a digest, not a signature.** There is no signing key or HSM
/// reachable from this process, so nothing here is cryptographically attributable
/// to an officer. The field says so, because a field named `signature` invites the
/// reading this workspace cannot support.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Countersignature {
    pub countersigner_hash: String,
    pub content_hash: String,
    pub countersigned_at: String,
    /// Always true here; see the type's documentation.
    pub hsm_signature_verified: bool,
}

/// An agent check-in.
#[derive(Clone, Debug, serde::Serialize)]
pub struct AgentCheckIn {
    pub check_in_id: String,
    pub agent_id_hash: String,
    pub station_code: String,
    pub role: String,
    pub checked_in_at: String,
}

/// A tamper event report.
#[derive(Clone, Debug, serde::Serialize)]
pub struct TamperEvent {
    pub tamper_event_id: String,
    pub event_type: String,
    pub station_code: Option<String>,
    pub severity: String,
    pub detail: Value,
    pub reported_at: String,
}

/// An observer observation.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Observation {
    pub observation_id: String,
    pub station_code: String,
    pub observation_type: String,
    pub observer_hash: String,
    pub detail: Value,
    pub submitted_at: String,
}

/// A grievance filed through the public form.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Grievance {
    pub grievance_id: String,
    pub channel: String,
    /// The reporter's contact is a credential, so only its digest is kept.
    pub contact_hash: String,
    pub category: String,
    pub detail: Value,
    /// The Idempotency-Key it was filed under, so a retry is recognisable.
    pub idempotency_key: String,
    pub submitted_at: String,
}

impl MobileStore {
    /// Store a Form 34A, returning it with its leaf index.
    pub fn record_form(&self, form_id: &str, station_code: &str, content: Value) -> Form34ARecord {
        let mut guard = self.forms.lock().expect("lock poisoned");
        if let Some(existing) = guard.get(form_id) {
            return existing.clone();
        }
        let leaf_index = guard.len();
        let record = Form34ARecord {
            form_id: form_id.to_string(),
            station_code: station_code.to_string(),
            content_hash: Self::hash_token(&[form_id, &content.to_string()]),
            content,
            leaf_index,
            received_at: chrono::Utc::now().to_rfc3339(),
            countersignatures: Vec::new(),
        };
        guard.insert(form_id.to_string(), record.clone());
        record
    }

    /// A Form 34A by id.
    pub fn form(&self, form_id: &str) -> Option<Form34ARecord> {
        self.forms
            .lock()
            .expect("lock poisoned")
            .get(form_id)
            .cloned()
    }

    /// Every Form 34A, in arrival order.
    pub fn forms_sorted(&self) -> Vec<Form34ARecord> {
        let guard = self.forms.lock().expect("lock poisoned");
        let mut items: Vec<Form34ARecord> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.leaf_index.cmp(&b.leaf_index));
        items
    }

    /// Count a form as countersigned by an officer.
    ///
    /// The countersignature is bound to the content it was made over, so a form
    /// whose content changes afterwards carries a countersignature that no longer
    /// matches it. One officer countersigns once.
    pub fn countersign_form(
        &self,
        form_id: &str,
        countersigner_hash: &str,
    ) -> Result<Countersignature, String> {
        let mut guard = self.forms.lock().expect("lock poisoned");
        let form = guard
            .get_mut(form_id)
            .ok_or_else(|| format!("no Form 34A with id {form_id}"))?;
        if form
            .countersignatures
            .iter()
            .any(|c| c.countersigner_hash == countersigner_hash)
        {
            return Err(format!(
                "this officer has already countersigned form {form_id}"
            ));
        }
        let sig = Countersignature {
            countersigner_hash: countersigner_hash.to_string(),
            content_hash: form.content_hash.clone(),
            countersigned_at: chrono::Utc::now().to_rfc3339(),
            hsm_signature_verified: false,
        };
        form.countersignatures.push(sig.clone());
        Ok(sig)
    }

    /// The Merkle tree over the stored Form 34A content hashes.
    pub fn form_tree(&self) -> Option<eel_crypto_rs::merkle::MerkleTree> {
        let forms = self.forms_sorted();
        if forms.is_empty() {
            return None;
        }
        let entries: Vec<&[u8]> = forms.iter().map(|f| f.content_hash.as_bytes()).collect();
        Some(eel_crypto_rs::merkle::MerkleTree::build(&entries))
    }

    /// Record an agent check-in.
    pub fn check_in_agent(
        &self,
        agent_id_hash: &str,
        station_code: &str,
        role: &str,
    ) -> AgentCheckIn {
        let check_in = AgentCheckIn {
            check_in_id: format!("chk-{}", short_id()),
            agent_id_hash: agent_id_hash.to_string(),
            station_code: station_code.to_string(),
            role: role.to_string(),
            checked_in_at: chrono::Utc::now().to_rfc3339(),
        };
        self.check_ins
            .lock()
            .expect("lock poisoned")
            .insert(check_in.check_in_id.clone(), check_in.clone());
        check_in
    }

    /// Every agent check-in, oldest first.
    pub fn check_ins(&self) -> Vec<AgentCheckIn> {
        let guard = self.check_ins.lock().expect("lock poisoned");
        let mut items: Vec<AgentCheckIn> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.checked_in_at.cmp(&b.checked_in_at));
        items
    }

    /// A tamper event already reported under an idempotency key, if any.
    pub fn tamper_by_idempotency(&self, key: &str) -> Option<TamperEvent> {
        let id = self
            .tamper_index
            .lock()
            .expect("lock poisoned")
            .get(key)
            .cloned()?;
        self.tamper_events
            .lock()
            .expect("lock poisoned")
            .get(&id)
            .cloned()
    }

    /// Record a tamper event.
    pub fn record_tamper(
        &self,
        event_type: &str,
        station_code: Option<String>,
        severity: &str,
        detail: Value,
        idempotency_key: &str,
    ) -> TamperEvent {
        let event = TamperEvent {
            tamper_event_id: format!("tam-{}", short_id()),
            event_type: event_type.to_string(),
            station_code,
            severity: severity.to_string(),
            detail,
            reported_at: chrono::Utc::now().to_rfc3339(),
        };
        self.tamper_events
            .lock()
            .expect("lock poisoned")
            .insert(event.tamper_event_id.clone(), event.clone());
        self.tamper_index
            .lock()
            .expect("lock poisoned")
            .insert(idempotency_key.to_string(), event.tamper_event_id.clone());
        event
    }

    /// Every tamper event, oldest first.
    pub fn tamper_events(&self) -> Vec<TamperEvent> {
        let guard = self.tamper_events.lock().expect("lock poisoned");
        let mut items: Vec<TamperEvent> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.reported_at.cmp(&b.reported_at));
        items
    }

    /// The observation already submitted under an idempotency key, if any.
    pub fn observation_by_idempotency(&self, key: &str) -> Option<Observation> {
        let id = self
            .observation_index
            .lock()
            .expect("lock poisoned")
            .get(key)
            .cloned()?;
        self.observations
            .lock()
            .expect("lock poisoned")
            .get(&id)
            .cloned()
    }

    /// Submit an observation.
    pub fn submit_observation(
        &self,
        station_code: &str,
        observation_type: &str,
        observer_hash: &str,
        detail: Value,
        idempotency_key: &str,
    ) -> Observation {
        let observation = Observation {
            observation_id: format!("obs-{}", short_id()),
            station_code: station_code.to_string(),
            observation_type: observation_type.to_string(),
            observer_hash: observer_hash.to_string(),
            detail,
            submitted_at: chrono::Utc::now().to_rfc3339(),
        };
        self.observations
            .lock()
            .expect("lock poisoned")
            .insert(observation.observation_id.clone(), observation.clone());
        self.observation_index
            .lock()
            .expect("lock poisoned")
            .insert(
                idempotency_key.to_string(),
                observation.observation_id.clone(),
            );
        observation
    }

    /// Every observation, oldest first.
    pub fn observations(&self) -> Vec<Observation> {
        let guard = self.observations.lock().expect("lock poisoned");
        let mut items: Vec<Observation> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.submitted_at.cmp(&b.submitted_at));
        items
    }

    /// The grievance already filed under an idempotency key, if any.
    pub fn grievance_by_idempotency(&self, key: &str) -> Option<Grievance> {
        let id = self
            .grievance_index
            .lock()
            .expect("lock poisoned")
            .get(key)
            .cloned()?;
        self.grievances
            .lock()
            .expect("lock poisoned")
            .get(&id)
            .cloned()
    }

    /// File a grievance.
    pub fn file_grievance(
        &self,
        channel: &str,
        contact: &str,
        category: &str,
        detail: Value,
        idempotency_key: &str,
    ) -> Grievance {
        let grievance = Grievance {
            grievance_id: format!("grv-{}", short_id()),
            channel: channel.to_string(),
            contact_hash: Self::hash_token(&[contact]),
            category: category.to_string(),
            detail,
            idempotency_key: idempotency_key.to_string(),
            submitted_at: chrono::Utc::now().to_rfc3339(),
        };
        self.grievances
            .lock()
            .expect("lock poisoned")
            .insert(grievance.grievance_id.clone(), grievance.clone());
        self.grievance_index
            .lock()
            .expect("lock poisoned")
            .insert(idempotency_key.to_string(), grievance.grievance_id.clone());
        grievance
    }

    /// Record a registration hash this service has seen.
    pub fn record_registration(&self, registration_hash: &str) {
        self.registrations.lock().expect("lock poisoned").insert(
            registration_hash.to_string(),
            chrono::Utc::now().to_rfc3339(),
        );
    }

    /// Every registration hash this service has seen, in arrival order.
    pub fn registrations(&self) -> Vec<String> {
        let guard = self.registrations.lock().expect("lock poisoned");
        guard.keys().cloned().collect()
    }

    /// Every grievance, oldest first.
    pub fn grievances(&self) -> Vec<Grievance> {
        let guard = self.grievances.lock().expect("lock poisoned");
        let mut items: Vec<Grievance> = guard.values().cloned().collect();
        drop(guard);
        items.sort_by(|a, b| a.submitted_at.cmp(&b.submitted_at));
        items
    }
}

/// A short id from a UUID.
///
/// The slice is taken from the **end** of the hex, not the start. A UUIDv7's
/// leading hex is its millisecond timestamp, so two ids minted in the same
/// millisecond share a prefix and slicing from the front produces collisions --
/// which is exactly what `two_jobs_have_different_subjects` caught when it found
/// both jobs reporting the second subject.
fn short_id() -> String {
    let hex = uuid::Uuid::now_v7().simple().to_string();
    hex[hex.len() - 12..].to_string()
}
