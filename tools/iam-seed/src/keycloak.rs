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

/// Apply a Keycloak realm configuration by diffing the current state
/// against the desired state from a local JSON file.
pub async fn apply(realm: &str, url: &str, realm_file: Option<&str>, yes: bool) -> Result<()> {
    info!(realm = %realm, url = %url, "applying Keycloak realm configuration");

    let desired = load_desired(realm, realm_file)?;
    let current = fetch_current(url, realm).await?;

    let changes = crate::compute_diff(&current, &desired);

    if changes.is_empty() {
        info!("Keycloak realm '{}' is up to date, no changes needed", realm);
        return Ok(());
    }

    println!("Keycloak realm '{}' diff:", realm);
    for change in &changes {
        println!("  {}", change);
    }

    if !yes {
        warn!("dry-run: pass --yes to apply {} change(s)", changes.len());
        return Ok(());
    }

    put_realm(url, realm, &desired).await?;
    info!("Keycloak realm '{}' applied ({} change(s))", realm, changes.len());
    Ok(())
}

fn load_desired(realm: &str, realm_file: Option<&str>) -> Result<serde_json::Value> {
    let path = realm_file
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("platform/keycloak/{}.json", realm));

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("reading realm file {}", path))?;
    serde_json::from_str(&content).with_context(|| format!("parsing realm file {}", path))
}

async fn fetch_current(url: &str, realm: &str) -> Result<serde_json::Value> {
    let admin_url = format!("{}/admin/realms/{}", url.trim_end_matches('/'), realm);
    let token = std::env::var("KEYCLOAK_ADMIN_TOKEN")
        .unwrap_or_else(|_| std::env::var("KEYCLOAK_TOKEN").unwrap_or_default());

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()?;

    let resp = client
        .get(&admin_url)
        .bearer_auth(&token)
        .send()
        .await
        .with_context(|| format!("GET {}", admin_url))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        // Realm does not exist yet; treat as empty object.
        return Ok(serde_json::Value::Object(Default::default()));
    }

    resp.error_for_status()
        .with_context(|| format!("unexpected response from {}", admin_url))?
        .json::<serde_json::Value>()
        .await
        .context("deserializing current realm JSON")
}

async fn put_realm(url: &str, realm: &str, desired: &serde_json::Value) -> Result<()> {
    let admin_url = format!("{}/admin/realms/{}", url.trim_end_matches('/'), realm);
    let token = std::env::var("KEYCLOAK_ADMIN_TOKEN")
        .unwrap_or_else(|_| std::env::var("KEYCLOAK_TOKEN").unwrap_or_default());

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()?;

    client
        .put(&admin_url)
        .bearer_auth(&token)
        .json(desired)
        .send()
        .await
        .with_context(|| format!("PUT {}", admin_url))?
        .error_for_status()
        .with_context(|| format!("applying realm to {}", admin_url))?;

    Ok(())
}
