use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub level: Level,
    pub description: String,
    #[serde(default)]
    pub mapped_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Level {
    L2,
    L3,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::L2 => write!(f, "L2"),
            Level::L3 => write!(f, "L3"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ManifestFile {
    requirements: Vec<Requirement>,
}

pub fn load(path: &Path) -> Result<Vec<Requirement>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read manifest: {}", path.display()))?;
    let mf: ManifestFile = serde_yaml::from_str(&content).context("invalid ASVS manifest YAML")?;
    Ok(mf.requirements)
}

pub fn default_manifest_path(repo_root: &Path) -> std::path::PathBuf {
    repo_root.join("tools/asvs-check/asvs-manifest.yaml")
}
