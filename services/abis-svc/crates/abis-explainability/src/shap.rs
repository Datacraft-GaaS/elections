//! Production plugs into a Python worker via gRPC for exact SHAP values.

use crate::ExplainError;

pub trait ShapExplainer {
    fn explain(&self, feature_vector: &[f64], baseline: &[f64]) -> Result<Vec<f64>, ExplainError>;
}

pub struct LocalShapExplainer {
    pub n_samples: usize,
}

impl ShapExplainer for LocalShapExplainer {
    fn explain(&self, feature_vector: &[f64], baseline: &[f64]) -> Result<Vec<f64>, ExplainError> {
        if feature_vector.is_empty() {
            return Err(ExplainError::InvalidInput(
                "feature_vector must not be empty".into(),
            ));
        }
        if feature_vector.len() != baseline.len() {
            return Err(ExplainError::DimensionMismatch);
        }
        let diffs: Vec<f64> = feature_vector
            .iter()
            .zip(baseline.iter())
            .map(|(a, b)| a - b)
            .collect();
        let total: f64 = diffs.iter().sum();
        if total.abs() < f64::EPSILON {
            return Ok(vec![0.0; feature_vector.len()]);
        }
        Ok(diffs.iter().map(|&d| d / total).collect())
    }
}
