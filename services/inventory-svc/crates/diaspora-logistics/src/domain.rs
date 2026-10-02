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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerStatus {
    Packed,
    CustomsCleared,
    InTransit,
    Received,
    Reconciled,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BallotContainer {
    pub id: String,
    pub container_code: String,
    pub station_id: String,
    pub contents_manifest_url: Option<String>,
    pub seal_id: Option<String>,
    pub status: ContainerStatus,
    pub packed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerEventType {
    Packed,
    CustomsCleared,
    Dispatched,
    Received,
    SealBroken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerCustodyEvent {
    pub id: String,
    pub container_id: String,
    pub event_type: ContainerEventType,
    pub actor_id: String,
    pub location: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManifestStatus {
    Submitted,
    Approved,
    Rejected,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreClearanceManifest {
    pub id: String,
    pub container_id: String,
    pub mfa_ref: Option<String>,
    pub kra_ref: Option<String>,
    pub airline_awb: Option<String>,
    pub status: ManifestStatus,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingPolicy {
    DiplomaticPouch,
    AirlineCargo,
    CharterFlight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubRoute {
    pub id: String,
    pub post_code: String,
    pub hub_code: String,
    pub routing_policy: RoutingPolicy,
    pub approved_at: Option<DateTime<Utc>>,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrivalRecord {
    pub id: String,
    pub container_id: String,
    pub expected_count: u64,
    pub received_count: u64,
    pub discrepancy: i64,
    pub resolved_at: Option<DateTime<Utc>>,
}

pub fn custody_chain(events: &[ContainerCustodyEvent]) -> Vec<&ContainerCustodyEvent> {
    let mut sorted: Vec<&ContainerCustodyEvent> = events.iter().collect();
    sorted.sort_by_key(|e| e.timestamp);
    sorted
}
