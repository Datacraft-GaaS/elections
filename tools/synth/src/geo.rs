// Geo generator: polling stations, constituencies, counties.
// Outputs PostGIS-compatible GeoJSON and SQL COPY blocks.

use anyhow::Result;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonObject, Value};
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;

use crate::rng::{child_seed, seeded};

static COUNTIES_JSON: &str = include_str!("../fixtures/counties.json");

// ---------------------------------------------------------------------------
// Fixture types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CountyDef {
    pub code: u32,
    pub name: String,
    pub lat_min: f64,
    pub lat_max: f64,
    pub lon_min: f64,
    pub lon_max: f64,
}

#[derive(Debug, Serialize)]
pub struct PollingStation {
    pub station_id: String,
    pub code: String,
    pub name: String,
    pub county_code: u32,
    pub county_name: String,
    pub constituency_code: u32,
    pub constituency_name: String,
    pub ward_code: u32,
    pub ward_name: String,
    pub registered_voters: u32,
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize)]
pub struct Constituency {
    pub constituency_code: u32,
    pub constituency_name: String,
    pub county_code: u32,
    pub county_name: String,
}

pub struct GeoData {
    pub counties: Vec<CountyDef>,
    pub constituencies: Vec<Constituency>,
    pub stations: Vec<PollingStation>,
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

pub fn generate(seed: u64, target_stations: u64) -> Result<GeoData> {
    let counties: Vec<CountyDef> = serde_json::from_str(COUNTIES_JSON)?;

    let mut rng = seeded(child_seed(seed, "geo"));

    // Build constituencies: 290 total, distributed proportionally across 47 counties.
    // County codes 1..47; distribute 290 seats roughly evenly (min 1 per county).
    let total_const: u32 = 290;
    let mut constituencies = Vec::with_capacity(290);
    let mut const_code: u32 = 1;

    // Assign constituencies to counties proportionally by lat/lon area (proxy for size).
    let areas: Vec<f64> = counties
        .iter()
        .map(|c| {
            let dlon = c.lon_max - c.lon_min;
            let dlat = c.lat_max - c.lat_min;
            (dlon * dlat).abs().max(0.01)
        })
        .collect();
    let total_area: f64 = areas.iter().sum();

    let mut county_const_counts: Vec<u32> = areas
        .iter()
        .map(|a| ((a / total_area * total_const as f64).round() as u32).max(1))
        .collect();

    // Adjust to exactly 290.
    let mut assigned: u32 = county_const_counts.iter().sum();
    let mut idx = 0usize;
    let n_counties = county_const_counts.len();
    while assigned < total_const {
        county_const_counts[idx % n_counties] += 1;
        assigned += 1;
        idx += 1;
    }
    while assigned > total_const {
        let k = idx % n_counties;
        if county_const_counts[k] > 1 {
            county_const_counts[k] -= 1;
            assigned -= 1;
        }
        idx += 1;
    }

    for (county_idx, county) in counties.iter().enumerate() {
        let n = county_const_counts[county_idx];
        for j in 0..n {
            constituencies.push(Constituency {
                constituency_code: const_code,
                constituency_name: format!("{} Constituency {}", county.name, j + 1),
                county_code: county.code,
                county_name: county.name.clone(),
            });
            const_code += 1;
        }
    }
    assert_eq!(constituencies.len(), 290);

    // Distribute target_stations across constituencies.
    let stations_per_const = ((target_stations as f64) / 290.0).ceil() as u64;
    let mut stations = Vec::with_capacity(target_stations as usize);
    let mut station_seq: u64 = 1;

    'outer: for const_entry in &constituencies {
        let county = counties
            .iter()
            .find(|c| c.code == const_entry.county_code)
            .unwrap();

        let ward_count = rng.gen_range(3u32..8);
        let stations_this_const = stations_per_const;

        for ward_idx in 0..ward_count {
            let ward_code = const_entry.constituency_code * 100 + ward_idx + 1;
            let ward_name = format!("{} Ward {}", const_entry.constituency_name, ward_idx + 1);
            let stations_this_ward = (stations_this_const / ward_count as u64).max(1);

            for _ in 0..stations_this_ward {
                if station_seq > target_stations {
                    break 'outer;
                }
                let lat = rng.gen_range(county.lat_min..county.lat_max);
                let lon = rng.gen_range(county.lon_min..county.lon_max);
                let reg = rng.gen_range(200u32..700);

                stations.push(PollingStation {
                    station_id: format!("PSN-{:07}", station_seq),
                    code: format!(
                        "{:02}{:03}{:02}{:04}",
                        county.code,
                        const_entry.constituency_code,
                        ward_idx + 1,
                        station_seq
                    ),
                    name: format!("Station {} Primary School", station_seq),
                    county_code: county.code,
                    county_name: county.name.clone(),
                    constituency_code: const_entry.constituency_code,
                    constituency_name: const_entry.constituency_name.clone(),
                    ward_code,
                    ward_name: ward_name.clone(),
                    registered_voters: reg,
                    lat,
                    lon,
                });
                station_seq += 1;
            }
        }
    }

    Ok(GeoData {
        counties,
        constituencies,
        stations,
    })
}

// ---------------------------------------------------------------------------
// GeoJSON output
// ---------------------------------------------------------------------------

pub fn to_geojson(stations: &[PollingStation]) -> String {
    let features: Vec<Feature> = stations
        .iter()
        .map(|s| {
            let geom = Geometry::new(Value::Point(vec![s.lon, s.lat]));
            let mut props = JsonObject::new();
            props.insert("station_id".into(), json!(s.station_id));
            props.insert("code".into(), json!(s.code));
            props.insert("name".into(), json!(s.name));
            props.insert("county_code".into(), json!(s.county_code));
            props.insert("county_name".into(), json!(s.county_name));
            props.insert("constituency_code".into(), json!(s.constituency_code));
            props.insert("registered_voters".into(), json!(s.registered_voters));
            Feature {
                bbox: None,
                geometry: Some(geom),
                id: None,
                properties: Some(props),
                foreign_members: None,
            }
        })
        .collect();

    let fc = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };
    GeoJson::FeatureCollection(fc).to_string()
}

// ---------------------------------------------------------------------------
// SQL COPY output
// ---------------------------------------------------------------------------

pub fn write_sql_copy<W: Write>(
    stations: &[PollingStation],
    constituencies: &[Constituency],
    mut w: W,
) -> Result<()> {
    writeln!(w, "-- constituencies")?;
    writeln!(
        w,
        "COPY constituencies (constituency_code, constituency_name, county_code, county_name) FROM STDIN (FORMAT text, DELIMITER E'\\t');"
    )?;
    for c in constituencies {
        writeln!(
            w,
            "{}\t{}\t{}\t{}",
            c.constituency_code, c.constituency_name, c.county_code, c.county_name
        )?;
    }
    writeln!(w, "\\.")?;
    writeln!(w)?;

    writeln!(w, "-- polling_stations")?;
    writeln!(
        w,
        "COPY polling_stations (station_id, code, name, county_code, county_name, constituency_code, constituency_name, ward_code, ward_name, registered_voters, lat, lon) FROM STDIN (FORMAT text, DELIMITER E'\\t');"
    )?;
    for s in stations {
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.6}\t{:.6}",
            s.station_id,
            s.code,
            s.name,
            s.county_code,
            s.county_name,
            s.constituency_code,
            s.constituency_name,
            s.ward_code,
            s.ward_name,
            s.registered_voters,
            s.lat,
            s.lon,
        )?;
    }
    writeln!(w, "\\.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_constituency_count() {
        let data = generate(42, 100).unwrap();
        assert_eq!(data.constituencies.len(), 290);
    }

    #[test]
    fn station_count_within_bounds() {
        let data = generate(42, 500).unwrap();
        assert!(!data.stations.is_empty());
        assert!(data.stations.len() <= 600);
    }

    #[test]
    fn deterministic_across_calls() {
        let a = generate(7, 50).unwrap();
        let b = generate(7, 50).unwrap();
        assert_eq!(a.stations.len(), b.stations.len());
        if !a.stations.is_empty() && !b.stations.is_empty() {
            let fa = &a.stations[0];
            let fb = &b.stations[0];
            assert_eq!(fa.station_id, fb.station_id);
            assert!((fa.lat - fb.lat).abs() < f64::EPSILON);
        }
    }
}
