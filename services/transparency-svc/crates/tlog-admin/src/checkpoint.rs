//! `tlog-admin checkpoint --domain <d>` — print the latest STH for a domain.

use anyhow::Result;
use tlog_domain::Domain;

pub async fn run(domain: &str, api_url: &str) -> Result<()> {
    let _d: Domain = domain.parse().map_err(|e: tlog_domain::DomainError| {
        anyhow::anyhow!("invalid domain: {}", e)
    })?;
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/transparency/sth/latest", api_url);
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}
