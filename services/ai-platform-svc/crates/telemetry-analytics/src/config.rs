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

//! Environment-driven configuration for telemetry-analytics.
//!
//! Variables:
//!   EEL_TELEMETRY_KAFKA_TOPIC   - Kafka topic for inbound signals (default: eel.telemetry.signal.v1)
//!   EEL_TELEMETRY_INGEST_URL    - HTTP URL of the telemetry-ingest crate alert endpoint
//!   EEL_ECC_WEBHOOK_URL         - Webhook URL for ECC dispatch alerts
//!   EEL_TELEMETRY_RULES_PATH    - Path to telemetry-rules.yaml (default: config/telemetry-rules.yaml)

use std::env;

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub kafka_topic: String,
    pub ingest_url: String,
    pub ecc_webhook_url: String,
    pub rules_path: String,
    pub kafka_brokers: String,
    pub kafka_group_id: String,
}

impl TelemetryConfig {
    /// Load from environment, returning an error if required vars are missing.
    pub fn from_env() -> Result<Self, String> {
        let ingest_url = env::var("EEL_TELEMETRY_INGEST_URL")
            .map_err(|_| "EEL_TELEMETRY_INGEST_URL is required")?;
        let ecc_webhook_url =
            env::var("EEL_ECC_WEBHOOK_URL").map_err(|_| "EEL_ECC_WEBHOOK_URL is required")?;

        Ok(Self {
            kafka_topic: env::var("EEL_TELEMETRY_KAFKA_TOPIC")
                .unwrap_or_else(|_| "eel.telemetry.signal.v1".into()),
            ingest_url,
            ecc_webhook_url,
            rules_path: env::var("EEL_TELEMETRY_RULES_PATH")
                .unwrap_or_else(|_| "config/telemetry-rules.yaml".into()),
            kafka_brokers: env::var("EEL_KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9092".into()),
            kafka_group_id: env::var("EEL_TELEMETRY_GROUP_ID")
                .unwrap_or_else(|_| "eel-telemetry-analytics".into()),
        })
    }
}
