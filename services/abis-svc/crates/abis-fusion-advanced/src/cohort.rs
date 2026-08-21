use crate::FusionError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortStats {
    pub mean: f64,
    pub std: f64,
    pub count: usize,
}

#[derive(Debug, Default)]
pub struct CohortNormalizer {
    cohorts: HashMap<String, CohortStats>,
}

impl CohortNormalizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_cohort(&mut self, id: String, mean: f64, std: f64, count: usize) {
        self.cohorts.insert(id, CohortStats { mean, std, count });
    }

    pub fn normalize(&self, cohort_id: &str, score: f64) -> Result<f64, FusionError> {
        let stats = self.cohorts.get(cohort_id).ok_or_else(|| {
            FusionError::InsufficientCohortData(format!("cohort '{}' not found", cohort_id))
        })?;
        if stats.std.abs() < f64::EPSILON {
            return Ok(0.0);
        }
        Ok((score - stats.mean) / stats.std)
    }
}
