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
pub struct ModelRegistration {
    pub id: Option<String>,
    pub code: String,
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub training_data_description: String,
    pub eval_accuracy: f64,
    pub known_limitations: String,
    pub human_in_loop: bool,
    pub ccb_approval_id: String,
    pub iab_notified_at: Option<DateTime<Utc>>,
    pub model_card_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecord {
    pub id: String,
    pub code: String,
    pub name: String,
    pub version: String,
    pub architecture: String,
    pub training_data_description: String,
    pub eval_accuracy: f64,
    pub known_limitations: String,
    pub human_in_loop: bool,
    pub ccb_approval_id: String,
    pub iab_notified_at: Option<DateTime<Utc>>,
    pub model_card_url: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub id: String,
    pub layer: u8,
    pub severity: Severity,
    pub station_code: String,
    pub description: String,
    pub anomaly_score: f64,
    pub generated_at: DateTime<Utc>,
    pub assigned_analyst_id: Option<String>,
    pub resolution: Option<String>,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLogEntry {
    pub id: String,
    pub model_id: String,
    pub model_version: String,
    pub interaction_type: String,
    pub input_hash: String,
    pub confidence_score: f64,
    pub model_output: serde_json::Value,
    pub human_decision: Option<String>,
    pub decision_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiStripRequest {
    pub text: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiStripResponse {
    pub stripped_text: String,
    pub tokens_replaced: u32,
    pub input_hash: String,
}
