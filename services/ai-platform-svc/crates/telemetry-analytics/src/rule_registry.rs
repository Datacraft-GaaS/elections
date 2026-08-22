//! Rule registry: loads detector configurations from YAML and supports hot-reload.
//!
//! File format: services/ai-platform-svc/config/telemetry-rules.yaml
//!
//! Each rule entry:
//!   id: string              - unique rule identifier
//!   class: string           - detector class name
//!   severity_threshold: f64 - minimum score to fire (detector-specific semantics)
//!   enabled: bool           - whether the detector runs

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    pub id: String,
    pub class: String,
    pub severity_threshold: f64,
    pub enabled: bool,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RulesFile {
    rules: Vec<RuleEntry>,
}

#[derive(Debug, Clone)]
pub struct RuleRegistry {
    path: PathBuf,
    rules: Arc<RwLock<HashMap<String, RuleEntry>>>,
}

impl RuleRegistry {
    /// Load rules from the given YAML path.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        let registry = Self {
            path: path.clone(),
            rules: Arc::new(RwLock::new(HashMap::new())),
        };
        registry.reload()?;
        Ok(registry)
    }

    /// Parse and replace the in-memory rules from the YAML file on disk.
    pub fn reload(&self) -> Result<(), String> {
        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| format!("failed to read rules file {:?}: {e}", self.path))?;
        let parsed: RulesFile = serde_yaml::from_str(&content)
            .map_err(|e| format!("failed to parse rules YAML: {e}"))?;

        // Validate: every entry must have a non-empty id and class.
        for rule in &parsed.rules {
            if rule.id.is_empty() {
                return Err("rule entry has empty id".into());
            }
            if rule.class.is_empty() {
                return Err(format!("rule '{}' has empty class", rule.id));
            }
        }

        let mut map = self
            .rules
            .write()
            .map_err(|e| format!("lock poisoned: {e}"))?;
        map.clear();
        for rule in parsed.rules {
            map.insert(rule.id.clone(), rule);
        }
        info!(path = ?self.path, count = map.len(), "rule registry reloaded");
        Ok(())
    }

    /// Look up a rule by id.
    pub fn get(&self, id: &str) -> Option<RuleEntry> {
        self.rules.read().ok().and_then(|m| m.get(id).cloned())
    }

    /// Return all rules (cloned).
    pub fn all(&self) -> Vec<RuleEntry> {
        self.rules
            .read()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Return only enabled rules.
    pub fn enabled(&self) -> Vec<RuleEntry> {
        self.all().into_iter().filter(|r| r.enabled).collect()
    }

    /// Upsert a rule (used by the admin API).
    pub fn upsert(&self, rule: RuleEntry) -> Result<(), String> {
        let mut map = self
            .rules
            .write()
            .map_err(|e| format!("lock poisoned: {e}"))?;
        map.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Delete a rule by id.  Returns true if the rule existed.
    pub fn delete(&self, id: &str) -> Result<bool, String> {
        let mut map = self
            .rules
            .write()
            .map_err(|e| format!("lock poisoned: {e}"))?;
        Ok(map.remove(id).is_some())
    }

    /// Watch the YAML file for changes and reload automatically.
    /// Spawns a background task; the JoinHandle is returned for the caller to manage.
    pub fn watch_hot_reload(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc;
        use std::time::Duration;

        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let (tx, rx) = mpsc::channel();
            let mut watcher = match RecommendedWatcher::new(
                tx,
                Config::default().with_poll_interval(Duration::from_secs(2)),
            ) {
                Ok(w) => w,
                Err(e) => {
                    error!("failed to create file watcher: {e}");
                    return;
                }
            };
            if let Err(e) = watcher.watch(&path, RecursiveMode::NonRecursive) {
                error!("failed to watch {:?}: {e}", path);
                return;
            }
            for event in rx {
                match event {
                    Ok(_) => {
                        if let Err(e) = self.reload() {
                            error!("hot-reload failed: {e}");
                        }
                    }
                    Err(e) => error!("watcher error: {e}"),
                }
            }
        })
    }
}
