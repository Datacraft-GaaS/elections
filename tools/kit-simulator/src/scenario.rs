use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Telco impairment profile applied per phase.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TelcoProfile {
    /// Simulated packet loss percentage (default 5%).
    #[serde(default = "default_packet_loss")]
    pub packet_loss_pct: f64,
    /// Fraction of kits using satellite fallback.
    #[serde(default)]
    pub satellite_fallback_pct: f64,
    /// Radio tech mix: {"3g": 0.30, "4g": 0.50, "lte": 0.20}
    #[serde(default = "default_tech_mix")]
    pub tech_mix: HashMap<String, f64>,
}

fn default_packet_loss() -> f64 {
    5.0
}

fn default_tech_mix() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("3g".into(), 0.30);
    m.insert("4g".into(), 0.50);
    m.insert("lte".into(), 0.20);
    m
}

/// A single named phase in the election-day timeline.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PhaseSpec {
    pub name: String,
    pub duration_min: u64,
    /// BVI identification requests per second at this phase.
    #[serde(default)]
    pub bvi_rps: f64,
    /// Peak concurrent RTS submissions.
    #[serde(default)]
    pub rts_concurrent: u64,
    /// Portal page requests per second.
    #[serde(default)]
    pub portal_rps: f64,
    /// Concurrent verifier sessions.
    #[serde(default)]
    pub verifier_concurrent: u64,
    /// Personnel check-ins within the phase's 90-min window, if applicable.
    #[serde(default)]
    pub checkins_per_90min: u64,
    /// Kafka ingestion event rate for this phase.
    #[serde(default)]
    pub kafka_events_per_sec: f64,
    #[serde(default)]
    pub telco_profile: TelcoProfile,
}

/// Full election-day scenario consumed by kit-simulator, k6, and Gatling.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    /// Total kit fleet size at 1x scale.
    pub total_kits: u64,
    /// Fraction of fleet running the legacy Android shim.
    pub legacy_android_pct: f64,
    /// Heartbeat period in seconds (target 30s).
    pub heartbeat_interval_secs: u64,
    pub phases: Vec<PhaseSpec>,
}

impl Scenario {
    /// Total scenario duration in minutes.
    pub fn total_duration_min(&self) -> u64 {
        self.phases.iter().map(|p| p.duration_min).sum()
    }

    /// Phase at elapsed time `elapsed_min` (wraps at end for short dry-runs).
    pub fn phase_at(&self, elapsed_min: u64) -> &PhaseSpec {
        let total = self.total_duration_min().max(1);
        let wrapped = elapsed_min % total;
        let mut acc = 0u64;
        for phase in &self.phases {
            acc += phase.duration_min;
            if wrapped < acc {
                return phase;
            }
        }
        self.phases.last().unwrap()
    }
}

pub fn load_scenario(path: &Path) -> Result<Scenario> {
    let f = std::fs::File::open(path).with_context(|| format!("opening scenario {:?}", path))?;
    serde_yaml::from_reader(f).with_context(|| format!("parsing scenario {:?}", path))
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

    fn scenario() -> Scenario {
        load_scenario(&ws_root().join("loadtest/scenarios/election-day.yaml")).unwrap()
    }

    #[test]
    fn load_election_day_scenario() {
        let s = scenario();
        assert_eq!(s.total_kits, 65_000);
        assert!(!s.phases.is_empty());
    }

    #[test]
    fn all_required_phases_present() {
        let names: Vec<&str> = scenario().phases.iter().map(|p| p.name.as_str()).collect();
        for required in &[
            "registration-open",
            "polling-open",
            "morning-peak",
            "mid-day",
            "evening-peak",
            "close",
            "results-flood",
            "petition-window",
        ] {
            assert!(
                names.contains(required),
                "missing phase '{required}' in election-day.yaml"
            );
        }
    }

    #[test]
    fn scenario_spans_full_election_day() {
        // Election day is 12 h of active polling + results; expect >= 720 min total.
        assert!(scenario().total_duration_min() >= 720);
    }

    #[test]
    fn morning_peak_has_nonzero_bvi_rps() {
        let s = scenario();
        let peak = s.phases.iter().find(|p| p.name == "morning-peak").unwrap();
        assert!(peak.bvi_rps > 0.0, "morning-peak must have bvi_rps > 0");
    }

    #[test]
    fn phase_at_wraps_correctly() {
        let s = scenario();
        let total = s.total_duration_min();
        // phase_at(0) and phase_at(total) should return first phase
        assert_eq!(s.phase_at(0).name, s.phases[0].name);
        assert_eq!(s.phase_at(total).name, s.phases[0].name);
    }
}
