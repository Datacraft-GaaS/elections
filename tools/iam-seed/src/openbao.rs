use anyhow::{Context, Result};
use tracing::{info, warn};

/// Apply OpenBao policies from a directory of .hcl files,
/// and mount configurations from a directory of .yaml files.
pub async fn apply(
    url: &str,
    token: &str,
    policies_dir: Option<&str>,
    mounts_dir: Option<&str>,
    yes: bool,
) -> Result<()> {
    info!(url = %url, "applying OpenBao configuration");

    let pol_dir = policies_dir.unwrap_or("platform/openbao/policies");
    apply_policies(url, token, pol_dir, yes).await?;

    let mnt_dir = mounts_dir.unwrap_or("platform/openbao/mounts");
    apply_mounts(url, token, mnt_dir, yes).await?;

    Ok(())
}

async fn apply_policies(url: &str, token: &str, dir: &str, yes: bool) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("reading policies directory {}", dir))?;

    let client = reqwest::Client::builder().use_rustls_tls().build()?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("hcl") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .with_context(|| format!("invalid filename: {}", path.display()))?
            .to_string();

        let desired_hcl = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        let current_hcl = fetch_policy(url, token, &client, &name).await?;

        if normalize_hcl(&current_hcl) == normalize_hcl(&desired_hcl) {
            info!("policy '{}' is up to date", name);
            continue;
        }

        println!("Policy '{}' changed:", name);
        println!("  current hash: {}", sha256_hex(&current_hcl));
        println!("  desired hash: {}", sha256_hex(&desired_hcl));

        if !yes {
            warn!("dry-run: pass --yes to apply policy '{}'", name);
            continue;
        }

        put_policy(url, token, &client, &name, &desired_hcl).await?;
        info!("policy '{}' applied", name);
    }

    Ok(())
}

async fn fetch_policy(
    url: &str,
    token: &str,
    client: &reqwest::Client,
    name: &str,
) -> Result<String> {
    let endpoint = format!("{}/v1/sys/policies/acl/{}", url.trim_end_matches('/'), name);
    let resp = client
        .get(&endpoint)
        .header("X-Vault-Token", token)
        .send()
        .await
        .with_context(|| format!("GET {}", endpoint))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(String::new());
    }

    let body: serde_json::Value = resp
        .error_for_status()
        .with_context(|| format!("fetching policy {}", name))?
        .json()
        .await
        .context("deserializing policy response")?;

    Ok(body
        .get("data")
        .and_then(|d| d.get("policy"))
        .and_then(|p| p.as_str())
        .unwrap_or("")
        .to_string())
}

async fn put_policy(
    url: &str,
    token: &str,
    client: &reqwest::Client,
    name: &str,
    hcl: &str,
) -> Result<()> {
    let endpoint = format!("{}/v1/sys/policies/acl/{}", url.trim_end_matches('/'), name);
    let body = serde_json::json!({ "policy": hcl });
    client
        .put(&endpoint)
        .header("X-Vault-Token", token)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("PUT {}", endpoint))?
        .error_for_status()
        .with_context(|| format!("applying policy {}", name))?;
    Ok(())
}

async fn apply_mounts(url: &str, token: &str, dir: &str, yes: bool) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("reading mounts directory {}", dir))?;

    let client = reqwest::Client::builder().use_rustls_tls().build()?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let desired: serde_json::Value = serde_yaml::from_str(&content)
            .with_context(|| format!("parsing {}", path.display()))?;

        let mounts = desired
            .get("mounts")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default();

        for mount in &mounts {
            let mount_path = mount
                .get("path")
                .and_then(|p| p.as_str())
                .with_context(|| format!("mount missing 'path' in {}", path.display()))?;

            let current = fetch_mount(url, token, &client, mount_path).await?;
            let changes = crate::compute_diff(&current, mount);

            if changes.is_empty() {
                info!("mount '{}' is up to date", mount_path);
                continue;
            }

            println!("Mount '{}' diff:", mount_path);
            for change in &changes {
                println!("  {}", change);
            }

            if !yes {
                warn!("dry-run: pass --yes to apply mount '{}'", mount_path);
                continue;
            }

            ensure_mount(url, token, &client, mount_path, mount).await?;
            info!("mount '{}' applied", mount_path);
        }
    }

    Ok(())
}

async fn fetch_mount(
    url: &str,
    token: &str,
    client: &reqwest::Client,
    mount_path: &str,
) -> Result<serde_json::Value> {
    let endpoint = format!("{}/v1/sys/mounts", url.trim_end_matches('/'));
    let resp = client
        .get(&endpoint)
        .header("X-Vault-Token", token)
        .send()
        .await
        .with_context(|| format!("GET {}", endpoint))?
        .error_for_status()
        .context("listing mounts")?
        .json::<serde_json::Value>()
        .await
        .context("deserializing mounts")?;

    let key = format!("{}/", mount_path.trim_end_matches('/'));
    Ok(resp.get(&key).cloned().unwrap_or(serde_json::Value::Null))
}

async fn ensure_mount(
    url: &str,
    token: &str,
    client: &reqwest::Client,
    mount_path: &str,
    config: &serde_json::Value,
) -> Result<()> {
    let endpoint = format!(
        "{}/v1/sys/mounts/{}",
        url.trim_end_matches('/'),
        mount_path.trim_matches('/')
    );
    client
        .post(&endpoint)
        .header("X-Vault-Token", token)
        .json(config)
        .send()
        .await
        .with_context(|| format!("POST {}", endpoint))?
        .error_for_status()
        .with_context(|| format!("enabling mount {}", mount_path))?;
    Ok(())
}

fn normalize_hcl(s: &str) -> String {
    // Trim trailing whitespace per line and collapse blank lines for comparison.
    s.lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn sha256_hex(s: &str) -> String {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(s.as_bytes());
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_trailing_whitespace() {
        let a = "path \"kv/*\" {  \n  capabilities = [\"read\"]  \n}\n";
        let b = "path \"kv/*\" {\n  capabilities = [\"read\"]\n}";
        assert_eq!(normalize_hcl(a), normalize_hcl(b));
    }
}
