// Personnel generator: returning officers, presiding officers, clerks, IT officers, etc.
// Phones use +254799xxxxxx reserved test range (distinct from voter range: +254799 shared prefix
// but starting from 900_000 suffix to avoid collisions with voter phones).

use anyhow::Result;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;

use crate::names::NamePool;
use crate::rng::{child_seed, seeded};

static ROLES: &[&str] = &[
    "RETURNING_OFFICER",
    "PRESIDING_OFFICER",
    "DEPUTY_PRESIDING_OFFICER",
    "POLLING_CLERK",
    "IT_OFFICER",
    "SECURITY_LIAISON",
    "OBSERVER_COORDINATOR",
    "LOGISTICS_OFFICER",
];

#[derive(Debug, Serialize)]
pub struct PersonnelRecord {
    pub personnel_id: String,
    pub national_id: String,
    pub first_name: String,
    pub last_name: String,
    pub role: &'static str,
    pub phone: String,
    pub county_code: u32,
    pub constituency_code: u32,
    pub station_code: Option<String>,
    pub accreditation_number: String,
    pub status: &'static str,
}

const PERSONNEL_ID_BASE: u64 = 999_500_000;
const PERSONNEL_PHONE_BASE: u64 = 799_900_000;

pub fn generate(
    seed: u64,
    count: u64,
    stations: &[crate::geo::PollingStation],
) -> Result<Vec<PersonnelRecord>> {
    assert!(!stations.is_empty());

    let pool = NamePool::load();
    let mut rng: ChaCha20Rng = seeded(child_seed(seed, "personnel"));
    let mut records = Vec::with_capacity(count as usize);

    for i in 0..count {
        let national_id = PERSONNEL_ID_BASE + i;
        let phone_suffix = PERSONNEL_PHONE_BASE + (i % 100_000);
        let phone = format!("+254{:09}", phone_suffix);

        let station = &stations[rng.gen_range(0..stations.len())];
        let role_idx = rng.gen_range(0..ROLES.len());
        let role = ROLES[role_idx];

        // Station-level roles are bound to a station; senior roles are county-wide.
        let station_code = match role {
            "PRESIDING_OFFICER" | "DEPUTY_PRESIDING_OFFICER" | "POLLING_CLERK" => {
                Some(station.code.clone())
            }
            _ => None,
        };

        records.push(PersonnelRecord {
            personnel_id: format!("PRS-{:010}", i + 1),
            national_id: national_id.to_string(),
            first_name: pool.first(&mut rng).to_owned(),
            last_name: pool.last(&mut rng).to_owned(),
            role,
            phone,
            county_code: station.county_code,
            constituency_code: station.constituency_code,
            station_code,
            accreditation_number: format!("ACC-{:08}", 100_000 + i),
            status: "ACCREDITED",
        });
    }

    Ok(records)
}

pub fn write_csv<W: std::io::Write>(records: &[PersonnelRecord], w: W) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(w);
    for r in records {
        wtr.serialize(r)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_sql_copy<W: std::io::Write>(records: &[PersonnelRecord], mut w: W) -> Result<()> {
    writeln!(
        w,
        "COPY personnel (personnel_id, national_id, first_name, last_name, role, phone, county_code, constituency_code, station_code, accreditation_number, status) FROM STDIN (FORMAT text, DELIMITER E'\\t');"
    )?;
    for r in records {
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            r.personnel_id,
            r.national_id,
            r.first_name,
            r.last_name,
            r.role,
            r.phone,
            r.county_code,
            r.constituency_code,
            r.station_code.as_deref().unwrap_or("\\N"),
            r.accreditation_number,
            r.status,
        )?;
    }
    writeln!(w, "\\.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stations() -> Vec<crate::geo::PollingStation> {
        vec![crate::geo::PollingStation {
            station_id: "PSN-0000001".into(),
            code: "47001001001".into(),
            name: "Test".into(),
            county_code: 47,
            county_name: "Nairobi".into(),
            constituency_code: 1,
            constituency_name: "Test".into(),
            ward_code: 101,
            ward_name: "Test".into(),
            registered_voters: 400,
            lat: -1.28,
            lon: 36.82,
        }]
    }

    #[test]
    fn personnel_ids_unique() {
        let p = generate(42, 30, &stations()).unwrap();
        let ids: std::collections::HashSet<_> = p.iter().map(|r| &r.personnel_id).collect();
        assert_eq!(ids.len(), 30);
    }

    #[test]
    fn phones_in_reserved_range() {
        let p = generate(42, 5, &stations()).unwrap();
        for r in &p {
            assert!(
                r.phone.starts_with("+254799"),
                "phone {} not in test range",
                r.phone
            );
        }
    }

    #[test]
    fn deterministic() {
        let a = generate(9, 10, &stations()).unwrap();
        let b = generate(9, 10, &stations()).unwrap();
        assert_eq!(a[0].national_id, b[0].national_id);
        assert_eq!(a[0].first_name, b[0].first_name);
    }
}
