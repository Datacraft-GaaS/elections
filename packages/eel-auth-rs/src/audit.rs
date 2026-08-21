use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use crate::error::AuthError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub subject: String,
    pub action: String,
    pub resource: String,
    pub decision: String,
    pub metadata: Value,
}

impl AuditEvent {
    pub fn new(subject: &str, action: &str, resource: &str, decision: &str, metadata: Value) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            subject: subject.to_owned(),
            action: action.to_owned(),
            resource: resource.to_owned(),
            decision: decision.to_owned(),
            metadata,
        }
    }
}

#[async_trait]
pub trait AuditSink: Send + Sync + 'static {
    async fn emit(&self, event: AuditEvent) -> Result<(), AuthError>;
}

pub struct NoopSink;

#[async_trait]
impl AuditSink for NoopSink {
    async fn emit(&self, _event: AuditEvent) -> Result<(), AuthError> {
        Ok(())
    }
}

pub struct LogSink;

#[async_trait]
impl AuditSink for LogSink {
    async fn emit(&self, event: AuditEvent) -> Result<(), AuthError> {
        tracing::info!(
            event_id = %event.event_id,
            subject = %event.subject,
            action = %event.action,
            resource = %event.resource,
            decision = %event.decision,
            "audit"
        );
        Ok(())
    }
}
