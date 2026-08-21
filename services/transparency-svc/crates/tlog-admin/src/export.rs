//! `tlog-admin export --to <s3-url>` — export a tree snapshot to MinIO/S3.

use anyhow::Result;

pub async fn run(to_url: &str) -> Result<()> {
    // TODO(WP-2.x): stream leaves + tile data to MinIO tar.gz.
    tracing::info!(to_url = %to_url, "export started (stub)");
    println!("export to {} not yet implemented (WP-2.1 stub)", to_url);
    Ok(())
}
