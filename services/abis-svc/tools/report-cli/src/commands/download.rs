use crate::api_client::ReportsClient;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::io::Write;

pub async fn run(
    client: &ReportsClient,
    artifact_id: &str,
    out_path: &str,
    verify: bool,
) -> Result<()> {
    // Fetch metadata for size and expected hash.
    let meta = client.get_artifact_meta(artifact_id).await?;

    // Stream artifact bytes.
    let bytes = client
        .get_bytes(&format!("/api/v1/artifacts/{artifact_id}/download"))
        .await?;

    // Write to disk.
    let mut file = std::fs::File::create(out_path).context("create output file")?;
    file.write_all(&bytes).context("write artifact")?;
    println!("Downloaded {} bytes -> {out_path}", bytes.len());

    if verify {
        let actual_hex = sha256_hex(&bytes);
        if actual_hex == meta.sha256_hex {
            println!("SHA-256 OK: {actual_hex}");
        } else {
            eprintln!("SHA-256 MISMATCH:");
            eprintln!("  expected: {}", meta.sha256_hex);
            eprintln!("  actual:   {actual_hex}");
            std::process::exit(2);
        }
    }
    Ok(())
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
