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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CalendarEventType {
    NominationOpens,
    NominationCloses,
    CampaignSilence,
    PollingDay,
    PetitionWindow,
    ResultsPublication,
    VoterRegistrationOpen,
    VoterRegistrationClose,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub election_id: Uuid,
    pub event_type: CalendarEventType,
    pub scheduled_at: DateTime<Utc>,
    pub gazette_ref: Option<String>,
    pub statutory_citation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NoticeType {
    CampaignStart,
    PollingDayAnnouncement,
    NominationNotice,
    ResultsNotice,
    PetitionNotice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazetteNotice {
    pub id: Uuid,
    pub notice_type: NoticeType,
    pub body: String,
    pub statutory_rule_ids: Vec<Uuid>,
    pub legal_approver_id: Option<Uuid>,
    pub hsm_signature: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub gov_printer_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    DaysBeforeEvent,
    DaysAfterEvent,
    FixedDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatutoryRule {
    pub id: Uuid,
    pub citation: String,
    pub rule_type: RuleType,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionChannel {
    Email,
    Sms,
    WhatsApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub subscriber_id: Uuid,
    pub channel: SubscriptionChannel,
    pub event_types: Vec<CalendarEventType>,
}
