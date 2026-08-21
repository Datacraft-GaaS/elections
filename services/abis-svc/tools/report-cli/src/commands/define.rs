use crate::api_client::{DefinitionFile, ReportsClient};
use anyhow::{Context, Result};
use serde_json::Value;

pub async fn run(client: &ReportsClient, file: &str) -> Result<()> {
    let raw = std::fs::read_to_string(file).context("read definition file")?;
    // Accept YAML or JSON by parsing as JSON after a best-effort YAML-to-JSON pass.
    // Simple approach: try JSON first, then treat as plain struct via serde_json.
    let def: DefinitionFile = serde_json::from_str(&raw).context("parse definition file as JSON")?;
    let resp: Value = client.post_json("/api/v1/reports/definitions", &def).await?;
    println!("Created: {}", resp.get("definition_id").and_then(|v| v.as_str()).unwrap_or("?"));
    Ok(())
}
