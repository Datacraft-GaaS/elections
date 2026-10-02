// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::FusionError;

pub fn min_max_normalize(scores: &[f64]) -> Result<Vec<f64>, FusionError> {
    if scores.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if (max - min).abs() < f64::EPSILON {
        return Ok(vec![0.5; scores.len()]);
    }
    Ok(scores.iter().map(|&x| (x - min) / (max - min)).collect())
}

pub fn z_score_normalize(scores: &[f64]) -> Result<Vec<f64>, FusionError> {
    if scores.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let n = scores.len() as f64;
    let mean = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();
    if std < f64::EPSILON {
        return Ok(vec![0.0; scores.len()]);
    }
    Ok(scores.iter().map(|&x| (x - mean) / std).collect())
}

pub fn tanh_normalize(scores: &[f64]) -> Result<Vec<f64>, FusionError> {
    if scores.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let n = scores.len() as f64;
    let mu = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|&x| (x - mu).powi(2)).sum::<f64>() / n;
    let sigma = variance.sqrt().max(f64::EPSILON);
    Ok(scores
        .iter()
        .map(|&x| 0.5 * (1.0 + (0.01 * (x - mu) / sigma).tanh()))
        .collect())
}

pub fn double_sigmoid_normalize(scores: &[f64]) -> Result<Vec<f64>, FusionError> {
    if scores.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let n = scores.len() as f64;
    let mu = scores.iter().sum::<f64>() / n;
    Ok(scores
        .iter()
        .map(|&x| {
            if x < mu {
                0.5 * (1.0 / (1.0 + (-2.0 * (x - mu)).exp()))
            } else {
                1.0 - 0.5 * (1.0 / (1.0 + (2.0 * (x - mu)).exp()))
            }
        })
        .collect())
}

pub fn t_norm(a: f64, b: f64) -> f64 {
    a * b
}

pub fn s_norm(a: f64, b: f64) -> f64 {
    a + b - a * b
}

pub fn weighted_sum(scores: &[f64], weights: &[f64]) -> Result<f64, FusionError> {
    if scores.len() != weights.len() {
        return Err(FusionError::DimensionMismatch {
            expected: scores.len(),
            found: weights.len(),
        });
    }
    let weight_sum: f64 = weights.iter().sum();
    if weight_sum.abs() < f64::EPSILON {
        return Err(FusionError::WeightSumZero);
    }
    let dot: f64 = scores
        .iter()
        .zip(weights.iter())
        .map(|(&s, &w)| s * w)
        .sum();
    Ok(dot / weight_sum)
}

pub fn product_fusion(scores: &[f64]) -> Result<f64, FusionError> {
    if scores.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    Ok(scores.iter().product())
}

pub fn likelihood_ratio_fusion(scores: &[f64], priors: &[f64]) -> Result<f64, FusionError> {
    if scores.len() != priors.len() {
        return Err(FusionError::DimensionMismatch {
            expected: scores.len(),
            found: priors.len(),
        });
    }
    if scores.is_empty() {
        return Err(FusionError::EmptyScores);
    }
    let log_odds: f64 = scores
        .iter()
        .zip(priors.iter())
        .map(|(&s, &p)| ((s * p) / (1.0 - s + 1e-10)).ln())
        .sum();
    let posterior = 1.0 / (1.0 + (-log_odds).exp());
    Ok(posterior)
}
