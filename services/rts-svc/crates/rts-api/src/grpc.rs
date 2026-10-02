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

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormEventRequest { pub form_id: String, pub event_type: String, pub payload: serde_json::Value }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormEventResponse { pub accepted: bool, pub event_id: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyQueryRequest { pub scope: String, pub code: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTally { pub candidate_id: String, pub votes: u64 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TallyQueryResponse { pub snapshot_id: Option<String>, pub candidate_totals: Vec<CandidateTally>, pub merkle_root: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest { pub from_snapshot_id: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationEvent { pub snapshot_id: String, pub merkle_root: String, pub signed_url: String, pub published_at: String }

pub async fn submit_form_event(req: FormEventRequest) -> FormEventResponse {
    info!(form_id=%req.form_id, "grpc: SubmitFormEvent (stub)");
    FormEventResponse { accepted: true, event_id: uuid::Uuid::new_v4().to_string() }
}
pub async fn query_tally(req: TallyQueryRequest) -> TallyQueryResponse {
    info!(scope=%req.scope, code=%req.code, "grpc: QueryTally (stub)");
    TallyQueryResponse { snapshot_id: None, candidate_totals: vec![], merkle_root: None }
}
