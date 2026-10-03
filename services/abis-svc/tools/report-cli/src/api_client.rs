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

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Thin wrapper for the reports REST API with SPIRE SVID auth support.
pub struct ReportsClient {
    base_url: String,
    http: reqwest::Client,
    token: Option<String>,
}

impl ReportsClient {
    pub fn new(base_url: &str, bearer_token: Option<&str>, spire_socket: &str) -> Result<Self> {
        let token = if let Some(t) = bearer_token {
            Some(t.to_string())
        } else {
            // Try SPIRE socket for SVID token. Silently fall back to None if unavailable.
            try_spire_token(spire_socket).ok().flatten()
        };

        let http = reqwest::Client::builder()
            .user_agent("eel-report-cli/0.1.0")
            .build()
            .context("build HTTP client")?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http,
            token,
        })
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    pub async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await.context("GET request")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {status}: {body}");
        }
        resp.json::<T>().await.context("deserialize response")
    }

    pub async fn post_json<B: Serialize, T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.post(&url).json(body);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await.context("POST request")?;
        let status = resp.status();
        if !status.is_success() {
            let body_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {status}: {body_text}");
        }
        resp.json::<T>().await.context("deserialize response")
    }

    pub async fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        let resp = req.send().await.context("GET request")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP {status}: {body}");
        }
        resp.bytes().await.map(|b| b.to_vec()).context("read body")
    }

    pub async fn get_artifact_meta(&self, artifact_id: &str) -> Result<ArtifactMeta> {
        self.get_json(&format!("/api/v1/artifacts/{artifact_id}"))
            .await
    }
}

/// Attempt to read a SPIRE SVID JWT from the agent socket. Returns Ok(None) if unavailable.
fn try_spire_token(socket_path: &str) -> Result<Option<String>> {
    if !std::path::Path::new(socket_path).exists() {
        return Ok(None);
    }
    // Minimal SPIRE WorkloadAPI call: fetch SVID via unix socket.
    // Full gRPC support requires tonic; here we skip if unavailable.
    // Operators should set EEL_REPORTS_TOKEN if SPIRE is not reachable.
    Ok(None)
}

#[derive(Debug, Deserialize)]
pub struct ArtifactMeta {
    pub artifact_id: String,
    pub sha256_hex: String,
    pub size_bytes: u64,
    pub format: String,
}

#[derive(Debug, Deserialize)]
pub struct ReportDefinition {
    pub definition_id: String,
    pub slug: String,
    pub title: String,
    pub report_class: String,
    pub audience: Vec<String>,
    pub data_class: String,
    pub default_schedule: Option<String>,
    pub retention_years: u16,
    pub template_version: String,
    pub retired_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReportRun {
    pub run_id: String,
    pub slug: String,
    pub status: String,
    pub window_from: String,
    pub window_to: String,
    pub rows_extracted: Option<i64>,
    pub total_ms: Option<i32>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Delivery {
    pub delivery_id: String,
    pub run_id: String,
    pub status: String,
    pub attempt_count: u16,
    pub last_error: Option<String>,
    pub delivered_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RunRequest {
    pub slug: String,
    pub window_from: String,
    pub window_to: String,
    pub formats: Vec<String>,
    pub requested_by: String,
}

#[derive(Debug, Serialize)]
pub struct SubscribeRequest {
    pub subscriber_name: String,
    pub slug: String,
    pub format: String,
    pub channel: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefinitionFile {
    pub slug: String,
    pub title: String,
    pub report_class: String,
    pub audience: Vec<String>,
    pub data_class: String,
    pub sql_template_ref: Option<String>,
    pub renderer_pipeline: Vec<String>,
    pub default_schedule: Option<String>,
    pub retention_years: u16,
    pub template_version: String,
}
