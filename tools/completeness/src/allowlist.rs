//! Reads `.completeness-allowlist.yaml` from the repo root.
//! The tool reads this file but does NOT create it.
//!
//! Expected file format:
//!
//! ```yaml
//! entries:
//!   - id: "src/foo.rs:42:todo"
//!     reason: "Tracked in ADR-0007"
//!     adr: "ADR-0007"
//!     expires: "2026-12-31"
//!   - id: "services/bvr-svc/crates/bvr-domain/src/lib.rs:10:unimplemented"
//!     reason: "Placeholder pending WP-3.5"
//!     expires: "2027-01-01"
//! ```

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistEntry {
    /// Must match Finding::id  (file:line:kind)
    pub id: String,
    pub reason: String,
    #[serde(default)]
    pub adr: Option<String>,
    /// ISO-8601 date (YYYY-MM-DD) after which this entry is no longer valid
    #[serde(default)]
    pub expires: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Allowlist {
    #[serde(default)]
    pub entries: Vec<AllowlistEntry>,
    #[serde(skip)]
    pub index: HashMap<String, AllowlistEntry>,
}

impl Allowlist {
    fn build_index(&mut self) {
        self.index = self
            .entries
            .iter()
            .cloned()
            .map(|e| (e.id.clone(), e))
            .collect();
    }

    pub fn get(&self, id: &str) -> Option<&AllowlistEntry> {
        self.index.get(id)
    }

    /// Returns true if the entry exists AND has not expired.
    pub fn is_active(&self, id: &str) -> bool {
        let Some(entry) = self.get(id) else {
            return false;
        };
        let today = chrono::Local::now().date_naive();
        if let Some(exp) = &entry.expires {
            if let Ok(d) = NaiveDate::parse_from_str(exp, "%Y-%m-%d") {
                return today <= d;
            }
        }
        true
    }
}

pub fn load(path: &Path) -> Result<Allowlist> {
    if !path.exists() {
        // Allowlist is optional; an absent file means no suppressions.
        return Ok(Allowlist::default());
    }
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading allowlist {}", path.display()))?;
    let mut al: Allowlist = serde_yaml::from_str(&content)
        .with_context(|| format!("parsing allowlist {}", path.display()))?;
    al.build_index();
    Ok(al)
}
