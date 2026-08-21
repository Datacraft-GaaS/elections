// Embassy (diaspora voting location) generator.
// Returns the fixture list, optionally truncated to --count.
// Embassy data is Class C (public): Kenyan diplomatic missions are publicly listed.

use anyhow::Result;
use serde::{Deserialize, Serialize};

static EMBASSIES_JSON: &str = include_str!("../fixtures/embassies.json");

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Embassy {
    pub code: String,
    pub city: String,
    pub country: String,
    pub country_code: String,
    pub lat: f64,
    pub lon: f64,
}

pub fn load_all() -> Result<Vec<Embassy>> {
    let embassies: Vec<Embassy> = serde_json::from_str(EMBASSIES_JSON)?;
    Ok(embassies)
}

pub fn generate(count: u64) -> Result<Vec<Embassy>> {
    let all = load_all()?;
    let take = (count as usize).min(all.len());
    Ok(all.into_iter().take(take).collect())
}

pub fn write_csv<W: std::io::Write>(embassies: &[Embassy], w: W) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(w);
    for e in embassies {
        wtr.serialize(e)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_sql_copy<W: std::io::Write>(embassies: &[Embassy], mut w: W) -> Result<()> {
    writeln!(
        w,
        "COPY diaspora_embassies (code, city, country, country_code, lat, lon) FROM STDIN (FORMAT text, DELIMITER E'\\t');"
    )?;
    for e in embassies {
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{:.4}\t{:.4}",
            e.code, e.city, e.country, e.country_code, e.lat, e.lon,
        )?;
    }
    writeln!(w, "\\.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_has_entries() {
        let all = load_all().unwrap();
        assert!(!all.is_empty());
    }

    #[test]
    fn generate_truncates_to_count() {
        let all = load_all().unwrap();
        let n = (all.len() / 2).max(1) as u64;
        let result = generate(n).unwrap();
        assert!(result.len() <= n as usize);
    }

    #[test]
    fn generate_full_returns_all() {
        let all = load_all().unwrap();
        let result = generate(1000).unwrap();
        assert_eq!(result.len(), all.len());
    }
}
