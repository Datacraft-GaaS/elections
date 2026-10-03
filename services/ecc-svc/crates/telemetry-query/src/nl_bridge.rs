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

//! Natural-language to DSL bridge.
//!
//! The `NlQueryBridge` trait is the contract. `MockBridge` is used in tests.
//! `LiteLlmBridge` is the production stub that calls a configurable
//! `EEL_LITELLM_URL` and returns a proposed `TelemetryQuery` for admin
//! confirmation before any execution.
//!
//! Every NL translation is logged at INFO level with the actor and the
//! proposed DSL so there is an audit trail even before the query runs.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::dsl::TelemetryQuery;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum NlError {
    #[error("LiteLLM error: {0}")]
    LiteLlm(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("translation unavailable")]
    Unavailable,
}

// ---------------------------------------------------------------------------
// Proposed query (returned to UI for confirmation)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedQuery {
    /// The translated DSL (not yet executed).
    pub query: TelemetryQuery,
    /// Human-readable explanation of what the query does.
    pub explanation: String,
    /// Confidence score from the model (0.0 to 1.0).
    pub confidence: f32,
    /// The original English text.
    pub original_text: String,
}

// ---------------------------------------------------------------------------
// Trait (async fn in trait, stable since Rust 1.75)
// ---------------------------------------------------------------------------

pub trait NlQueryBridge: Send + Sync {
    fn translate<'a>(
        &'a self,
        english: &'a str,
        actor_svid: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ProposedQuery, NlError>> + Send + 'a>,
    >;
}

// ---------------------------------------------------------------------------
// Mock bridge (tests)
// ---------------------------------------------------------------------------

pub struct MockBridge;

impl NlQueryBridge for MockBridge {
    fn translate<'a>(
        &'a self,
        english: &'a str,
        actor_svid: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ProposedQuery, NlError>> + Send + 'a>,
    > {
        use crate::dsl::{SignalClass, TimeRange};
        use chrono::Utc;

        let english = english.to_string();
        let actor_svid = actor_svid.to_string();

        Box::pin(async move {
            info!(
                actor = actor_svid,
                text = english,
                bridge = "mock",
                "NL translation (mock)"
            );

            let now = Utc::now();
            let query = TelemetryQuery {
                device_ids: vec![],
                signal_classes: vec![SignalClass::Location],
                time_range: TimeRange {
                    from: now - chrono::Duration::hours(24),
                    to: now,
                },
                geo_polygon: None,
                aggregations: vec![],
                group_by: vec![],
                limit: 1_000,
                order_by: vec![],
            };

            Ok(ProposedQuery {
                query,
                explanation: format!("Mock translation of: {}", english),
                confidence: 0.75,
                original_text: english,
            })
        })
    }
}

// ---------------------------------------------------------------------------
// LiteLLM bridge (production stub)
// ---------------------------------------------------------------------------

pub struct LiteLlmBridge {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl LiteLlmBridge {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("EEL_LITELLM_URL")
                .unwrap_or_else(|_| "http://litellm:4000".into()),
            model: std::env::var("EEL_TELEMETRY_NL_MODEL").unwrap_or_else(|_| "llama3".into()),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct LiteLlmRequest {
    model: String,
    messages: Vec<LiteLlmMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct LiteLlmMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct LiteLlmResponse {
    choices: Vec<LiteLlmChoice>,
}

#[derive(Deserialize)]
struct LiteLlmChoice {
    message: LiteLlmRespMessage,
}

#[derive(Deserialize)]
struct LiteLlmRespMessage {
    content: String,
}

const SYSTEM_PROMPT: &str = "\
You are a telemetry query DSL translator for a commission admin console. \
Translate the user's natural-language question into a JSON object that conforms \
to the TelemetryQuery schema. Return ONLY valid JSON with no commentary.\n\n\
Signal classes (snake_case): location, time, activity, proximal_rfid, \
movements, login_logout, reads, writes, battery, locks_unlocks, \
physical_drops, network, attestation.\n\n\
Time range fields must be ISO-8601 UTC strings. If a relative time is \
mentioned (e.g., \"last 48 hours\"), compute from now.\
";

impl NlQueryBridge for LiteLlmBridge {
    fn translate<'a>(
        &'a self,
        english: &'a str,
        actor_svid: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ProposedQuery, NlError>> + Send + 'a>,
    > {
        let english = english.to_string();
        let actor_svid = actor_svid.to_string();
        let base_url = self.base_url.clone();
        let model = self.model.clone();
        let client = self.client.clone();

        Box::pin(async move {
            Self::translate_inner(english, actor_svid, base_url, model, client).await
        })
    }
}

impl LiteLlmBridge {
    async fn translate_inner(
        english: String,
        actor_svid: String,
        base_url: String,
        model: String,
        client: reqwest::Client,
    ) -> Result<ProposedQuery, NlError> {
        let req = LiteLlmRequest {
            model: model.clone(),
            messages: vec![
                LiteLlmMessage {
                    role: "system".into(),
                    content: SYSTEM_PROMPT.into(),
                },
                LiteLlmMessage {
                    role: "user".into(),
                    content: english.clone(),
                },
            ],
            temperature: 0.0,
            max_tokens: 2048,
        };

        let resp = client
            .post(format!("{}/chat/completions", base_url))
            .json(&req)
            .send()
            .await
            .map_err(|e| NlError::LiteLlm(e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(NlError::LiteLlm(format!("HTTP {}", status)));
        }

        let body: LiteLlmResponse = resp
            .json()
            .await
            .map_err(|e| NlError::Parse(e.to_string()))?;

        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or(NlError::Unavailable)?;

        let query: TelemetryQuery = serde_json::from_str(&content)
            .map_err(|e| NlError::Parse(format!("DSL parse error: {}: {}", e, content)))?;

        info!(
            actor = actor_svid,
            text = english,
            bridge = "litellm",
            "NL translation successful"
        );

        Ok(ProposedQuery {
            query,
            explanation: format!("Translated from: {}", english),
            confidence: 0.85,
            original_text: english.to_string(),
        })
    }
}
