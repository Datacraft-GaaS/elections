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
