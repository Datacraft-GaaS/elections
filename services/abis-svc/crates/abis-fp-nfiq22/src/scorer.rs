//! NFIQ 2.2 quality scorer.
//!
//! Reference: NIST SP 500-333, Section 5 (Random Forest inference pipeline).

use crate::features::extract_features;
use crate::model::RandomForestModel;
use crate::{GrayImage, Nfiq22Error};

/// Compute NFIQ 2.2 quality score using the default (synthetic) model.
/// Returns an integer quality value in [0, 100] (higher is better).
///
/// For production use, load the NIST-published model via
/// `RandomForestModel::from_path` and call `score_with_model`.
pub fn nfiq22_score(image: &GrayImage) -> Result<u8, Nfiq22Error> {
    let model = RandomForestModel::default_model();
    score_with_model(image, &model)
}

/// Compute quality using a provided model.
pub fn score_with_model(image: &GrayImage, model: &RandomForestModel) -> Result<u8, Nfiq22Error> {
    let features = extract_features(image)?;
    let arr = features.as_array();
    let raw = model.predict(&arr);
    Ok(raw.round().clamp(0.0, 100.0) as u8)
}
