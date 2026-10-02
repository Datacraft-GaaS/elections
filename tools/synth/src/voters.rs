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

// Voter register generator.
// National IDs start at 999_000_000 (reserved synthetic range).
// Phones use +254799xxxxxx (reserved test range, ITU E.164).

use anyhow::Result;
use chrono::{Datelike, Duration, NaiveDate};
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;

use crate::names::NamePool;
use crate::rng::{child_seed, seeded};

const NATIONAL_ID_BASE: u64 = 999_000_000;
const PHONE_BASE: u64 = 799_000_000; // +254 799 000 000

static GENDERS: [&str; 2] = ["M", "F"];

#[derive(Debug, Serialize)]
pub struct VoterRecord {
    pub voter_id: String,
    pub national_id: String,
    pub first_name: String,
    pub last_name: String,
    pub gender: &'static str,
    pub dob: String,
    pub phone: String,
    pub county_code: u32,
    pub constituency_code: u32,
    pub station_code: String,
    pub registration_date: String,
    pub status: &'static str,
}

pub fn generate(
    seed: u64,
    count: u64,
    stations: &[crate::geo::PollingStation],
) -> Result<Vec<VoterRecord>> {
    assert!(
        !stations.is_empty(),
        "stations list must not be empty for voter generation"
    );

    let pool = NamePool::load();
    let mut rng: ChaCha20Rng = seeded(child_seed(seed, "voters"));
    let mut records = Vec::with_capacity(count as usize);

    // Epoch range: voters born between 1945-01-01 and 2005-12-31 (age 18-79 at election).
    let epoch_start = NaiveDate::from_ymd_opt(1945, 1, 1).unwrap();
    let epoch_end = NaiveDate::from_ymd_opt(2005, 12, 31).unwrap();
    let span_days = (epoch_end - epoch_start).num_days() as u64;

    // Registration date range: 2021-01-01 to 2022-06-30.
    let reg_start = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
    let reg_span: u64 = 545;

    for i in 0..count {
        let national_id = NATIONAL_ID_BASE + i;
        let phone_suffix = PHONE_BASE + (i % 1_000_000);
        let phone = format!("+254{:09}", phone_suffix);

        let dob_days = rng.gen_range(0..span_days);
        let dob = epoch_start + Duration::days(dob_days as i64);

        let reg_days = rng.gen_range(0..reg_span);
        let reg_date = reg_start + Duration::days(reg_days as i64);

        let station = &stations[rng.gen_range(0..stations.len())];
        let gender_idx = rng.gen_range(0..2usize);

        records.push(VoterRecord {
            voter_id: format!("VTR-{:010}", i + 1),
            national_id: national_id.to_string(),
            first_name: pool.first(&mut rng).to_owned(),
            last_name: pool.last(&mut rng).to_owned(),
            gender: GENDERS[gender_idx],
            dob: format!("{}-{:02}-{:02}", dob.year(), dob.month(), dob.day()),
            phone,
            county_code: station.county_code,
            constituency_code: station.constituency_code,
            station_code: station.code.clone(),
            registration_date: format!(
                "{}-{:02}-{:02}",
                reg_date.year(),
                reg_date.month(),
                reg_date.day()
            ),
            status: "ACTIVE",
        });
    }

    Ok(records)
}

pub fn write_csv<W: std::io::Write>(records: &[VoterRecord], w: W) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(w);
    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_sql_copy<W: std::io::Write>(records: &[VoterRecord], mut w: W) -> Result<()> {
    writeln!(
        w,
        "COPY voters (voter_id, national_id, first_name, last_name, gender, dob, phone, county_code, constituency_code, station_code, registration_date, status) FROM STDIN (FORMAT text, DELIMITER E'\\t');"
    )?;
    for r in records {
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            r.voter_id,
            r.national_id,
            r.first_name,
            r.last_name,
            r.gender,
            r.dob,
            r.phone,
            r.county_code,
            r.constituency_code,
            r.station_code,
            r.registration_date,
            r.status,
        )?;
    }
    writeln!(w, "\\.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stations() -> Vec<crate::geo::PollingStation> {
        vec![crate::geo::PollingStation {
            station_id: "PSN-0000001".into(),
            code: "47001001001".into(),
            name: "Test Station".into(),
            county_code: 47,
            county_name: "Nairobi".into(),
            constituency_code: 1,
            constituency_name: "Test Constituency".into(),
            ward_code: 101,
            ward_name: "Test Ward".into(),
            registered_voters: 400,
            lat: -1.28,
            lon: 36.82,
        }]
    }

    #[test]
    fn voter_ids_are_unique() {
        let stations = make_stations();
        let records = generate(42, 100, &stations).unwrap();
        let ids: std::collections::HashSet<_> = records.iter().map(|r| &r.voter_id).collect();
        assert_eq!(ids.len(), 100);
    }

    #[test]
    fn national_ids_in_reserved_range() {
        let stations = make_stations();
        let records = generate(42, 10, &stations).unwrap();
        for r in &records {
            let id: u64 = r.national_id.parse().unwrap();
            assert!(id >= NATIONAL_ID_BASE, "ID {} below reserved range", id);
        }
    }

    #[test]
    fn phones_in_reserved_range() {
        let stations = make_stations();
        let records = generate(42, 10, &stations).unwrap();
        for r in &records {
            assert!(
                r.phone.starts_with("+254799"),
                "phone {} not in test range",
                r.phone
            );
        }
    }

    #[test]
    fn deterministic() {
        let stations = make_stations();
        let a = generate(5, 20, &stations).unwrap();
        let b = generate(5, 20, &stations).unwrap();
        assert_eq!(a[0].voter_id, b[0].voter_id);
        assert_eq!(a[0].national_id, b[0].national_id);
        assert_eq!(a[0].first_name, b[0].first_name);
    }
}
