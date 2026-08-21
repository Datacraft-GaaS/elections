use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSubmitted {
    pub event_type: String,
    pub app_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl ApplicationSubmitted {
    pub fn new(app_id: String) -> Self {
        Self {
            event_type: "eel.vprs.application_submitted.v1".to_string(),
            app_id,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationCancelled {
    pub event_type: String,
    pub app_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl ApplicationCancelled {
    pub fn new(app_id: String) -> Self {
        Self {
            event_type: "eel.vprs.application_cancelled.v1".to_string(),
            app_id,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRedeemed {
    pub event_type: String,
    pub app_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl HandoffRedeemed {
    pub fn new(app_id: String) -> Self {
        Self {
            event_type: "eel.vprs.handoff_redeemed.v1".to_string(),
            app_id,
            occurred_at: Utc::now(),
        }
    }
}
