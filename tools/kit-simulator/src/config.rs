use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Load-test profile: SLA values scaled by `scaling_factor`.
/// Fields mirror loadtest/profiles/*.yaml keys exactly.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub scaling_factor: f64,
    pub bvi_auth_p95_ms: u64,
    pub bvi_auth_p99_ms: u64,
    pub bvi_req_per_sec: u64,
    pub rts_submit_p95_ms: u64,
    pub rts_submit_p99_ms: u64,
    pub rts_concurrent: u64,
    #[serde(default = "default_abis_1n_p95")]
    pub abis_1n_p95_s: u64,
    #[serde(default = "default_abis_1n_p99")]
    pub abis_1n_p99_s: u64,
    #[serde(default = "default_abis_enrolments")]
    pub abis_enrolments_per_hour: u64,
    pub portal_p95_ms: u64,
    pub portal_p99_ms: u64,
    pub portal_visitors_per_hour: u64,
    pub verifier_concurrent: u64,
    pub checkins_per_90min: u64,
    #[serde(default = "default_payroll")]
    pub payroll_records_per_run: u64,
    pub kafka_events_per_sec: u64,
    #[serde(default)]
    pub abort_on_anomaly: bool,
}

fn default_abis_1n_p95() -> u64 {
    3
}
fn default_abis_1n_p99() -> u64 {
    6
}
fn default_abis_enrolments() -> u64 {
    10
}
fn default_payroll() -> u64 {
    100
}

impl Profile {
    /// Number of virtual kits at this scale (65k raw, floor 10).
    pub fn kit_count(&self) -> u64 {
        ((65_000.0_f64 * self.scaling_factor) as u64).max(10)
    }

    /// Legacy Android shim kits: 22% of kit_count.
    pub fn legacy_kit_count(&self) -> u64 {
        (self.kit_count() as f64 * 0.22) as u64
    }

    /// Scale a raw SLA float by this profile's factor (floor 1.0).
    pub fn scale(&self, raw: f64) -> f64 {
        (raw * self.scaling_factor).max(1.0)
    }
}

/// Resolve a profile name to its YAML path, searching from `base_dir`.
pub fn profile_path(name: &str, base_dir: &Path) -> PathBuf {
    base_dir.join(format!("{}.yaml", name))
}

pub fn load_profile(name: &str) -> Result<Profile> {
    let path = profile_path(name, Path::new("loadtest/profiles"));
    load_profile_from_path(&path)
}

pub fn load_profile_from_path(path: &Path) -> Result<Profile> {
    let f = std::fs::File::open(path).with_context(|| format!("opening profile {:?}", path))?;
    serde_yaml::from_reader(f).with_context(|| format!("parsing profile {:?}", path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn load_local_profile() {
        let p = load_profile_from_path(&ws_root().join("loadtest/profiles/local.yaml")).unwrap();
        assert!((p.scaling_factor - 0.01).abs() < 1e-9);
        assert!(p.kit_count() >= 10);
        assert_eq!(p.bvi_auth_p95_ms, 200);
    }

    #[test]
    fn kit_count_scales_correctly() {
        let p = load_profile_from_path(&ws_root().join("loadtest/profiles/local.yaml")).unwrap();
        // 65000 * 0.01 = 650
        assert_eq!(p.kit_count(), 650);
    }

    #[test]
    fn legacy_kit_count_is_22_pct() {
        let p = load_profile_from_path(&ws_root().join("loadtest/profiles/local.yaml")).unwrap();
        let expected = (650.0_f64 * 0.22) as u64;
        assert_eq!(p.legacy_kit_count(), expected);
    }

    #[test]
    fn scale_applies_factor() {
        let p = load_profile_from_path(&ws_root().join("loadtest/profiles/local.yaml")).unwrap();
        // 18000 * 0.01 = 180
        assert!((p.scale(18_000.0) - 180.0).abs() < 1.0);
    }
}
