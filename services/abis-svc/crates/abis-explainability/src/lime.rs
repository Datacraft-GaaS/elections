//! Production trains a local linear model on perturbed samples.

use crate::ExplainError;

pub trait LimeExplainer {
    fn explain(
        &self,
        feature_vector: &[f64],
        n_perturbations: usize,
    ) -> Result<Vec<f64>, ExplainError>;
}

pub struct LocalLimeExplainer {
    pub kernel_width: f64,
}

impl LimeExplainer for LocalLimeExplainer {
    fn explain(
        &self,
        feature_vector: &[f64],
        _n_perturbations: usize,
    ) -> Result<Vec<f64>, ExplainError> {
        if feature_vector.is_empty() {
            return Err(ExplainError::InvalidInput(
                "feature_vector must not be empty".into(),
            ));
        }
        let n = feature_vector.len();
        Ok(vec![1.0 / n as f64; n])
    }
}
