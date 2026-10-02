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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoterCard {
    pub id: Uuid,
    pub voter_id: Uuid,
    pub card_number: String,
    pub batch_id: Uuid,
    pub status: CardStatus,
    pub issued_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CardStatus {
    Ordered,
    Printed,
    Dispatched,
    Collected,
    Deactivated,
    Blacklisted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardBatch {
    pub id: Uuid,
    pub election_id: Uuid,
    pub constituency_id: String,
    pub order_count: u64,
    pub manufacturer_ref: Option<String>,
    pub status: BatchStatus,
    pub ordered_at: DateTime<Utc>,
    pub received_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchStatus {
    Draft,
    Submitted,
    InProduction,
    Received,
    Distributed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRecord {
    pub id: Uuid,
    pub card_id: Uuid,
    pub voter_id: Uuid,
    pub collected_at: DateTime<Utc>,
    pub station_id: String,
    pub officer_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlacklistEntry {
    pub id: Uuid,
    pub card_number: String,
    pub reason: String,
    pub blacklisted_at: DateTime<Utc>,
    pub blacklist_source: BlacklistSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlacklistSource {
    LostReport,
    StolenReport,
    Deactivation,
    SystemFlag,
}
