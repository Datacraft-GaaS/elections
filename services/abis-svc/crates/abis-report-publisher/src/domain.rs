//! Domain types mirroring the V027 schema exactly.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Channel type as constrained by eel_report_subscribers.channel CHECK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    #[sqlx(rename = "email")]
    Email,
    #[sqlx(rename = "webhook")]
    Webhook,
    #[sqlx(rename = "kafka")]
    Kafka,
    #[sqlx(rename = "sftp")]
    Sftp,
    #[sqlx(rename = "gov_printer")]
    GovPrinter,
    #[sqlx(rename = "kbc_sftp")]
    KbcSftp,
    #[sqlx(rename = "api_pull")]
    ApiPull,
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Channel::Email => "email",
            Channel::Webhook => "webhook",
            Channel::Kafka => "kafka",
            Channel::Sftp => "sftp",
            Channel::GovPrinter => "gov_printer",
            Channel::KbcSftp => "kbc_sftp",
            Channel::ApiPull => "api_pull",
        };
        write!(f, "{s}")
    }
}

/// Delivery status as constrained by eel_report_deliveries.status CHECK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "snake_case")]
pub enum DeliveryStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "sending")]
    Sending,
    #[sqlx(rename = "delivered")]
    Delivered,
    #[sqlx(rename = "failed")]
    Failed,
    #[sqlx(rename = "abandoned")]
    Abandoned,
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DeliveryStatus::Pending => "pending",
            DeliveryStatus::Sending => "sending",
            DeliveryStatus::Delivered => "delivered",
            DeliveryStatus::Failed => "failed",
            DeliveryStatus::Abandoned => "abandoned",
        };
        write!(f, "{s}")
    }
}

/// A subscriber record from eel_report_subscribers.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscriber {
    pub subscriber_id: Uuid,
    pub name: String,
    pub channel: String,
    /// Endpoint value (URL, email address, topic name, SFTP host, etc.)
    pub endpoint: String,
    /// SHA-256 of endpoint for PII protection.
    pub endpoint_hash: Vec<u8>,
    pub audience: String,
    /// OpenBao path to credentials (e.g. private key, SMTP password).
    pub auth_ref: Option<String>,
    /// OpenBao path to PGP public key for encrypted delivery.
    pub pgp_pubkey_ref: Option<String>,
    pub preferred_format: String,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub suspended_at: Option<DateTime<Utc>>,
    pub suspension_reason: Option<String>,
}

/// A subscription record from eel_report_subscriptions.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Subscription {
    pub subscription_id: Uuid,
    pub subscriber_id: Uuid,
    pub definition_id: Uuid,
    pub format: String,
    pub created_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

/// A delivery attempt record from eel_report_deliveries.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Delivery {
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

/// An artifact record joined during pipeline dispatch.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Artifact {
    pub artifact_id: Uuid,
    pub run_id: Uuid,
    pub format: String,
    pub blob_ref: String,
    pub sha256: Vec<u8>,
    pub size_bytes: i64,
    pub hsm_signature: Option<Vec<u8>>,
    pub rendered_at: DateTime<Utc>,
}

/// Tlog anchor associated with an artifact (may not be present).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TlogAnchor {
    pub anchor_id: Uuid,
    pub run_id: Uuid,
    pub artifact_id: Uuid,
    pub tlog_leaf_index: i64,
}

/// Everything a channel adapter needs to send one delivery.
#[derive(Debug, Clone)]
pub struct DeliveryContext {
    pub delivery: Delivery,
    pub subscriber: Subscriber,
    pub subscription: Subscription,
    pub artifact: Artifact,
    pub tlog_anchor: Option<TlogAnchor>,
}

/// Outcome returned by a channel adapter.
#[derive(Debug)]
pub enum DeliveryOutcome {
    /// Delivered successfully; optional remote receipt.
    Succeeded {
        remote_receipt_id: Option<String>,
        remote_receipt_at: Option<DateTime<Utc>>,
    },
    /// Transient failure; should be retried.
    Retryable { error: String },
    /// Permanent failure; no retry.
    Permanent { error: String },
}
