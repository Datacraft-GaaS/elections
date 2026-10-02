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

//! CloudEvent envelope builders for adjudication lifecycle events.

use uuid::Uuid;

use crate::domain::{AdjudicationCase, AdjudicationDecision};

pub const CASE_OPENED: &str = "eel.abis.case_opened.v1";
pub const CASE_ASSIGNED: &str = "eel.abis.case_assigned.v1";
pub const CASE_DECIDED: &str = "eel.abis.case_decided.v1";
pub const CASE_ESCALATED: &str = "eel.abis.case_escalated.v1";

const SOURCE: &str = "spiffe://eel.local/service/abis-adjudication";

/// Build an unsigned CloudEvent envelope for case_opened.
pub fn case_opened_event(
    case: &AdjudicationCase,
) -> Result<eel_events_rs::envelope::Envelope, eel_events_rs::error::EventError> {
    eel_events_rs::EventBuilder::new()
        .event_type(CASE_OPENED)
        .source(SOURCE)
        .stream_id(case.id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(case.id.to_string())
        .data(serde_json::json!({
            "case_id": case.id.to_string(),
            "enrolment_job_id": case.enrolment_job_id.map(|u| u.to_string()),
            "priority": format!("{:?}", case.priority),
            "tier": format!("{:?}", case.tier),
            "opened_at": case.opened_at.to_rfc3339(),
            "sla_deadline_at": case.sla_deadline_at.to_rfc3339(),
        }))
        .data_class(eel_events_rs::envelope::DataClass::B)
        .build_unsigned()
}

/// Build an unsigned CloudEvent envelope for case_assigned.
pub fn case_assigned_event(
    case_id: Uuid,
    reviewer_id: Uuid,
) -> Result<eel_events_rs::envelope::Envelope, eel_events_rs::error::EventError> {
    eel_events_rs::EventBuilder::new()
        .event_type(CASE_ASSIGNED)
        .source(SOURCE)
        .stream_id(case_id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(case_id.to_string())
        .data(serde_json::json!({
            "case_id": case_id.to_string(),
            "reviewer_id": reviewer_id.to_string(),
        }))
        .data_class(eel_events_rs::envelope::DataClass::B)
        .build_unsigned()
}

/// Build an unsigned CloudEvent envelope for case_decided.
pub fn case_decided_event(
    case_id: Uuid,
    decision: &AdjudicationDecision,
) -> Result<eel_events_rs::envelope::Envelope, eel_events_rs::error::EventError> {
    let decision_json = serde_json::to_value(decision).unwrap_or(serde_json::Value::Null);
    eel_events_rs::EventBuilder::new()
        .event_type(CASE_DECIDED)
        .source(SOURCE)
        .stream_id(case_id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(case_id.to_string())
        .data(serde_json::json!({
            "case_id": case_id.to_string(),
            "decision": decision_json,
        }))
        .data_class(eel_events_rs::envelope::DataClass::B)
        .build_unsigned()
}

/// Build an unsigned CloudEvent envelope for case_escalated.
pub fn case_escalated_event(
    case_id: Uuid,
    reason: &str,
) -> Result<eel_events_rs::envelope::Envelope, eel_events_rs::error::EventError> {
    eel_events_rs::EventBuilder::new()
        .event_type(CASE_ESCALATED)
        .source(SOURCE)
        .stream_id(case_id.to_string())
        .seq(0)
        .hlc(chrono::Utc::now().timestamp_millis().to_string())
        .subject(case_id.to_string())
        .data(serde_json::json!({
            "case_id": case_id.to_string(),
            "reason": reason,
        }))
        .data_class(eel_events_rs::envelope::DataClass::B)
        .build_unsigned()
}
