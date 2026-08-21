//! PAD policy enforcing iBeta Level 1 and Level 2 PAI coverage.
//!
//! Fail-closed: any unacceptable APCER or BPCER causes rejection.

use crate::PadError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// iBeta conformance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IbetaLevel {
    Level1,
    Level2,
}

/// PAI species per ISO/IEC 30107-3 taxonomy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaiSpecies {
    PrintedPhoto,
    DisplayReplay,
    SiliconeArtefact,
    GelatinArtefact,
    LatexArtefact,
    EcoflexArtefact,
    Deepfake,
    VoiceClone,
    Other(String),
}

/// Detector performance on a PAI species.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorPerformance {
    /// Attack Presentation Classification Error Rate (lower is better).
    pub apcer: f64,
    /// Bona Fide Presentation Classification Error Rate (lower is better).
    pub bpcer: f64,
}

/// Policy outcome after evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyOutcome {
    pub passed: bool,
    pub level: IbetaLevel,
    pub violations: Vec<String>,
}

/// Enforces PAI species coverage and error rate budgets.
pub struct PadPolicy {
    level: IbetaLevel,
    /// Maximum acceptable APCER per species.
    max_apcer: f64,
    /// Maximum acceptable BPCER overall.
    max_bpcer: f64,
}

impl PadPolicy {
    pub fn new(level: IbetaLevel, max_apcer: f64, max_bpcer: f64) -> Result<Self, PadError> {
        if !(0.0..=1.0).contains(&max_apcer) || !(0.0..=1.0).contains(&max_bpcer) {
            return Err(PadError::InvalidInput {
                detail: "error rates must be in [0, 1]".into(),
            });
        }
        Ok(Self {
            level,
            max_apcer,
            max_bpcer,
        })
    }

    /// iBeta Level 2 preset: APCER <= 2%, BPCER <= 10%.
    pub fn ibeta_level2() -> Self {
        Self {
            level: IbetaLevel::Level2,
            max_apcer: 0.02,
            max_bpcer: 0.10,
        }
    }

    /// Evaluates detector performance map against the policy.
    ///
    /// Returns `PolicyOutcome` with all violations listed.
    ///
    /// # Errors
    /// Returns `PadError::InvalidInput` if the performance map is empty.
    pub fn enforce(
        &self,
        performances: &HashMap<PaiSpecies, DetectorPerformance>,
    ) -> Result<PolicyOutcome, PadError> {
        if performances.is_empty() {
            return Err(PadError::InvalidInput {
                detail: "no detector performance data supplied".into(),
            });
        }

        let required = self.required_species();
        let mut violations = Vec::new();

        for species in &required {
            match performances.get(species) {
                None => {
                    violations.push(format!("missing coverage for PAI species {:?}", species));
                }
                Some(perf) => {
                    if perf.apcer > self.max_apcer {
                        violations.push(format!(
                            "APCER {:.4} > {:.4} for {:?}",
                            perf.apcer, self.max_apcer, species
                        ));
                    }
                    if perf.bpcer > self.max_bpcer {
                        violations.push(format!(
                            "BPCER {:.4} > {:.4} for {:?}",
                            perf.bpcer, self.max_bpcer, species
                        ));
                    }
                }
            }
        }

        Ok(PolicyOutcome {
            passed: violations.is_empty(),
            level: self.level,
            violations,
        })
    }

    fn required_species(&self) -> Vec<PaiSpecies> {
        let level1 = vec![
            PaiSpecies::PrintedPhoto,
            PaiSpecies::DisplayReplay,
            PaiSpecies::SiliconeArtefact,
        ];
        match self.level {
            IbetaLevel::Level1 => level1,
            IbetaLevel::Level2 => {
                let mut species = level1;
                species.extend([
                    PaiSpecies::GelatinArtefact,
                    PaiSpecies::LatexArtefact,
                    PaiSpecies::EcoflexArtefact,
                    PaiSpecies::Deepfake,
                    PaiSpecies::VoiceClone,
                ]);
                species
            }
        }
    }
}
