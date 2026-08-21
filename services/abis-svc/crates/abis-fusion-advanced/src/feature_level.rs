use crate::FusionError;

pub fn concatenate(features_a: &[f64], features_b: &[f64]) -> Vec<f64> {
    let mut result = features_a.to_vec();
    result.extend_from_slice(features_b);
    result
}

/// Stub: wire real PCA decomposition in production.
pub fn pca_reduce(features: &[f64], n_components: usize) -> Result<Vec<f64>, FusionError> {
    if n_components > features.len() {
        return Err(FusionError::DimensionMismatch {
            expected: features.len(),
            found: n_components,
        });
    }
    Ok(features[..n_components].to_vec())
}

/// KernelPCA trait: implement compute_kernel_matrix + project; production integrates an ndarray-linalg backend.
pub trait KernelPca {
    fn compute_kernel_matrix(&self, features: &[f64]) -> Vec<Vec<f64>>;
    fn project(&self, kernel_matrix: &[Vec<f64>], n_components: usize) -> Vec<f64>;
}
