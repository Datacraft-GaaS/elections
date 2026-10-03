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
use tracing::{info, warn};

/// Apply SpiceDB schema from .zed files in a directory.
/// Uses the SpiceDB HTTP API (REST gateway on :8443 by default).
pub async fn apply(endpoint: &str, token: &str, schema_dir: Option<&str>, yes: bool) -> Result<()> {
    info!(endpoint = %endpoint, "applying SpiceDB schema");

    let dir = schema_dir.unwrap_or("platform/spicedb/schema");
    let desired_schema = load_schema(dir)?;
    let current_schema = fetch_schema(endpoint, token).await?;

    let current_val = serde_json::json!({ "schema": current_schema });
    let desired_val = serde_json::json!({ "schema": desired_schema });

    let changes = crate::compute_diff(&current_val, &desired_val);

    if changes.is_empty() {
        info!("SpiceDB schema is up to date");
        return Ok(());
    }

    println!("SpiceDB schema diff:");
    for change in &changes {
        println!("  {}", change);
    }

    if !yes {
        warn!("dry-run: pass --yes to apply schema changes");
        return Ok(());
    }

    write_schema(endpoint, token, &desired_schema).await?;
    info!("SpiceDB schema applied");
    Ok(())
}

fn load_schema(dir: &str) -> Result<String> {
    let mut schema = String::new();
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading schema directory {}", dir))?;

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("zed"))
        .collect();
    paths.sort();

    for path in &paths {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        schema.push_str(&content);
        schema.push('\n');
    }

    Ok(schema)
}

async fn fetch_schema(endpoint: &str, token: &str) -> Result<String> {
    let url = format!(
        "https://{}/v1/schema/read",
        endpoint.trim_start_matches("https://")
    );
    let client = reqwest::Client::builder().use_rustls_tls().build()?;
    let resp = client
        .post(&url)
        .bearer_auth(token)
        .json(&serde_json::json!({}))
        .send()
        .await
        .with_context(|| format!("POST {}", url))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(String::new());
    }

    let body: serde_json::Value = resp
        .error_for_status()
        .context("reading SpiceDB schema")?
        .json()
        .await
        .context("deserializing SpiceDB schema response")?;

    Ok(body
        .get("schemaText")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

async fn write_schema(endpoint: &str, token: &str, schema: &str) -> Result<()> {
    let url = format!(
        "https://{}/v1/schema/write",
        endpoint.trim_start_matches("https://")
    );
    let client = reqwest::Client::builder().use_rustls_tls().build()?;
    client
        .post(&url)
        .bearer_auth(token)
        .json(&serde_json::json!({ "schema": schema }))
        .send()
        .await
        .with_context(|| format!("POST {}", url))?
        .error_for_status()
        .context("writing SpiceDB schema")?;
    Ok(())
}
