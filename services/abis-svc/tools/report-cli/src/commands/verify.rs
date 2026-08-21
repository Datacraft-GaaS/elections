use crate::api_client::ReportsClient;
use crate::commands::download::sha256_hex;
use anyhow::{Context, Result};

pub async fn run(client: &ReportsClient, file: &str, artifact_id: &str) -> Result<()> {
    let bytes = std::fs::read(file).context("read local file")?;
    let actual_hex = sha256_hex(&bytes);

    let meta = client.get_artifact_meta(artifact_id).await?;
    let expected_hex = &meta.sha256_hex;

    println!("artifact_id:  {artifact_id}");
    println!("local_file:   {file}");
    println!("expected_sha: {expected_hex}");
    println!("actual_sha:   {actual_hex}");

    if actual_hex == *expected_hex {
        println!("Result: PASS - file matches catalog hash");
        Ok(())
    } else {
        eprintln!("Result: FAIL - SHA-256 mismatch (file may be tampered or corrupt)");
        std::process::exit(2);
    }
}
