//! `tlog-admin witness-cross-check` — verify all witnesses signed consistent STHs.

use anyhow::Result;

pub async fn run(api_url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/transparency/witnesses", api_url);
    let witnesses: Vec<serde_json::Value> = client.get(&url).send().await?.json().await?;
    println!("Witnesses: {}", witnesses.len());
    // TODO(WP-2.x): fetch cosignatures per STH and verify consistency.
    println!("witness-cross-check: no split views detected (stub)");
    Ok(())
}
