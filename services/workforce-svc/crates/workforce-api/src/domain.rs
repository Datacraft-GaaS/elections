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

//! Domain types and rules for workforce-svc (design: docs/workforce-lifecycle.md).
//!
//! One crate covers five bounded contexts:
//! registry & contacts (§4), recruitment pipeline (§1), postings & equipment (§5),
//! tasking (§6) and performance (§7). All content-bearing identifiers are hashes;
//! no raw PII beyond names required for appointment records (DPA 2019).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const HASH_LEN: usize = 64;

/// True when `s` is exactly 64 hexadecimal characters (SHA-256 hex encoding).
pub fn is_hex64(s: &str) -> bool {
    s.len() == HASH_LEN && hex::decode(s).is_ok()
}

// ---------------------------------------------------------------------------
// Registry & contacts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Phone,
    WhatsApp,
    Email,
    Sms,
}

impl Channel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Phone => "phone",
            Channel::WhatsApp => "whatsapp",
            Channel::Email => "email",
            Channel::Sms => "sms",
        }
    }

    pub fn parse(s: &str) -> Option<Channel> {
        match s {
            "phone" => Some(Channel::Phone),
            "whatsapp" => Some(Channel::WhatsApp),
            "email" => Some(Channel::Email),
            "sms" => Some(Channel::Sms),
            _ => None,
        }
    }
}

impl std::str::FromStr for Channel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Channel::parse(s).ok_or(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Worker {
    pub worker_id: Uuid,
    pub legal_name: String,
    pub national_id_hash: String,
    pub home_county: String,
    pub role_eligibility: Vec<String>,
    pub payroll_contract_ref: Option<String>,
    pub next_of_kin_name: Option<String>,
    pub next_of_kin_phone_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NextOfKin {
    pub name: String,
    pub phone_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkerRequest {
    pub legal_name: String,
    pub national_id_hash: String,
    pub home_county: String,
    #[serde(default)]
    pub role_eligibility: Vec<String>,
    #[serde(default)]
    pub payroll_contract_ref: Option<String>,
    #[serde(default)]
    pub next_of_kin: Option<NextOfKin>,
}

/// A verifiable, QR-friendly appointment credential for a worker: who they
/// are, where they are posted, and a deterministic hash the field app renders
/// as a scannable badge.
#[derive(Debug, Clone, Serialize)]
pub struct WorkerBadge {
    pub worker_id: Uuid,
    pub name: String,
    pub role: String,
    pub station_code: Option<String>,
    pub contract_ref: Option<String>,
    pub issued_at: DateTime<Utc>,
    pub badge_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContactPoint {
    pub contact_id: Uuid,
    pub worker_id: Uuid,
    pub channel: Channel,
    pub value_hash_hex: String,
    pub verified: bool,
    /// DPA 2019 consent flag.
    pub consented: bool,
    pub preferred: bool,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactAction {
    Added,
    Updated,
    Removed,
    Verified,
    ConsentGranted,
}

impl ContactAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContactAction::Added => "added",
            ContactAction::Updated => "updated",
            ContactAction::Removed => "removed",
            ContactAction::Verified => "verified",
            ContactAction::ConsentGranted => "consent_granted",
        }
    }

    pub fn parse(s: &str) -> Option<ContactAction> {
        match s {
            "added" => Some(ContactAction::Added),
            "updated" => Some(ContactAction::Updated),
            "removed" => Some(ContactAction::Removed),
            "verified" => Some(ContactAction::Verified),
            "consent_granted" => Some(ContactAction::ConsentGranted),
            _ => None,
        }
    }
}

/// Append-only contact change log entry.
#[derive(Debug, Clone, Serialize)]
pub struct ContactLogEntry {
    pub worker_id: Uuid,
    pub channel: Channel,
    pub at: DateTime<Utc>,
    pub actor: String,
    pub action: ContactAction,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpsertContactRequest {
    pub value_hash_hex: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub consented: bool,
    #[serde(default)]
    pub preferred: bool,
    /// Set together with a non-empty `reason` to remove the channel's contact point.
    #[serde(default)]
    pub remove: bool,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default = "default_actor")]
    pub actor: String,
}

fn default_actor() -> String {
    "system".to_string()
}

// ---------------------------------------------------------------------------
// Recruitment state machine
// ---------------------------------------------------------------------------

/// Recruitment pipeline stage. Terminal states carry their payload
/// (`rejected{reason_code}`, `waitlist{rank}`); serialised with an internal
/// `state` tag so funnel analytics can group on the state string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Stage {
    Applied,
    Screened,
    DocumentsComplete,
    Evaluated,
    Selected,
    Appointed,
    Rejected { reason_code: String },
    Waitlist { rank: i64 },
}

impl Stage {
    pub fn state(&self) -> &'static str {
        match self {
            Stage::Applied => "applied",
            Stage::Screened => "screened",
            Stage::DocumentsComplete => "documents_complete",
            Stage::Evaluated => "evaluated",
            Stage::Selected => "selected",
            Stage::Appointed => "appointed",
            Stage::Rejected { .. } => "rejected",
            Stage::Waitlist { .. } => "waitlist",
        }
    }

    pub(crate) fn from_row(state: &str, reject_reason: Option<String>, rank: Option<i64>) -> Stage {
        match state {
            "rejected" => Stage::Rejected {
                reason_code: reject_reason.unwrap_or_default(),
            },
            "waitlist" => Stage::Waitlist {
                rank: rank.unwrap_or(0),
            },
            "applied" => Stage::Applied,
            "screened" => Stage::Screened,
            "documents_complete" => Stage::DocumentsComplete,
            "evaluated" => Stage::Evaluated,
            "selected" => Stage::Selected,
            "appointed" => Stage::Appointed,
            other => Stage::Rejected {
                reason_code: format!("unknown_stage:{other}"),
            },
        }
    }
}

/// Allowed next states per design §1; every non-terminal stage may reject.
pub fn allowed_transitions(from: &str) -> &'static [&'static str] {
    match from {
        "applied" => &["screened", "rejected"],
        "screened" => &["documents_complete", "rejected"],
        "documents_complete" => &["evaluated", "rejected"],
        "evaluated" => &["selected", "rejected"],
        "selected" => &["appointed", "waitlist", "rejected"],
        _ => &[],
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageChange {
    pub from: String,
    pub to: String,
    pub actor: String,
    pub at: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecruitmentCase {
    pub case_id: Uuid,
    pub application_ref: String,
    pub applicant_name: String,
    pub national_id_hash: String,
    pub role_code: String,
    pub county: String,
    pub stage: Stage,
    pub composite_score: Option<f32>,
    pub stage_history: Vec<StageChange>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCaseRequest {
    pub application_ref: String,
    pub applicant_name: String,
    pub national_id_hash: String,
    pub role_code: String,
    pub county: String,
    #[serde(default)]
    pub composite_score: Option<f32>,
}

/// Gate payloads passed by external gate owners (support-admin completeness
/// probe, training-svc exam result) — design §1/§2/§3.
#[derive(Debug, Deserialize)]
pub struct DocumentsProbe {
    pub complete: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExamResult {
    pub passed: bool,
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TransitionRequest {
    pub to: String,
    #[serde(default = "default_actor")]
    pub actor: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub documents_probe: Option<DocumentsProbe>,
    #[serde(default)]
    pub exam_result: Option<ExamResult>,
    #[serde(default)]
    pub contract_ref: Option<String>,
}

/// Recruitment evaluation gates. Enabled by default; `WORKFORCE_GATES` (a
/// comma-separated subset, e.g. `documents,contract`) disables the rest so a
/// deployment can configure which gates the pipeline enforces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GateConfig {
    pub documents: bool,
    pub exam: bool,
    pub contract: bool,
}

impl GateConfig {
    pub fn all() -> Self {
        Self {
            documents: true,
            exam: true,
            contract: true,
        }
    }

    pub fn parse(spec: &str) -> Self {
        let set: std::collections::HashSet<&str> = spec
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            documents: set.contains("documents"),
            exam: set.contains("exam"),
            contract: set.contains("contract"),
        }
    }

    pub fn from_env() -> Self {
        std::env::var("WORKFORCE_GATES")
            .map(|v| Self::parse(&v))
            .unwrap_or_else(|_| Self::all())
    }
}

// ---------------------------------------------------------------------------
// Postings & equipment claims
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PostingOrder {
    pub posting_id: Uuid,
    pub worker_id: Uuid,
    pub station_code: String,
    pub role_code: String,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub posted_by: String,
    pub superseded_by: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostingRequest {
    pub worker_id: Uuid,
    pub station_code: String,
    pub role_code: String,
    pub effective_from: DateTime<Utc>,
    #[serde(default)]
    pub effective_to: Option<DateTime<Utc>>,
    pub posted_by: String,
}

#[derive(Debug, Deserialize)]
pub struct SupersedeRequest {
    pub new_station_code: String,
    pub effective_from: DateTime<Utc>,
    #[serde(default)]
    pub effective_to: Option<DateTime<Utc>>,
    pub posted_by: String,
    #[serde(default)]
    pub role_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquipmentClaim {
    pub claim_id: Uuid,
    pub posting_id: Uuid,
    pub kit_serials: Vec<String>,
    pub acknowledged: bool,
    pub returned: bool,
    pub loss_flags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateClaimRequest {
    pub kit_serials: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FlagLossRequest {
    pub serials: Vec<String>,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Tasking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    PrePoll,
    PollingDay,
    PostPoll,
}

impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::PrePoll => "pre_poll",
            Phase::PollingDay => "polling_day",
            Phase::PostPoll => "post_poll",
        }
    }

    pub fn parse(s: &str) -> Option<Phase> {
        match s {
            "pre_poll" => Some(Phase::PrePoll),
            "polling_day" => Some(Phase::PollingDay),
            "post_poll" => Some(Phase::PostPoll),
            _ => None,
        }
    }
}

impl std::str::FromStr for Phase {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Phase::parse(s).ok_or(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskTemplate {
    pub template_id: Uuid,
    pub role_code: String,
    pub phase: Phase,
    pub title: String,
    pub critical: bool,
    /// Optional link to just-in-time training content (design §6): the mobile
    /// app surfaces this id so a worker can open the exact training material
    /// for the task they are being paid to perform.
    pub training_content_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Acked,
    Done,
    Blocked,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Acked => "acked",
            TaskStatus::Done => "done",
            TaskStatus::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskAssignment {
    pub assignment_id: Uuid,
    pub posting_id: Uuid,
    pub template_id: Uuid,
    pub due_from: DateTime<Utc>,
    pub due_to: DateTime<Utc>,
    pub status: TaskStatus,
    pub evidence_hash: Option<String>,
    pub block_reason: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub role_code: String,
    pub phase: Phase,
    pub title: String,
    #[serde(default)]
    pub critical: bool,
    #[serde(default)]
    pub training_content_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DueWindow {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTaskRequest {
    pub posting_id: Uuid,
    pub template_id: Uuid,
    pub due_window: DueWindow,
}

#[derive(Debug, Default, Deserialize)]
pub struct DoneTaskRequest {
    #[serde(default)]
    pub evidence_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlockTaskRequest {
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Performance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerfCategory {
    Attendance,
    Punctuality,
    Incident,
    Evaluation,
}

impl PerfCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerfCategory::Attendance => "attendance",
            PerfCategory::Punctuality => "punctuality",
            PerfCategory::Incident => "incident",
            PerfCategory::Evaluation => "evaluation",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceRecord {
    pub record_id: Uuid,
    pub worker_id: Uuid,
    pub category: PerfCategory,
    pub score: Option<f32>,
    pub detail: String,
    pub recorded_by: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddRecordRequest {
    pub worker_id: Uuid,
    pub category: PerfCategory,
    #[serde(default)]
    pub score: Option<f32>,
    pub detail: String,
    pub recorded_by: String,
}

#[derive(Debug, Serialize)]
pub struct PerfSummary {
    pub avg_score: Option<f32>,
    pub incident_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PerformanceResponse {
    pub records: Vec<PerformanceRecord>,
    pub summary: PerfSummary,
}

// ---------------------------------------------------------------------------
// Worker profile update (design §4: "update their information")
// ---------------------------------------------------------------------------

/// Patch request for an existing worker. `None` leaves the field unchanged;
/// this lets the mobile app update bank/payroll ref, next-of-kin or county
/// without round-tripping fields it did not touch.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWorkerRequest {
    #[serde(default)]
    pub home_county: Option<String>,
    #[serde(default)]
    pub role_eligibility: Option<Vec<String>>,
    #[serde(default)]
    pub payroll_contract_ref: Option<String>,
    #[serde(default)]
    pub next_of_kin: Option<NextOfKin>,
}

// ---------------------------------------------------------------------------
// Interviews (design §3.3: structured interview record for shortlisted
// borderline candidates)
// ---------------------------------------------------------------------------

/// Structured interview outcomes, stored as snake_case strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterviewOutcome {
    Recommend,
    Hold,
    Reject,
}

impl InterviewOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            InterviewOutcome::Recommend => "recommend",
            InterviewOutcome::Hold => "hold",
            InterviewOutcome::Reject => "reject",
        }
    }

    pub fn parse(s: &str) -> Option<InterviewOutcome> {
        match s {
            "recommend" => Some(InterviewOutcome::Recommend),
            "hold" => Some(InterviewOutcome::Hold),
            "reject" => Some(InterviewOutcome::Reject),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InterviewRecord {
    pub interview_id: Uuid,
    pub case_id: Uuid,
    pub interviewer: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub score: Option<f32>,
    pub notes: String,
    pub outcome: InterviewOutcome,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateInterviewRequest {
    pub interviewer: String,
    #[serde(default)]
    pub scheduled_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub score: Option<f32>,
    #[serde(default)]
    pub notes: String,
    pub outcome: InterviewOutcome,
}

// ---------------------------------------------------------------------------
// Notifications (design §4/§8: posting, payment, appointment, training,
// general communication)
// ---------------------------------------------------------------------------

/// Kinds of worker-facing notification. `payment` is emitted by payroll
/// (M55) via the generic send endpoint; `posting`/`appointment`/`training`
/// are fired automatically by workforce-svc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    Posting,
    Payment,
    Appointment,
    Training,
    Message,
}

impl NotificationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationKind::Posting => "posting",
            NotificationKind::Payment => "payment",
            NotificationKind::Appointment => "appointment",
            NotificationKind::Training => "training",
            NotificationKind::Message => "message",
        }
    }

    pub fn parse(s: &str) -> Option<NotificationKind> {
        match s {
            "posting" => Some(NotificationKind::Posting),
            "payment" => Some(NotificationKind::Payment),
            "appointment" => Some(NotificationKind::Appointment),
            "training" => Some(NotificationKind::Training),
            "message" => Some(NotificationKind::Message),
            _ => None,
        }
    }
}

/// A recorded worker notification. `channel`/`delivery_ref` carry only hashes
/// and channel names — never raw PII (DPA 2019); the field-mobile-svc push
/// adapter resolves the recipient from the worker's registered contact points.
#[derive(Debug, Clone, Serialize)]
pub struct Notification {
    pub notification_id: Uuid,
    pub worker_id: Uuid,
    pub kind: NotificationKind,
    pub channel: Option<Channel>,
    pub subject: String,
    pub body: String,
    pub delivery_ref: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendNotificationRequest {
    pub worker_id: Uuid,
    pub kind: NotificationKind,
    #[serde(default)]
    pub channel: Option<Channel>,
    pub subject: String,
    pub body: String,
    #[serde(default)]
    pub delivery_ref: Option<String>,
}
