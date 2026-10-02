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

//! Environment-driven configuration for the BVI backend service.
//!
//! Read once at process start (`DATABASE_URL`, `BIND_ADDR`, `EEL_ELECTION_ID`).
//! Deliberately dependency-free so it stays trivially testable.
#![warn(missing_docs)]

use std::env;

/// Runtime configuration for `bvi-api`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BviConfig {
    /// Postgres connection string. Required.
    pub database_url: String,
    /// Socket address to bind, e.g. `0.0.0.0:8088`. Defaults to `0.0.0.0:8088`.
    pub bind_addr: String,
    /// Election context applied where a request does not carry one.
    /// Defaults to the current election cycle placeholder from V3.
    pub election_id: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub missing: Vec<&'static str>,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "missing required environment variables: {:?}", self.missing)
    }
}

impl std::error::Error for ConfigError {}

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8088";
const DEFAULT_ELECTION_ID: &str = "EL-2027";

impl BviConfig {
    /// Load configuration from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut missing = Vec::new();
        let database_url = env::var("DATABASE_URL").unwrap_or_default();
        if database_url.is_empty() {
            missing.push("DATABASE_URL");
        }
        if !missing.is_empty() {
            return Err(ConfigError { missing });
        }
        Ok(Self {
            database_url,
            bind_addr: {
                let v = env::var("BIND_ADDR").unwrap_or_default();
                if v.is_empty() { DEFAULT_BIND_ADDR.to_owned() } else { v }
            },
            election_id: match env::var("EEL_ELECTION_ID") {
                Ok(v) if !v.is_empty() => v,
                _ => DEFAULT_ELECTION_ID.to_owned(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Env vars are process-global; guard mutation behind a mutex and run the
    // two env-dependent tests serially.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn requires_database_url() {
        let _g = ENV_LOCK.lock().unwrap();
        env::remove_var("DATABASE_URL");
        let err = BviConfig::from_env().unwrap_err();
        assert_eq!(err.missing, vec!["DATABASE_URL"]);
    }

    #[test]
    fn applies_defaults_for_optional_vars() {
        let _g = ENV_LOCK.lock().unwrap();
        env::set_var("DATABASE_URL", "postgres://localhost/bvi_test");
        env::remove_var("BIND_ADDR");
        env::remove_var("EEL_ELECTION_ID");
        let cfg = BviConfig::from_env().unwrap();
        assert_eq!(cfg.bind_addr, "0.0.0.0:8088");
        assert_eq!(cfg.election_id, "EL-2027");
    }

    #[test]
    fn honours_explicit_values() {
        let _g = ENV_LOCK.lock().unwrap();
        env::set_var("DATABASE_URL", "postgres://localhost/bvi_test");
        env::set_var("BIND_ADDR", "127.0.0.1:9999");
        env::set_var("EEL_ELECTION_ID", "elec-2026-general");
        let cfg = BviConfig::from_env().unwrap();
        assert_eq!(cfg.bind_addr, "127.0.0.1:9999");
        assert_eq!(cfg.election_id, "elec-2026-general");
    }
}
