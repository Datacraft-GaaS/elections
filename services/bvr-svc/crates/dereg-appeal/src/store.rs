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

//! In-memory projection for the deregistration-appeal surface.
//!
//! `AppState` was a Postgres pool and nothing else, so the four handlers built
//! their answers instead:
//!
//! - `decide_dereg_appeal` returned `201 Created` with
//!   `reinstated: req.verdict == "reinstate"` for **any** appeal id. That is a
//!   decision reinstating a voter's registration against a case that need not
//!   exist, and it is the decision a deregistered voter most needs to be right.
//! - `get_dereg_appeal` returned `404` for every id, so a filed case could not be
//!   read back.
//! - `list_dereg_appeals` returned an empty page, so filed cases were invisible.
//! - `handle_dereg_ussd_session` answered `END Service unavailable` to every
//!   caller, an honest refusal shaped as a session that started and immediately
//!   hung up.
//!
//! An appeal here is a person's claim that their deregistration was wrong. The
//! appeal and the decision on it are the record; a decision that can be recorded
//! against nothing is the failure this store removes.
//!
//! The shapes come from `packages/openapi-contracts/dereg/appeal.yaml`:
//!
//! - `DeregAppeal` = `DeregAppealCreate` (voter_id_hash, deregistration_ref,
//!   grounds, supporting_doc_hash) plus appeal_id (a **UUID**), status, submitted_at
//!   and updated_at.
//! - `status` is one of submitted, under_review, upheld, dismissed.
//! - `verdict` is one of upheld, dismissed -- and the direction matters: **upholding
//!   the appeal** means the deregistration was wrong, so the voter is reinstated.
//!   Dismissing the appeal means the deregistration stands. The previous handler
//!   had this backwards, which is the direction that puts a person back on the
//!   register on a dismissal.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

/// Where a deregistration appeal is in its lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppealStatus {
    /// Filed, not yet picked up.
    Submitted,
    /// Picked up and awaiting a decision.
    UnderReview,
    /// The appeal succeeded: the deregistration was wrong and the voter is restored.
    Upheld,
    /// The appeal failed: the deregistration stands.
    Dismissed,
}

impl AppealStatus {
    /// The wire value.
    pub fn wire(&self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::UnderReview => "under_review",
            Self::Upheld => "upheld",
            Self::Dismissed => "dismissed",
        }
    }

    /// Whether the appeal can still be decided.
    pub fn is_decidable(&self) -> bool {
        matches!(self, Self::Submitted | Self::UnderReview)
    }
}

/// A decision on an appeal.
#[derive(Clone, Debug, Serialize)]
pub struct AppealDecision {
    /// A UUID, per the contract.
    pub decision_id: String,
    /// `upheld` or `dismissed`.
    pub verdict: String,
    pub grounds: String,
    /// SHA-256(salt||officer_id). The officer is never identified by name here.
    pub decided_by_hash: String,
    /// True when the appeal succeeded and the voter returns to the register.
    pub reinstated: bool,
    pub decided_at: String,
    /// Why the register itself was not changed here.
    pub blocking_reason: Option<String>,
}

/// One filed deregistration appeal. Serialises to `DeregAppeal`.
#[derive(Clone, Debug, Serialize)]
pub struct DeregAppeal {
    /// A UUID, per the contract's `format: uuid`.
    pub appeal_id: String,
    /// SHA-256(salt||voter_id). Class A; the voter is never identified by name.
    pub voter_id_hash: String,
    /// Reference number on the deregistration notice.
    pub deregistration_ref: String,
    pub grounds: String,
    /// SHA-256 of the attached supporting document.
    pub supporting_doc_hash: String,
    /// One of submitted, under_review, upheld, dismissed.
    pub status: AppealStatus,
    pub submitted_at: String,
    pub updated_at: String,
    /// The decision, once decided.
    pub decision: Option<AppealDecision>,
}

/// One USSD exchange against the appeal line.
#[derive(Clone, Debug, Serialize)]
pub struct UssdStep {
    pub input: String,
    pub response_text: String,
    pub action: String,
    pub at: String,
}

/// Shared state for the deregistration-appeal surface.
#[derive(Default)]
pub struct DeregStore {
    appeals: Mutex<HashMap<String, DeregAppeal>>,
    order: Mutex<Vec<String>>,
    ussd: Mutex<HashMap<String, Vec<UssdStep>>>,
}

/// The handle threaded through the router.
pub type SharedDeregStore = Arc<DeregStore>;

/// The hash of a national ID, so an appeal record holds no class A PII.
pub fn hash_national_id_public(national_id: &str) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(b"eel.bvr.dereg.national-id.v1");
    h.update([0x1e]);
    h.update(national_id.trim().as_bytes());
    format!("sha256:{}", hex::encode(h.finalize()))
}

impl DeregStore {
    /// Construct an empty store.
    pub fn new() -> SharedDeregStore {
        Arc::new(Self::default())
    }

    /// Take an appeal as filed by the deregistration workflow.
    ///
    /// There is no `POST /appeals` in the contract, so nothing in this workspace
    /// creates an appeal: the deregistration workflow that raises one lives
    /// upstream. This is the ingestion point that workflow writes through, and it
    /// is what the read and decision paths are exercised against.
    pub fn file_appeal(
        &self,
        voter_id_hash: &str,
        deregistration_ref: &str,
        grounds: &str,
        supporting_doc_hash: &str,
    ) -> DeregAppeal {
        let now = chrono::Utc::now().to_rfc3339();
        let appeal = DeregAppeal {
            appeal_id: uuid::Uuid::now_v7().to_string(),
            voter_id_hash: voter_id_hash.to_string(),
            deregistration_ref: deregistration_ref.to_string(),
            grounds: grounds.to_string(),
            supporting_doc_hash: supporting_doc_hash.to_string(),
            status: AppealStatus::Submitted,
            submitted_at: now.clone(),
            updated_at: now,
            decision: None,
        };
        self.order
            .lock()
            .expect("lock poisoned")
            .push(appeal.appeal_id.clone());
        self.appeals
            .lock()
            .expect("lock poisoned")
            .insert(appeal.appeal_id.clone(), appeal.clone());
        appeal
    }

    /// An appeal by its id.
    pub fn appeal(&self, appeal_id: &str) -> Option<DeregAppeal> {
        self.appeals
            .lock()
            .expect("lock poisoned")
            .get(appeal_id)
            .cloned()
    }

    /// Move an appeal into `under_review`.
    pub fn mark_under_review(&self, appeal_id: &str) -> Result<DeregAppeal, String> {
        let mut guard = self.appeals.lock().expect("lock poisoned");
        let appeal = guard
            .get_mut(appeal_id)
            .ok_or_else(|| format!("no appeal with id {appeal_id}"))?;
        if appeal.status != AppealStatus::Submitted {
            return Err(format!(
                "appeal is {}; only a submitted appeal moves to review",
                appeal.status.wire()
            ));
        }
        appeal.status = AppealStatus::UnderReview;
        appeal.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(appeal.clone())
    }

    /// Every appeal, oldest first.
    pub fn appeals(&self) -> Vec<DeregAppeal> {
        let order = self.order.lock().expect("lock poisoned");
        let guard = self.appeals.lock().expect("lock poisoned");
        order.iter().filter_map(|id| guard.get(id).cloned()).collect()
    }

    /// Appeals with a given status, oldest first.
    pub fn appeals_with_status(&self, status: AppealStatus) -> Vec<DeregAppeal> {
        self.appeals()
            .into_iter()
            .filter(|a| a.status == status)
            .collect()
    }

    /// Appeals sharing a deregistration reference.
    pub fn appeals_for_ref(&self, deregistration_ref: &str) -> Vec<DeregAppeal> {
        self.appeals()
            .into_iter()
            .filter(|a| a.deregistration_ref.eq_ignore_ascii_case(deregistration_ref))
            .collect()
    }

    /// Decide an appeal.
    ///
    /// The direction is the part that was previously inverted. The contract's
    /// verdict enum is `[upheld, dismissed]`:
    ///
    /// - **upheld**: the appeal is upheld, so the deregistration was wrong and the
    ///   voter is reinstated.
    /// - **dismissed**: the appeal is dismissed, so the deregistration stands and
    ///   the voter is not reinstated.
    pub fn decide(
        &self,
        appeal_id: &str,
        verdict: &str,
        grounds: &str,
        decided_by_hash: &str,
    ) -> Result<AppealDecision, String> {
        const VERDICTS: [&str; 2] = ["upheld", "dismissed"];
        let verdict_norm = verdict.trim().to_ascii_lowercase();
        if !VERDICTS.contains(&verdict_norm.as_str()) {
            return Err(format!(
                "verdict must be one of {}, got {verdict}",
                VERDICTS.join(", ")
            ));
        }
        let mut guard = self.appeals.lock().expect("lock poisoned");
        let appeal = guard
            .get_mut(appeal_id)
            .ok_or_else(|| format!("no appeal with id {appeal_id}"))?;
        if !appeal.status.is_decidable() {
            return Err(format!(
                "appeal is {}; an appeal that has already been decided cannot be decided again",
                appeal.status.wire()
            ));
        }
        let decision = AppealDecision {
            decision_id: uuid::Uuid::now_v7().to_string(),
            reinstated: verdict_norm == "upheld",
            verdict: verdict_norm.clone(),
            grounds: grounds.to_string(),
            decided_by_hash: decided_by_hash.to_string(),
            decided_at: chrono::Utc::now().to_rfc3339(),
            // The decision is recorded here. Writing it back to the register is the
            // register service's job, and that service is not in this workspace --
            // so this decision is a finding, not an applied change.
            blocking_reason: Some(
                "the decision is recorded here; writing it back to the register is the \
                 register service's job and that service is not in this workspace"
                    .to_string(),
            ),
        };
        appeal.status = if decision.reinstated {
            AppealStatus::Upheld
        } else {
            AppealStatus::Dismissed
        };
        appeal.updated_at = decision.decided_at.clone();
        appeal.decision = Some(decision.clone());
        Ok(decision)
    }

    /// Answer one USSD step on the deregistration line.
    ///
    /// The menu is keyed on the **reference on the deregistration notice**, which
    /// the citizen already holds. It is deliberately not keyed on the caller's
    /// phone number: the request carries `msisdn_hash`, and appeals are stored
    /// against `voter_id_hash`, and nothing in this workspace resolves one to the
    /// other. Offering a "look up by my number" branch would be a promise this
    /// process cannot keep, so the branch is not offered.
    ///
    /// State is derived from the accumulated `*`-delimited input rather than a
    /// server-side session, so the endpoint is idempotent and no lock is held
    /// across the store lookup.
    pub fn ussd_step(&self, session_id: &str, text: &str) -> (String, String) {
        const MAX: usize = 182;
        let segments: Vec<&str> = text.split('*').map(str::trim).collect();
        let first = segments.first().copied().unwrap_or_default();
        let (body, action) = if first.is_empty() {
            (
                "Karibu. 1. Angalia appeal yako\n2. Onyesha maelekezo\n0. Mwisho".to_string(),
                "CON",
            )
        } else if first == "0" {
            ("Karibu tena.".to_string(), "END")
        } else if first == "1" {
            match segments.get(1).copied().unwrap_or_default() {
                "" => ("Andika namba ya appeal, au 0 kuacha.".to_string(), "CON"),
                reference => {
                    let matches = self.lookup_reference(reference);
                    let reply = match matches.len() {
                        1 => {
                            let a = &matches[0];
                            let status = a.status.wire();
                            let outcome = match &a.decision {
                                Some(d) if d.reinstated => {
                                    format!("Appeal ulioadhimishwa. Umeirudishwa kwenye orodha ({status}).")
                                }
                                Some(_) => format!(
                                    "Appeal umeangukwa. Uendelea kwenye orodha ya kuviandikisha ({status})."
                                ),
                                None => format!("Appeal yako iko ({status})."),
                            };
                            format!("{outcome}")
                        }
                        0 => format!("Hakuna appeal yenye kumbukumbu {reference}."),
                        n => format!("Kuna appeals {n} zilizo shikiliwa kumbukumbu hiyo. Fungua iko."),
                    };
                    (reply, "END")
                }
            }
        } else if first == "2" {
            (
                "Unaweza kuwasilisha appeal kwa kumbukumbu ya uthibitisho. Hii inahitaji \
                 kumbukumbu ya kituo na hati."
                    .to_string(),
                "END",
            )
        } else {
            (format!("{first} si mojawepo. Jibu 1, 2 au 0."), "CON")
        };
        let text = if body.chars().count() > MAX {
            body.chars().take(MAX).collect()
        } else {
            body
        };
        self.ussd
            .lock()
            .expect("lock poisoned")
            .entry(session_id.to_string())
            .or_default()
            .push(UssdStep {
                input: text.to_string(),
                response_text: text.clone(),
                action: action.to_string(),
                at: chrono::Utc::now().to_rfc3339(),
            });
        (text, action.to_string())
    }

    /// Find an appeal by its id or by the reference on the deregistration notice.
    fn lookup_reference(&self, reference: &str) -> Vec<DeregAppeal> {
        if let Some(a) = self.appeal(reference) {
            return vec![a];
        }
        self.appeals_for_ref(reference)
    }

    /// A USSD session's exchanges, oldest first.
    pub fn ussd_session(&self, session_id: &str) -> Option<Vec<UssdStep>> {
        self.ussd.lock().expect("lock poisoned").get(session_id).cloned()
    }
}
