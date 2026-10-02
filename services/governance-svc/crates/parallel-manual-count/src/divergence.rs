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

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ingest::{CandidateCount, ManualCountSubmission};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTally {
    pub station_id: String,
    pub candidates: Vec<CandidateCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateDivergence {
    pub candidate_id: String,
    pub manual_count: u64,
    pub platform_count: u64,
    pub delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationDivergence {
    pub station_id: String,
    pub candidate_divergences: Vec<CandidateDivergence>,
    pub max_abs_divergence: i64,
    pub relative_divergence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceReport {
    pub report_id: String,
    pub generated_at: String,
    pub total_stations_sampled: usize,
    pub stations_with_divergence: usize,
    pub mean_abs_divergence: f64,
    pub max_divergence_station: Option<String>,
    pub station_reports: Vec<StationDivergence>,
}

pub fn compute_divergence(
    submissions: &[ManualCountSubmission],
    tallies: &[PlatformTally],
) -> DivergenceReport {
    let tally_map: std::collections::HashMap<&str, &PlatformTally> =
        tallies.iter().map(|t| (t.station_id.as_str(), t)).collect();

    let mut station_reports: Vec<StationDivergence> = Vec::new();
    let mut all_abs_deltas: Vec<i64> = Vec::new();

    for sub in submissions {
        let Some(tally) = tally_map.get(sub.station_id.as_str()) else {
            continue;
        };

        let platform_map: std::collections::HashMap<&str, u64> = tally
            .candidates
            .iter()
            .map(|c| (c.candidate_id.as_str(), c.manual_count))
            .collect();

        let mut candidate_divergences: Vec<CandidateDivergence> = Vec::new();
        let mut total_platform: u64 = 0;

        for cc in &sub.candidates {
            let platform_count = *platform_map.get(cc.candidate_id.as_str()).unwrap_or(&0);
            let delta = cc.manual_count as i64 - platform_count as i64;
            all_abs_deltas.push(delta.abs());
            total_platform += platform_count;
            candidate_divergences.push(CandidateDivergence {
                candidate_id: cc.candidate_id.clone(),
                manual_count: cc.manual_count,
                platform_count,
                delta,
            });
        }

        let max_abs_divergence = candidate_divergences
            .iter()
            .map(|d| d.delta.abs())
            .max()
            .unwrap_or(0);

        let relative_divergence = if total_platform > 0 {
            max_abs_divergence as f64 / total_platform as f64
        } else {
            0.0
        };

        station_reports.push(StationDivergence {
            station_id: sub.station_id.clone(),
            candidate_divergences,
            max_abs_divergence,
            relative_divergence,
        });
    }

    let stations_with_divergence = station_reports
        .iter()
        .filter(|r| r.max_abs_divergence > 0)
        .count();

    let mean_abs_divergence = if all_abs_deltas.is_empty() {
        0.0
    } else {
        all_abs_deltas.iter().sum::<i64>() as f64 / all_abs_deltas.len() as f64
    };

    let max_divergence_station = station_reports
        .iter()
        .max_by_key(|r| r.max_abs_divergence)
        .filter(|r| r.max_abs_divergence > 0)
        .map(|r| r.station_id.clone());

    DivergenceReport {
        report_id: Uuid::new_v4().to_string(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        total_stations_sampled: submissions.len(),
        stations_with_divergence,
        mean_abs_divergence,
        max_divergence_station,
        station_reports,
    }
}
