//! NFIQ 2.2 score calibrator for local population adaptation.
//!
//! Allows recalibration of the score distribution when the local population
//! (e.g., a Kenyan enrolment cohort) shows systematic quality bias relative
//! to the NIST training population.
//!
//! Method: percentile-rank mapping.  A calibration corpus is scored with the
//! base model; the resulting distribution is used to remap raw scores so that
//! the output distribution is approximately uniform [0, 100].
//!
//! Reference: NIST SP 500-333, Section 6.

use crate::model::RandomForestModel;
use crate::{GrayImage, Nfiq22Error};

/// Calibrator that holds a percentile mapping derived from a calibration corpus.
#[derive(Debug, Clone)]
pub struct Calibrator {
    /// 101 percentile breakpoints: `breakpoints[k]` is the raw score at the k-th
    /// percentile of the calibration distribution.
    breakpoints: Vec<f32>,
}

impl Calibrator {
    /// Build a calibrator from a corpus of raw scores (uncalibrated outputs of the RF).
    pub fn from_raw_scores(mut raw_scores: Vec<f32>) -> Self {
        raw_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = raw_scores.len();
        let breakpoints: Vec<f32> = (0..=100)
            .map(|pct| {
                if n == 0 {
                    return 50.0;
                }
                let idx = (pct as f32 / 100.0 * (n - 1) as f32).round() as usize;
                raw_scores[idx.min(n - 1)]
            })
            .collect();
        Self { breakpoints }
    }

    /// Fit calibrator from a set of images using a model.
    pub fn fit(images: &[GrayImage], model: &RandomForestModel) -> Result<Self, Nfiq22Error> {
        let mut raw_scores = Vec::with_capacity(images.len());
        for img in images {
            let features = crate::features::extract_features(img)?;
            let arr = features.as_array();
            raw_scores.push(model.predict(&arr));
        }
        Ok(Self::from_raw_scores(raw_scores))
    }

    /// Map a raw score to a calibrated score in [0, 100].
    pub fn calibrate(&self, raw: f32) -> u8 {
        // Binary-search for the percentile bucket that contains `raw`.
        let pos = self.breakpoints.partition_point(|&bp| bp < raw);
        pos.min(100) as u8
    }
}

impl Default for Calibrator {
    /// Identity calibrator (no adjustment).
    fn default() -> Self {
        Self {
            breakpoints: (0..=100).map(|p| p as f32).collect(),
        }
    }
}
