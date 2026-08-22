//! Backtest harness: replays a historical time window against a labelled anomaly set
//! and reports precision, recall, and F1 per detector.

use crate::detector::{Detector, SignalStream};
use crate::metrics;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

/// A labelled anomaly entry for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelledAnomaly {
    pub device_id: String,
    pub class: String,
    /// The window in which the anomaly is considered ground truth.
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Per-detector evaluation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorMetrics {
    pub detector: String,
    pub true_positive: usize,
    pub false_positive: usize,
    pub false_negative: usize,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
}

impl DetectorMetrics {
    fn compute(detector: &str, tp: usize, fp: usize, labels: usize) -> Self {
        let fn_ = labels.saturating_sub(tp);
        let precision = if tp + fp == 0 {
            0.0
        } else {
            tp as f64 / (tp + fp) as f64
        };
        let recall = if labels == 0 {
            0.0
        } else {
            tp as f64 / labels as f64
        };
        let f1 = if precision + recall < f64::EPSILON {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        DetectorMetrics {
            detector: detector.to_string(),
            true_positive: tp,
            false_positive: fp,
            false_negative: fn_,
            precision,
            recall,
            f1,
        }
    }
}

/// Run detector against historical signals and compare to labelled anomalies.
pub async fn run_backtest(
    detector: Arc<dyn Detector>,
    signals: SignalStream,
    labels: &[LabelledAnomaly],
) -> DetectorMetrics {
    let name = detector.name().to_string();
    let alerts = detector.evaluate(&signals).await.unwrap_or_default();

    // Build a set of (device_id, class) from labels for quick lookup.
    let _label_set: Vec<(&str, &str)> = labels
        .iter()
        .map(|l| (l.device_id.as_str(), l.class.as_str()))
        .collect();

    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut matched: HashSet<usize> = HashSet::new();

    for alert in &alerts {
        let device = alert.device_id.as_deref().unwrap_or("");
        let matched_label = labels.iter().enumerate().find(|(idx, l)| {
            !matched.contains(idx)
                && l.device_id == device
                && l.class == alert.class
                && alert.timestamp >= l.start
                && alert.timestamp <= l.end
        });
        if let Some((idx, _)) = matched_label {
            tp += 1;
            matched.insert(idx);
        } else {
            fp += 1;
        }
    }

    let result = DetectorMetrics::compute(&name, tp, fp, labels.len());

    // Update prometheus gauges.
    let m = metrics::global();
    m.backtest_precision
        .with_label_values(&[&name])
        .set(result.precision);
    m.backtest_recall
        .with_label_values(&[&name])
        .set(result.recall);

    result
}

/// Run backtest across multiple detectors and return one result per detector.
pub async fn run_backtest_suite(
    detectors: Vec<Arc<dyn Detector>>,
    signals: SignalStream,
    labels: &[LabelledAnomaly],
) -> Vec<DetectorMetrics> {
    let mut results = Vec::with_capacity(detectors.len());
    for det in detectors {
        results.push(run_backtest(det, signals.clone(), labels).await);
    }
    results
}
