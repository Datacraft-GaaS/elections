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

//! Performance monitoring runner: FMR, FNMR, rank-N accuracy.

use tracing::info;

use crate::domain::HoldoutPair;
use crate::error::PerfMonitorError;

#[derive(Debug, Clone)]
pub struct PmrResult {
    pub fmr: f64,
    pub fnmr: f64,
    pub rank1_accuracy: f64,
    pub rank5_accuracy: f64,
    pub n_genuine: usize,
    pub n_impostor: usize,
}

pub fn compute_fmr(pairs: &[HoldoutPair], threshold: f64) -> Result<f64, PerfMonitorError> {
    if !(0.0..1.0).contains(&threshold) {
        return Err(PerfMonitorError::InvalidThreshold(threshold));
    }
    let impostors: Vec<_> = pairs.iter().filter(|p| !p.genuine).collect();
    if impostors.is_empty() {
        return Ok(0.0);
    }
    let fa = impostors.iter().filter(|p| p.score >= threshold).count();
    Ok(fa as f64 / impostors.len() as f64)
}

pub fn compute_fnmr(pairs: &[HoldoutPair], threshold: f64) -> Result<f64, PerfMonitorError> {
    if !(0.0..1.0).contains(&threshold) {
        return Err(PerfMonitorError::InvalidThreshold(threshold));
    }
    let genuines: Vec<_> = pairs.iter().filter(|p| p.genuine).collect();
    if genuines.is_empty() {
        return Ok(0.0);
    }
    let fr = genuines.iter().filter(|p| p.score < threshold).count();
    Ok(fr as f64 / genuines.len() as f64)
}

pub fn compute_rank_n_accuracy(pairs: &[HoldoutPair], n: usize) -> f64 {
    let ident: Vec<_> = pairs
        .iter()
        .filter(|p| p.genuine && p.rank.is_some())
        .collect();
    if ident.is_empty() {
        return 0.0;
    }
    let hits = ident
        .iter()
        .filter(|p| p.rank.map(|r| r <= n).unwrap_or(false))
        .count();
    hits as f64 / ident.len() as f64
}

pub fn run_pmr(pairs: &[HoldoutPair], threshold: f64) -> Result<PmrResult, PerfMonitorError> {
    const MIN_PAIRS: usize = 50;
    if pairs.len() < MIN_PAIRS {
        return Err(PerfMonitorError::InsufficientData {
            min: MIN_PAIRS,
            got: pairs.len(),
        });
    }
    let fmr = compute_fmr(pairs, threshold)?;
    let fnmr = compute_fnmr(pairs, threshold)?;
    let r1 = compute_rank_n_accuracy(pairs, 1);
    let r5 = compute_rank_n_accuracy(pairs, 5);
    let n_genuine = pairs.iter().filter(|p| p.genuine).count();
    let n_impostor = pairs.len() - n_genuine;
    info!(fmr, fnmr, rank1 = r1, rank5 = r5, "PMR run complete");
    Ok(PmrResult {
        fmr,
        fnmr,
        rank1_accuracy: r1,
        rank5_accuracy: r5,
        n_genuine,
        n_impostor,
    })
}
