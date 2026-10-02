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

//! Core audit orchestration: browser invocation and stub mode.

use crate::axe::{self, AxeViolation};
use crate::manifest::ManifestEntry;
use crate::runner::RUNNER_SCRIPT;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    pub url: String,
    pub name: String,
    pub module: String,
    pub level: String,
    pub violations: Vec<AxeViolation>,
    pub passes: usize,
    pub incomplete: usize,
    pub status: String,
    pub error: Option<String>,
}

impl AuditResult {
    pub fn is_passing(&self) -> bool {
        self.violations.is_empty() && self.error.is_none()
    }
}

/// Run axe audit for one entry, using stub dir if provided.
pub fn audit_entry(entry: &ManifestEntry, stub_dir: Option<&Path>) -> Result<AuditResult> {
    let raw = if let Some(dir) = stub_dir {
        load_stub(entry, dir)?
    } else {
        run_playwright(entry)?
    };

    let axe_result =
        axe::parse(&raw).with_context(|| format!("parsing axe output for {}", entry.url))?;

    let violations = axe::filter_violations(&axe_result.violations, &entry.level);
    let status = if violations.is_empty() {
        "pass".to_string()
    } else {
        "fail".to_string()
    };

    Ok(AuditResult {
        url: entry.url.clone(),
        name: entry.name.clone(),
        module: entry.module.clone(),
        level: entry.level.clone(),
        violations,
        passes: axe_result.passes.len(),
        incomplete: axe_result.incomplete.len(),
        status,
        error: None,
    })
}

fn load_stub(entry: &ManifestEntry, dir: &Path) -> Result<String> {
    let slug = crate::manifest::url_slug(&entry.url);
    let path: PathBuf = dir.join(format!("{slug}.json"));
    if !path.exists() {
        // Fall back to a generic pass fixture if no specific stub file exists
        let pass_path = dir.join("axe-pass.json");
        if pass_path.exists() {
            return std::fs::read_to_string(&pass_path)
                .with_context(|| format!("reading stub {}", pass_path.display()));
        }
        bail!(
            "stub file not found: {} (and no axe-pass.json fallback)",
            path.display()
        );
    }
    std::fs::read_to_string(&path).with_context(|| format!("reading stub {}", path.display()))
}

fn run_playwright(entry: &ManifestEntry) -> Result<String> {
    use std::io::Write;

    // Write runner script to a temp file
    let mut tmp = tempfile::NamedTempFile::new().context("creating temp file for runner script")?;
    tmp.write_all(RUNNER_SCRIPT.as_bytes())
        .context("writing runner script")?;
    let script_path = tmp.path().to_path_buf();

    let output = std::process::Command::new("node")
        .arg(&script_path)
        .arg(&entry.url)
        .arg(&entry.level)
        .output();

    match output {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!(
                "node not found. Install Node.js and run: npm install -g playwright axe-core\n\
                 Alternatively, set EEL_WCAG_STUB_DIR to a directory with pre-recorded axe results."
            );
        }
        Err(e) => bail!("running node: {e}"),
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.contains("Cannot find module 'playwright'") {
                    bail!(
                        "playwright not installed. Run: npm install -g playwright axe-core\n\
                         Alternatively, set EEL_WCAG_STUB_DIR to a directory with pre-recorded axe results."
                    );
                }
                bail!(
                    "runner exited with status {}: {}",
                    out.status,
                    stderr.trim()
                );
            }
            String::from_utf8(out.stdout).context("runner output is not UTF-8")
        }
    }
}

/// Audit all entries in the manifest.
pub fn audit_all(entries: &[ManifestEntry], stub_dir: Option<&Path>) -> Vec<AuditResult> {
    entries
        .iter()
        .map(|e| {
            audit_entry(e, stub_dir).unwrap_or_else(|err| AuditResult {
                url: e.url.clone(),
                name: e.name.clone(),
                module: e.module.clone(),
                level: e.level.clone(),
                violations: vec![],
                passes: 0,
                incomplete: 0,
                status: "error".to_string(),
                error: Some(err.to_string()),
            })
        })
        .collect()
}
