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

//! Public contract models (field-mobile-support/support.yaml).

use serde::{Deserialize, Serialize};

/// Device information attached to a support ticket.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceInfo {
    pub model: Option<String>,
    pub os_version: Option<String>,
    pub battery_pct: Option<u8>,
    pub storage_free_mb: Option<u32>,
    pub network_type: Option<String>,
}

/// Request body for creating a support ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicketRequest {
    pub station_code: String,
    pub kit_id_hash: Option<String>,
    pub issue_type: String,
    pub description: String,
    pub severity: String,
    pub device_info: Option<DeviceInfo>,
    pub knowledge_base_article_tried: Option<String>,
}

/// Public support ticket view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicket {
    pub ticket_id: String,
    pub station_code: String,
    pub kit_id_hash: Option<String>,
    pub issue_type: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub resolution_notes: Option<String>,
    pub resolved_by_hash: Option<String>,
    pub device_info: DeviceInfo,
    pub created_at: String,
    pub updated_at: String,
    pub resolved_at: Option<String>,
}

/// Request body for updating a ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketUpdate {
    pub status: Option<String>,
    pub resolution_notes: Option<String>,
    pub resolved_by_hash: Option<String>,
}

/// Public knowledge-base article summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSummary {
    pub article_id: String,
    pub title: String,
    pub category: String,
    pub summary: String,
    pub version: u32,
    pub updated_at: String,
}

/// Public knowledge-base article.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub article_id: String,
    pub title: String,
    pub category: String,
    pub body: String,
    pub steps: Vec<ArticleStep>,
    pub related_article_ids: Vec<String>,
    pub version: u32,
    pub updated_at: String,
}

/// One step in an article's resolution procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleStep {
    pub step_number: u32,
    pub instruction: String,
    pub image_ref: Option<String>,
}
