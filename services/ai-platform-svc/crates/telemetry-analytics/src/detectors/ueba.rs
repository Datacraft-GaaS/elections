//! UebaDetector: thin bridge to the platform/detection/ueba baseline learner.
//!
//! Feeds telemetry signals as entity-typed events; consumes anomaly scores
//! from the ueba-config.yaml score model.
//!
//! Reference: UEBA (User and Entity Behaviour Analytics) as defined in
//! Gartner Market Guide for User and Entity Behavior Analytics, 2015.

use crate::detector::{Alert, AlertSeverity, Detector, DetectorError, SignalStream};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration for UebaDetector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UebaConfig {
    pub rule_id: String,
    /// Anomaly score threshold from the ueba baseline learner.
    pub score_threshold: f64,
    pub enabled: bool,
}

impl Default for UebaConfig {
    fn default() -> Self {
        Self {
            rule_id: "ueba_anomaly".into(),
            score_threshold: 0.75,
            enabled: true,
        }
    }
}

pub struct UebaDetector {
    config: UebaConfig,
}

impl UebaDetector {
    pub fn new(config: UebaConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Detector for UebaDetector {
    fn name(&self) -> &str {
        "UebaDetector"
    }

    fn rule_id(&self) -> &str {
        &self.config.rule_id
    }

    fn enabled(&self) -> bool {
        self.config.enabled
    }

    async fn evaluate(&self, signals: &SignalStream) -> Result<Vec<Alert>, DetectorError> {
        let mut alerts = Vec::new();

        for signal in signals {
            if signal.signal_type != "ueba_score" {
                continue;
            }
            let score = signal.payload["anomaly_score"].as_f64().unwrap_or(0.0);
            if score <= self.config.score_threshold {
                continue;
            }
            let entity_id = signal.payload["entity_id"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let entity_type = signal.payload["entity_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            let mut alert = Alert::new(
                &self.config.rule_id,
                "ueba_anomaly",
                AlertSeverity::Warn,
                serde_json::json!({
                    "entity_id": entity_id,
                    "entity_type": entity_type,
                    "anomaly_score": score,
                    "threshold": self.config.score_threshold,
                }),
            )
            .with_device(signal.device_id.clone());

            if entity_type == "officer" {
                alert = alert.with_officer(&entity_id);
            }
            alerts.push(alert);
        }
        Ok(alerts)
    }
}
