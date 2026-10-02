// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Manifest loading: YAML file listing UI URLs to audit.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub url: String,
    pub name: String,
    pub module: String,
    #[serde(default = "default_level")]
    pub level: String,
}

fn default_level() -> String {
    "AA".to_string()
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub urls: Vec<ManifestEntry>,
}

pub fn load(path: &Path) -> Result<Manifest> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading manifest {}", path.display()))?;
    let manifest: Manifest =
        serde_yaml::from_str(&content).with_context(|| "parsing manifest YAML")?;
    Ok(manifest)
}

/// Slugify a URL for use as a filename.
pub fn url_slug(url: &str) -> String {
    url.replace("http://", "")
        .replace("https://", "")
        .replace(['/', ':', '.', '?', '&', '=', '#'], "_")
        .trim_matches('_')
        .to_string()
}
