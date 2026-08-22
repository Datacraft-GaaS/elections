//! Per-officer session baseline and anomaly detection via Mahalanobis distance.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::keystroke::KeystrokeFeatures;
use crate::mouse::MouseFeatures;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedFeatures {
    pub keystroke: KeystrokeFeatures,
    pub mouse: MouseFeatures,
    /// RFC-3339 timestamp.
    pub extracted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub mean_vec: Vec<f64>,
    /// Diagonal covariance: per-feature variance.
    pub cov_diag: Vec<f64>,
    pub n_samples: usize,
}

/// 30 minutes at 10-second intervals = 180 samples.
const DEFAULT_BASELINE_WINDOW: usize = 180;

pub struct SessionProfile {
    pub officer_id: String,
    pub session_id: String,
    pub baseline: Option<Baseline>,
    pub learning_phase: bool,
    pub sample_count: usize,
    pub window_samples: VecDeque<CombinedFeatures>,
    pub baseline_window: usize,
    /// Last computed Mahalanobis distance, stored for API reads.
    pub last_distance: Option<f64>,
}

impl SessionProfile {
    pub fn new(officer_id: String, session_id: String) -> Self {
        Self {
            officer_id,
            session_id,
            baseline: None,
            learning_phase: true,
            sample_count: 0,
            window_samples: VecDeque::with_capacity(DEFAULT_BASELINE_WINDOW),
            baseline_window: DEFAULT_BASELINE_WINDOW,
            last_distance: None,
        }
    }

    /// Ingest a combined feature sample.
    /// Returns None during the learning phase, Some(mahalanobis_distance) once a baseline exists.
    pub fn ingest(&mut self, features: CombinedFeatures) -> Option<f64> {
        self.sample_count += 1;

        if self.window_samples.len() == self.baseline_window {
            self.window_samples.pop_front();
        }
        self.window_samples.push_back(features.clone());

        if self.learning_phase && self.sample_count >= self.baseline_window {
            self.update_baseline();
            self.learning_phase = false;
        }

        if self.learning_phase {
            return None;
        }

        let baseline = self.baseline.as_ref()?;
        let x = Self::features_to_vec(&features);
        let distance = Self::mahalanobis(&x, baseline);
        self.last_distance = Some(distance);
        Some(distance)
    }

    fn features_to_vec(f: &CombinedFeatures) -> Vec<f64> {
        let k = &f.keystroke;
        let m = &f.mouse;
        let mut v = vec![
            k.mean_dwell,
            k.std_dwell,
            k.mean_flight,
            k.std_flight,
            m.mean_velocity,
            m.std_velocity,
            m.mean_acceleration,
            m.mean_jerk,
            m.mean_curvature,
        ];
        // Append pressure histogram (10 bins).
        v.extend_from_slice(&k.pressure_histogram);
        v
    }

    fn compute_baseline(samples: &VecDeque<CombinedFeatures>) -> Baseline {
        let n = samples.len();
        if n == 0 {
            return Baseline {
                mean_vec: vec![],
                cov_diag: vec![],
                n_samples: 0,
            };
        }
        let vecs: Vec<Vec<f64>> = samples.iter().map(Self::features_to_vec).collect();
        let dim = vecs[0].len();

        let mut mean_vec = vec![0.0f64; dim];
        for v in &vecs {
            for (i, &x) in v.iter().enumerate() {
                mean_vec[i] += x;
            }
        }
        for m in &mut mean_vec {
            *m /= n as f64;
        }

        let mut cov_diag = vec![0.0f64; dim];
        for v in &vecs {
            for (i, &x) in v.iter().enumerate() {
                cov_diag[i] += (x - mean_vec[i]).powi(2);
            }
        }
        for c in &mut cov_diag {
            *c /= n as f64;
            // Floor variance to avoid division by zero.
            if *c < 1e-9 {
                *c = 1e-9;
            }
        }

        Baseline {
            mean_vec,
            cov_diag,
            n_samples: n,
        }
    }

    fn mahalanobis(x: &[f64], baseline: &Baseline) -> f64 {
        baseline
            .mean_vec
            .iter()
            .zip(baseline.cov_diag.iter())
            .zip(x.iter())
            .map(|((mean_i, var_i), xi)| (xi - mean_i).powi(2) / var_i)
            .sum::<f64>()
            .sqrt()
    }

    fn update_baseline(&mut self) {
        self.baseline = Some(Self::compute_baseline(&self.window_samples));
    }
}
