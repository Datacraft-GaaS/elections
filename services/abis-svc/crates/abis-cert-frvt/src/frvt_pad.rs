//! FRVT PAD track submission.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrvtPadReport {
    pub apcer: f64,
    pub bpcer: f64,
    pub acer: f64,
    pub species_tested: Vec<String>,
}

pub struct FrvtPadSubmission {
    algorithm_id: String,
}

impl FrvtPadSubmission {
    pub fn new(algorithm_id: &str) -> Self {
        Self {
            algorithm_id: algorithm_id.to_owned(),
        }
    }

    /// Packages PAD performance into an FRVT PAD report card.
    pub fn run(&self, apcer: f64, bpcer: f64, species_tested: Vec<String>) -> FrvtPadReport {
        let acer = (apcer + bpcer) / 2.0;
        FrvtPadReport {
            apcer,
            bpcer,
            acer,
            species_tested,
        }
    }

    pub fn algorithm_id(&self) -> &str {
        &self.algorithm_id
    }
}
