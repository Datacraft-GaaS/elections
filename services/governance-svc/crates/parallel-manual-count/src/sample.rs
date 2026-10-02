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
use std::collections::HashSet;

pub const TOTAL_STATIONS: u32 = 46_229;
pub const SAMPLE_TARGET: u32 = 462; // ceil(46229 * 0.01)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stratum {
    pub name: String,
    pub station_count: u32,
    pub allocated: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampledStation {
    pub station_id: String,
    pub stratum: String,
    pub sequence_in_stratum: u32,
}

/// Neyman allocation under equal variance assumption simplifies to proportional allocation:
/// n_h = n * N_h / N. Returns allocations in same order as input strata.
/// Remainder votes are distributed to strata with largest fractional parts.
pub fn neyman_allocate(strata: &[Stratum], total_sample: u32) -> Vec<u32> {
    let n = total_sample as f64;
    let total: u32 = strata.iter().map(|s| s.station_count).sum();
    let n_total = total as f64;

    let mut floored: Vec<u32> = strata
        .iter()
        .map(|s| (n * s.station_count as f64 / n_total).floor() as u32)
        .collect();

    let sum_floored: u32 = floored.iter().sum();
    let remainder = total_sample.saturating_sub(sum_floored) as usize;

    // Distribute remainder to strata with largest fractional parts.
    let mut fracs: Vec<(usize, f64)> = strata
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let exact = n * s.station_count as f64 / n_total;
            (i, exact - exact.floor())
        })
        .collect();
    fracs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (i, _) in fracs.into_iter().take(remainder) {
        floored[i] += 1;
    }

    floored
}

/// Select stations from each stratum using a seeded LCG. No duplicates per stratum.
pub fn sample_stations(strata: &[Stratum], seed: u64) -> Vec<SampledStation> {
    let allocations = neyman_allocate(strata, SAMPLE_TARGET);
    let mut result = Vec::new();

    for (stratum_idx, (stratum, &alloc)) in strata.iter().zip(allocations.iter()).enumerate() {
        if alloc == 0 || stratum.station_count == 0 {
            continue;
        }
        let mut seen: HashSet<u64> = HashSet::new();
        let mut state: u64 =
            seed.wrapping_add((stratum_idx as u64).wrapping_mul(0xDEAD_BEEF_CAFE_1234));
        let mut sequence: u32 = 0;

        while seen.len() < alloc as usize {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let idx = state % stratum.station_count as u64;
            if seen.insert(idx) {
                result.push(SampledStation {
                    station_id: format!("STN-{:05}", idx + 1),
                    stratum: stratum.name.clone(),
                    sequence_in_stratum: sequence,
                });
                sequence += 1;
            }
        }
    }

    result
}

/// Default 9-stratum breakdown summing to TOTAL_STATIONS (46,229).
pub fn default_strata() -> Vec<Stratum> {
    let raw = [
        ("Nairobi", 2_645u32),
        ("Central", 3_201),
        ("Coast", 2_868),
        ("Eastern", 5_891),
        ("NorthEastern", 1_234),
        ("Nyanza", 4_502),
        ("RiftValley", 10_388),
        ("Western", 3_109),
        ("Diaspora", 12_391),
    ];
    // Allocated field is filled in during plan generation; zero here is a placeholder.
    raw.iter()
        .map(|(name, count)| Stratum {
            name: name.to_string(),
            station_count: *count,
            allocated: 0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strata_sum_matches_total() {
        let strata = default_strata();
        let total: u32 = strata.iter().map(|s| s.station_count).sum();
        assert_eq!(total, TOTAL_STATIONS);
    }
}
