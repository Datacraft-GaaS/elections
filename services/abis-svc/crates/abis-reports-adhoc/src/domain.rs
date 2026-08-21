use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClass {
    Restricted,
    Sensitive,
    Public,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Private,
    Shared,
    Team,
    Org,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Sql,
    QueryBuilder,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewFormat {
    Json,
    Markdown,
    Docx,
    Csv,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SharePermission {
    View,
    Run,
    Edit,
    Admin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnippetKind {
    Cte,
    Where,
    Select,
    MarkdownPartial,
}

// ---------------------------------------------------------------------------
// Main structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhocReport {
    pub adhoc_id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub author_id: Uuid,
    pub data_class: DataClass,
    pub visibility: Visibility,
    pub published_as_definition_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhocReportVersion {
    pub version_id: Uuid,
    pub adhoc_id: Uuid,
    pub version_number: i32,
    pub source_type: SourceType,
    pub source: String,
    pub parameters: serde_json::Value,
    pub allowed_tables: Vec<String>,
    pub column_projection: Option<serde_json::Value>,
    pub render_pipeline: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub commit_message: Option<String>,
    pub linted_at: Option<DateTime<Utc>>,
    pub lint_report: serde_json::Value,
    pub safety_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhocPreview {
    pub preview_id: Uuid,
    pub version_id: Uuid,
    pub parameter_values: serde_json::Value,
    pub sample_limit: i32,
    pub rows_returned: Option<i32>,
    pub elapsed_ms: Option<i32>,
    pub preview_format: PreviewFormat,
    pub preview_bytes: Option<Vec<u8>>,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhocShare {
    pub share_id: Uuid,
    pub adhoc_id: Uuid,
    pub shared_with: Uuid,
    pub permission: SharePermission,
    pub granted_by: Uuid,
    pub granted_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdhocSnippet {
    pub snippet_id: Uuid,
    pub author_id: Uuid,
    pub name: String,
    pub body: String,
    pub kind: SnippetKind,
    pub visibility: Visibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Request / response DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportRequest {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub data_class: DataClass,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub visibility: Option<Visibility>,
    pub data_class: Option<DataClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVersionRequest {
    pub source_type: SourceType,
    pub source: String,
    pub parameters: Option<serde_json::Value>,
    pub allowed_tables: Vec<String>,
    pub commit_message: Option<String>,
    pub render_pipeline: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewRequest {
    pub parameter_values: Option<serde_json::Value>,
    pub sample_limit: Option<i32>,
    pub format: Option<PreviewFormat>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub schedule: Option<String>,
    pub subscribers: Option<Vec<Uuid>>,
    pub formats: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    pub shared_with: Uuid,
    pub permission: SharePermission,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSnippetRequest {
    pub name: String,
    pub body: String,
    pub kind: SnippetKind,
    pub visibility: Option<Visibility>,
}
