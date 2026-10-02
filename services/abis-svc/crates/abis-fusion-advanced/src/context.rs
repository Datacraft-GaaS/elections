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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchCandidate {
    pub id: String,
    pub station: Location,
    pub timestamp_utc: i64,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct SpatiotemporalFilter {
    pub max_distance_km: f64,
    pub max_time_diff_seconds: i64,
}

impl SpatiotemporalFilter {
    pub fn new(max_distance_km: f64, max_time_diff_seconds: i64) -> Self {
        Self {
            max_distance_km,
            max_time_diff_seconds,
        }
    }

    pub fn filter<'a>(
        &self,
        probe_station: &Location,
        probe_time: i64,
        candidates: &'a [MatchCandidate],
    ) -> Vec<&'a MatchCandidate> {
        candidates
            .iter()
            .filter(|c| {
                let dist = haversine(probe_station, &c.station);
                let time_diff = (probe_time - c.timestamp_utc).abs();
                dist <= self.max_distance_km && time_diff <= self.max_time_diff_seconds
            })
            .collect()
    }
}

fn haversine(a: &Location, b: &Location) -> f64 {
    const R: f64 = 6371.0;
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let dlat = (b.lat - a.lat).to_radians();
    let dlon = (b.lon - a.lon).to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * R * h.sqrt().asin()
}
