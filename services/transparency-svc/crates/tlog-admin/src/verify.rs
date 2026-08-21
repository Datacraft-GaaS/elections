//! `tlog-admin verify-chain --sth-a <n> --sth-b <n>` — verify consistency between two STHs.

use anyhow::Result;

pub async fn run(sth_a: u64, sth_b: u64, api_url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/v1/transparency/proof/consistency?from_size={}&to_size={}",
        api_url, sth_a, sth_b
    );
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    println!("Consistency proof: {}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}
