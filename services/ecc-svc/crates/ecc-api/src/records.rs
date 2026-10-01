//! Records for the ECC grievance, escalation and IAB endpoints.
//!
//! The handlers in this crate took no state at all: `create_complaint` minted a
//! reference code and dropped it, `list_complaints` always returned an empty
//! array, `get_complaint` answered for any reference code, and `get_audit_log`
//! returned `{"entries": []}`. A complaint filed through the grievance portal was
//! never retrievable, and the audit log recorded nothing.
//!
//! These live behind an `Extension` rather than in the router state, because
//! `.with_state(store)` is already the speech handlers' `State<Store>` and
//! retyping that would touch code unrelated to this fix.

use eel_memstore_rs::Collection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A grievance complaint as stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Complaint {
    pub complaint_id: Uuid,
    pub reference_code: String,
    pub complainant_name: String,
    pub complainant_contact: String,
    pub category: String,
    pub description: String,
    pub station_code: Option<String>,
    pub status: String,
    pub created_at: String,
}

/// A note added to a complaint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplaintUpdate {
    pub update_id: Uuid,
    pub complaint_id: Uuid,
    pub note: String,
    pub author: String,
    pub created_at: String,
}

/// An election-security escalation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub escalation_id: Uuid,
    pub station_code: String,
    pub category: String,
    pub severity: String,
    pub description: String,
    pub raised_by: String,
    pub status: String,
    pub resolution_notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// An Internal Audit Bureau flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IabFlag {
    pub flag_id: Uuid,
    pub subject: String,
    pub reason: String,
    pub raised_by: String,
    pub status: String,
    pub created_at: String,
}

/// An audit report produced by the IAB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_id: Uuid,
    pub module_code: String,
    pub summary: String,
    pub findings: u32,
    pub generated_by: String,
    pub generated_at: String,
}

/// All records the grievance/escalation/IAB endpoints read and write.
///
/// `Clone` on a shared handle rather than a deep copy: the collections are
/// internally synchronised, so cloning must share state rather than fork it.
#[derive(Debug, Default, Clone)]
pub struct Records {
    pub complaints: Collection<Complaint>,
    pub complaint_updates: Collection<ComplaintUpdate>,
    pub escalations: Collection<Escalation>,
    pub iab_flags: Collection<IabFlag>,
    pub audit_reports: Collection<AuditReport>,
}
