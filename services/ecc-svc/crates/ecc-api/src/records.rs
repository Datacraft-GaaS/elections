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

//! Records for the ECC station, grievance, escalation and IAB endpoints.
//!
//! Every handler in this group took no state at all. `list_stations` returned an
//! empty array, `get_station` answered 200 with `name: "stub"` for any station
//! code, `create_complaint` minted a reference code and dropped it, `list_complaints`
//! always returned `[]`, `get_complaint` answered for any reference code, and
//! `get_audit_log` returned `{"entries": []}`. A complaint filed through the
//! grievance portal could never be retrieved, and an escalation could never be
//! resolved.
//!
//! These live behind an `Extension` rather than in the router state, because
//! `.with_state(store)` is already the speech handlers' `State<Store>` and retyping
//! that would touch code unrelated to this fix. A fresh `Records` is constructed
//! per `build_router` call, so each router — including every test's — is isolated.

use eel_memstore_rs::Collection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{ComplaintUpdate, Escalation, GrievanceComplaint, Station};

/// An Internal Audit Bureau flag, with the detail the public shape omits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IabFlagRecord {
    pub flag_id: Uuid,
    pub module_code: String,
    pub reason: String,
    pub raised_by: String,
    pub status: String,
    pub created_at: String,
}

/// An audit report, with the detail the public shape omits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReportRecord {
    pub report_id: Uuid,
    pub module_code: String,
    pub title: String,
    pub status: String,
    pub submitted_at: String,
    pub findings: u32,
}

/// A single audit-log entry, derived from what actually happened.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub entry_id: String,
    pub kind: String,
    pub subject: String,
    pub detail: String,
    pub at: String,
}

/// All records these endpoints read and write.
///
/// `Clone` shares rather than copies, which is what axum's `Extension` requires;
/// see `Collection::shares_state_with`.
#[derive(Debug, Default, Clone)]
pub struct Records {
    pub stations: Collection<Station>,
    pub complaints: Collection<GrievanceComplaint>,
    pub complaint_updates: Collection<ComplaintUpdate>,
    pub escalations: Collection<Escalation>,
    pub iab_flags: Collection<IabFlagRecord>,
    pub audit_reports: Collection<AuditReportRecord>,
    /// Audit-log entries, appended as things happen rather than derived on read.
    pub audit_log: Collection<AuditEntry>,
}

impl Records {
    /// Record an audit-log entry.
    pub fn audit(&self, kind: &str, subject: &str, detail: &str, at: String) {
        let entry = AuditEntry {
            entry_id: Uuid::now_v7().to_string(),
            kind: kind.to_string(),
            subject: subject.to_string(),
            detail: detail.to_string(),
            at,
        };
        self.audit_log.insert(entry.entry_id.clone(), entry);
    }

    /// Escalations not yet resolved.
    pub fn open_escalations(&self) -> Vec<Escalation> {
        self.escalations
            .list_where(|e| e.status != "resolved" && e.status != "closed")
    }
}
