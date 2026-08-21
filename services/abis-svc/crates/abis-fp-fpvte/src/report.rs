//! FpVTE-format report emission.
//!
//! Generates JSON (primary), CSV (tabular), and a text summary mimicking the
//! NIST FpVTE report structure.
//!
//! Reference: P. Grother, "NIST FpVTE 2003," NISTIR 7123; P. Grother, M. Indovina,
//! "PFT-III," NIST IR 8307.

use crate::FpVteError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One point on an ROC curve: (FMR, FNMR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocPoint {
    pub threshold: f32,
    pub fmr: f32,
    pub fnmr: f32,
}

/// Rank-N identification accuracy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankAccuracy {
    pub rank: usize,
    pub accuracy: f32,
}

/// Per-demographic fairness slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessSlice {
    pub label: String,
    pub fmr_at_001: f32,
    pub fnmr_at_001: f32,
}

/// Full FpVTE evaluation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub matcher_name: String,
    pub n_probes: usize,
    pub n_gallery: usize,
    pub roc: Vec<RocPoint>,
    pub rank_accuracies: Vec<RankAccuracy>,
    pub fairness_slices: Vec<FairnessSlice>,
    /// Equal Error Rate (FMR = FNMR operating point).
    pub eer: f32,
    /// FMR at the threshold where FNMR = 0.01.
    pub fmr_at_fnmr_001: Option<f32>,
    /// Rank-1 identification accuracy.
    pub rank1_accuracy: f32,
}

impl Report {
    /// Write the report as JSON to `output_dir/{matcher_name}_report.json`.
    pub fn write_json(&self, output_dir: &Path) -> Result<(), FpVteError> {
        std::fs::create_dir_all(output_dir).map_err(|e| FpVteError::ReportWrite(e.to_string()))?;
        let path = output_dir.join(format!("{}_report.json", self.matcher_name));
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| FpVteError::ReportWrite(e.to_string()))?;
        std::fs::write(&path, json).map_err(|e| FpVteError::ReportWrite(e.to_string()))?;
        tracing::info!("FpVTE report written to {}", path.display());
        Ok(())
    }

    /// Write the ROC curve as CSV to `output_dir/{matcher_name}_roc.csv`.
    pub fn write_roc_csv(&self, output_dir: &Path) -> Result<(), FpVteError> {
        std::fs::create_dir_all(output_dir).map_err(|e| FpVteError::ReportWrite(e.to_string()))?;
        let path = output_dir.join(format!("{}_roc.csv", self.matcher_name));
        let mut csv = String::from("threshold,fmr,fnmr\n");
        for pt in &self.roc {
            csv.push_str(&format!("{},{},{}\n", pt.threshold, pt.fmr, pt.fnmr));
        }
        std::fs::write(&path, csv).map_err(|e| FpVteError::ReportWrite(e.to_string()))?;
        Ok(())
    }

    /// Write a plain-text summary.
    pub fn write_summary(&self, output_dir: &Path) -> Result<(), FpVteError> {
        std::fs::create_dir_all(output_dir).map_err(|e| FpVteError::ReportWrite(e.to_string()))?;
        let path = output_dir.join(format!("{}_summary.txt", self.matcher_name));
        let mut text = format!(
            "FpVTE Report: {}\n\
             Probes: {}  Gallery: {}\n\
             EER: {:.4}\n\
             Rank-1 accuracy: {:.4}\n",
            self.matcher_name, self.n_probes, self.n_gallery, self.eer, self.rank1_accuracy
        );
        if let Some(fmr) = self.fmr_at_fnmr_001 {
            text.push_str(&format!("FMR at FNMR=0.01: {fmr:.6}\n"));
        }
        text.push_str("\nFairness slices:\n");
        for slice in &self.fairness_slices {
            text.push_str(&format!(
                "  {}: FMR={:.4} FNMR={:.4}\n",
                slice.label, slice.fmr_at_001, slice.fnmr_at_001
            ));
        }
        std::fs::write(&path, text).map_err(|e| FpVteError::ReportWrite(e.to_string()))?;
        Ok(())
    }
}
