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

use crate::sample::{
    default_strata, neyman_allocate, sample_stations, SampledStation, Stratum, SAMPLE_TARGET,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplePlan {
    pub plan_id: String,
    pub created_at: String,
    pub election_day: String,
    pub seed: u64,
    pub strata: Vec<Stratum>,
    pub sampled_stations: Vec<SampledStation>,
    pub total_sampled: usize,
}

impl SamplePlan {
    pub fn generate(election_day: &str, seed: u64) -> Self {
        let mut strata = default_strata();
        let allocations = neyman_allocate(&strata, SAMPLE_TARGET);
        for (s, alloc) in strata.iter_mut().zip(allocations.iter()) {
            s.allocated = *alloc;
        }

        let sampled_stations = sample_stations(&strata, seed);
        let total_sampled = sampled_stations.len();

        SamplePlan {
            plan_id: Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            election_day: election_day.to_string(),
            seed,
            strata,
            sampled_stations,
            total_sampled,
        }
    }

    pub fn station_ids(&self) -> Vec<String> {
        self.sampled_stations
            .iter()
            .map(|s| s.station_id.clone())
            .collect()
    }

    pub fn contains_station(&self, station_id: &str) -> bool {
        self.sampled_stations
            .iter()
            .any(|s| s.station_id == station_id)
    }
}
