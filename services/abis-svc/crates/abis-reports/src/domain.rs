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
use sqlx::FromRow;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums mirroring SQL CHECK constraints exactly
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ReportClass {
    #[serde(rename = "operational")]
    Operational,
    #[serde(rename = "statistical")]
    Statistical,
    #[serde(rename = "regulatory")]
    Regulatory,
    #[serde(rename = "legal")]
    Legal,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "financial")]
    Financial,
    #[serde(rename = "executive")]
    Executive,
    #[serde(rename = "ad_hoc")]
    AdHoc,
}

impl std::fmt::Display for ReportClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Operational => "operational",
            Self::Statistical => "statistical",
            Self::Regulatory => "regulatory",
            Self::Legal => "legal",
            Self::Public => "public",
            Self::Financial => "financial",
            Self::Executive => "executive",
            Self::AdHoc => "ad_hoc",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum DataClass {
    #[serde(rename = "restricted")]
    Restricted,
    #[serde(rename = "sensitive")]
    Sensitive,
    #[serde(rename = "public")]
    Public,
}

impl std::fmt::Display for DataClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Restricted => "restricted",
            Self::Sensitive => "sensitive",
            Self::Public => "public",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RendererFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "csv")]
    Csv,
    #[serde(rename = "parquet")]
    Parquet,
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "xlsx")]
    Xlsx,
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "zip")]
    Zip,
}

impl std::fmt::Display for RendererFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Parquet => "parquet",
            Self::Pdf => "pdf",
            Self::Html => "html",
            Self::Png => "png",
            Self::Xlsx => "xlsx",
            Self::Svg => "svg",
            Self::Zip => "zip",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum RunStatus {
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "extracting")]
    Extracting,
    #[serde(rename = "rendering")]
    Rendering,
    #[serde(rename = "signing")]
    Signing,
    #[serde(rename = "anchoring")]
    Anchoring,
    #[serde(rename = "delivering")]
    Delivering,
    #[serde(rename = "complete")]
    Complete,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Queued => "queued",
            Self::Extracting => "extracting",
            Self::Rendering => "rendering",
            Self::Signing => "signing",
            Self::Anchoring => "anchoring",
            Self::Delivering => "delivering",
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum DeliveryStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "sending")]
    Sending,
    #[serde(rename = "delivered")]
    Delivered,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "abandoned")]
    Abandoned,
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Sending => "sending",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
            Self::Abandoned => "abandoned",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Database structs (sqlx::FromRow)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportDefinition {
    pub definition_id: Uuid,
    pub slug: String,
    pub title: String,
    pub report_class: String,
    pub audience: Vec<String>,
    pub data_class: String,
    pub sql_template_ref: Option<String>,
    pub renderer_pipeline: Vec<String>,
    pub requires_hsm_sign: bool,
    pub requires_tlog_anchor: bool,
    pub default_schedule: Option<String>,
    pub timezone: String,
    pub retention_years: i16,
    pub template_version: String,
    pub cosign_signature: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub retired_at: Option<DateTime<Utc>>,
    pub retirement_reason: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportRun {
    pub run_id: Uuid,
    pub definition_id: Uuid,
    pub slug: String,
    pub window_from: DateTime<Utc>,
    pub window_to: DateTime<Utc>,
    pub status: String,
    pub requested_by: String,
    pub requested_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub rows_extracted: Option<i64>,
    pub extraction_ms: Option<i32>,
    pub render_ms: Option<i32>,
    pub total_ms: Option<i32>,
    pub failure_reason: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub trace_id: Option<Vec<u8>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportArtifact {
    pub artifact_id: Uuid,
    pub run_id: Uuid,
    pub format: String,
    pub blob_ref: String,
    pub sha256: Vec<u8>,
    pub size_bytes: i64,
    pub page_count: Option<i32>,
    pub row_count: Option<i64>,
    pub hsm_signature: Option<Vec<u8>>,
    pub hsm_key_role: Option<String>,
    pub rendered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportTlogAnchor {
    pub anchor_id: Uuid,
    pub run_id: Uuid,
    pub artifact_id: Uuid,
    pub tlog_leaf_index: i64,
    pub tlog_root_hash: Vec<u8>,
    pub inclusion_proof: Vec<u8>,
    pub sth_size: i64,
    pub witness_cosigs: serde_json::Value,
    pub anchored_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportSubscriber {
    pub subscriber_id: Uuid,
    pub name: String,
    pub channel: String,
    pub endpoint: String,
    pub endpoint_hash: Vec<u8>,
    pub audience: String,
    pub auth_ref: Option<String>,
    pub pgp_pubkey_ref: Option<String>,
    pub preferred_format: String,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspension_reason: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportSubscription {
    pub subscription_id: Uuid,
    pub subscriber_id: Uuid,
    pub definition_id: Uuid,
    pub format: String,
    pub filter: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportDelivery {
    pub delivery_id: Uuid,
    pub run_id: Uuid,
    pub subscription_id: Uuid,
    pub artifact_id: Uuid,
    pub status: String,
    pub attempt_count: i16,
    pub max_attempts: i16,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub remote_receipt_id: Option<String>,
    pub remote_receipt_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ReportAccessLogEntry {
    pub access_id: Uuid,
    pub artifact_id: Uuid,
    pub subject_id: Option<String>,
    pub ip_prefix: Option<String>,
    pub user_agent_hash: Option<Vec<u8>>,
    pub accessed_at: DateTime<Utc>,
    pub byte_range: Option<String>,
}
