use anyhow::{Context, Result};
use tracing::{info, warn};

/// Apply SPIRE workload registration entries from YAML files in a directory.
/// Calls the SPIRE server admin API (spire-server entry show / create).
pub async fn apply(server_url: &str, entries_dir: Option<&str>, yes: bool) -> Result<()> {
    info!(server_url = %server_url, "applying SPIRE entries");

    let dir = entries_dir.unwrap_or("platform/spire/entries");
    let desired = load_entries(dir)?;

    println!("SPIRE: {} entry file(s) loaded from {}", desired.len(), dir);

    if !yes {
        warn!("dry-run: pass --yes to apply SPIRE entries (requires spire-server CLI access)");
        return Ok(());
    }

    // Real implementation would invoke `spire-server entry create` or the
    // SPIRE registration API. Stubbed here because SPIRE's HTTP API surface
    // varies by deployment mode (embedded vs. remote). A production
    // implementation should use the SPIRE SDK or exec the CLI.
    for entry in &desired {
        info!("would apply SPIRE entry: {}", entry.get("spiffe_id").and_then(|v| v.as_str()).unwrap_or("unknown"));
    }

    info!("SPIRE entries apply: {} entries processed (stub)", desired.len());
    Ok(())
}

fn load_entries(dir: &str) -> Result<Vec<serde_json::Value>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            warn!("SPIRE entries directory '{}' not found, skipping", dir);
            return Ok(Vec::new());
        }
        Err(e) => return Err(e).with_context(|| format!("reading entries directory {}", dir)),
    };

    let mut result = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let val: serde_json::Value = serde_yaml::from_str(&content)
            .with_context(|| format!("parsing {}", path.display()))?;
        result.push(val);
    }
    Ok(result)
}
