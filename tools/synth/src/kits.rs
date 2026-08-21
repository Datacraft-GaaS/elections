// KIEMS kit generator.
// Kit IDs use KIT-XXXXXXX format. TPM public key is a 32-byte stub (deterministic).
// Geo-fence assignment links each kit to a polling station.

use anyhow::Result;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use serde::Serialize;

use crate::rng::{child_seed, seeded};

#[derive(Debug, Serialize)]
pub struct KiemKit {
    pub kit_id: String,
    pub serial: String,
    pub firmware_version: String,
    pub tpm_pubkey_hex: String,
    pub cert_fingerprint_hex: String,
    pub station_code: String,
    pub county_code: u32,
    pub geo_lat: f64,
    pub geo_lon: f64,
    pub geo_radius_m: u32,
    pub status: &'static str,
}

pub fn generate(
    seed: u64,
    count: u64,
    stations: &[crate::geo::PollingStation],
) -> Result<Vec<KiemKit>> {
    assert!(!stations.is_empty());

    let mut rng: ChaCha20Rng = seeded(child_seed(seed, "kits"));
    let mut kits = Vec::with_capacity(count as usize);

    for i in 0..count {
        let station = &stations[rng.gen_range(0..stations.len())];

        // Deterministic 32-byte TPM pubkey stub.
        let mut tpm_bytes = [0u8; 32];
        for b in tpm_bytes.iter_mut() {
            *b = rng.gen();
        }
        let tpm_hex: String = tpm_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        // Certificate fingerprint: 20-byte stub.
        let mut cert_bytes = [0u8; 20];
        for b in cert_bytes.iter_mut() {
            *b = rng.gen();
        }
        let cert_hex: String = cert_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let fw_major: u8 = 3;
        let fw_minor: u8 = rng.gen_range(0u8..5);
        let fw_patch: u8 = rng.gen_range(0u8..20);
        let geo_radius = rng.gen_range(150u32..500);

        kits.push(KiemKit {
            kit_id: format!("KIT-{:07}", i + 1),
            serial: format!("KE{:04}{:06}", rng.gen_range(2019u32..2024), i + 1),
            firmware_version: format!("{}.{}.{}", fw_major, fw_minor, fw_patch),
            tpm_pubkey_hex: tpm_hex,
            cert_fingerprint_hex: cert_hex,
            station_code: station.code.clone(),
            county_code: station.county_code,
            geo_lat: station.lat,
            geo_lon: station.lon,
            geo_radius_m: geo_radius,
            status: "PROVISIONED",
        });
    }

    Ok(kits)
}

pub fn write_csv<W: std::io::Write>(kits: &[KiemKit], w: W) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(w);
    for k in kits {
        wtr.serialize(k)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn write_sql_copy<W: std::io::Write>(kits: &[KiemKit], mut w: W) -> Result<()> {
    writeln!(
        w,
        "COPY kiems_kits (kit_id, serial, firmware_version, tpm_pubkey_hex, cert_fingerprint_hex, station_code, county_code, geo_lat, geo_lon, geo_radius_m, status) FROM STDIN (FORMAT text, DELIMITER E'\\t');"
    )?;
    for k in kits {
        writeln!(
            w,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.6}\t{:.6}\t{}\t{}",
            k.kit_id,
            k.serial,
            k.firmware_version,
            k.tpm_pubkey_hex,
            k.cert_fingerprint_hex,
            k.station_code,
            k.county_code,
            k.geo_lat,
            k.geo_lon,
            k.geo_radius_m,
            k.status,
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
    fn kit_ids_unique() {
        let kits = generate(42, 50, &stations()).unwrap();
        let ids: std::collections::HashSet<_> = kits.iter().map(|k| &k.kit_id).collect();
        assert_eq!(ids.len(), 50);
    }

    #[test]
    fn deterministic() {
        let a = generate(1, 10, &stations()).unwrap();
        let b = generate(1, 10, &stations()).unwrap();
        assert_eq!(a[0].kit_id, b[0].kit_id);
        assert_eq!(a[0].tpm_pubkey_hex, b[0].tpm_pubkey_hex);
    }
}
