// Determinism verifier.
//
// Runs the full default generation pipeline, hashes the concatenated CSV output,
// and compares to a stored checkpoint. On first run (no checkpoint) it writes
// the checkpoint and exits 0. Subsequent runs compare and exit 1 on mismatch.

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

static CHECKPOINT_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fixtures/checksums/seed42-default.sha256"
);

/// Hash of CSV bytes for all generators under default profile, seed 42.
pub fn compute_hash(
    seed: u64,
    profile: &crate::ScaleProfile,
    stations: &[crate::geo::PollingStation],
) -> Result<String> {
    let mut hasher = Sha256::new();

    // Hash voters CSV.
    {
        let records = crate::voters::generate(seed, profile.voters, stations)?;
        let mut buf = Vec::new();
        crate::voters::write_csv(&records, &mut buf)?;
        hasher.update(&buf);
    }

    // Hash kits CSV.
    {
        let kits = crate::kits::generate(seed, profile.kits, stations)?;
        let mut buf = Vec::new();
        crate::kits::write_csv(&kits, &mut buf)?;
        hasher.update(&buf);
    }

    // Hash personnel CSV.
    {
        let p = crate::personnel::generate(seed, profile.personnel, stations)?;
        let mut buf = Vec::new();
        crate::personnel::write_csv(&p, &mut buf)?;
        hasher.update(&buf);
    }

    // Hash embassies CSV.
    {
        let e = crate::embassies::generate(profile.embassies)?;
        let mut buf = Vec::new();
        crate::embassies::write_csv(&e, &mut buf)?;
        hasher.update(&buf);
    }

    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Load the stored checkpoint hash, if it exists.
pub fn load_checkpoint() -> Option<String> {
    let path = Path::new(CHECKPOINT_PATH);
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
}

/// Write a checkpoint hash to the fixture file.
pub fn write_checkpoint(hash: &str) -> Result<()> {
    let path = Path::new(CHECKPOINT_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{}\n", hash))?;
    Ok(())
}

/// Run verify for a given seed and profile.
/// Returns Ok(()) on match (or first-run checkpoint write), Err on mismatch.
pub fn run(seed: u64, profile: &crate::ScaleProfile) -> Result<()> {
    // Build geo first (stations needed by all generators).
    let geo_data = crate::geo::generate(seed, profile.stations)?;
    let stations = &geo_data.stations;

    eprintln!("eel-synth verify: computing hash for seed={} profile stations={} voters={} kits={} personnel={} embassies={}",
        seed, profile.stations, profile.voters, profile.kits, profile.personnel, profile.embassies);

    let computed = compute_hash(seed, profile, stations)?;
    eprintln!("eel-synth verify: computed sha256={}", computed);

    match load_checkpoint() {
        None => {
            eprintln!("eel-synth verify: no checkpoint found, writing first-run checkpoint.");
            write_checkpoint(&computed)?;
            eprintln!(
                "eel-synth verify: checkpoint written to {}",
                CHECKPOINT_PATH
            );
            eprintln!("eel-synth verify: OK (first run)");
            Ok(())
        }
        Some(stored) => {
            if stored == computed {
                eprintln!("eel-synth verify: OK (hash matches stored checkpoint)");
                Ok(())
            } else {
                anyhow::bail!(
                    "Determinism check FAILED.\n  stored:   {}\n  computed: {}\nSame seed must produce identical output. Investigate non-determinism.",
                    stored,
                    computed
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScaleProfile;

    #[test]
    fn hash_is_deterministic() {
        let profile = ScaleProfile::SMOKE;
        let geo = crate::geo::generate(42, profile.stations).unwrap();
        let h1 = compute_hash(42, &profile, &geo.stations).unwrap();
        let h2 = compute_hash(42, &profile, &geo.stations).unwrap();
        assert_eq!(
            h1, h2,
            "compute_hash must return identical results for same inputs"
        );
    }

    #[test]
    fn different_seeds_produce_different_hashes() {
        let profile = ScaleProfile::SMOKE;
        let g1 = crate::geo::generate(1, profile.stations).unwrap();
        let g2 = crate::geo::generate(2, profile.stations).unwrap();
        let h1 = compute_hash(1, &profile, &g1.stations).unwrap();
        let h2 = compute_hash(2, &profile, &g2.stations).unwrap();
        assert_ne!(h1, h2);
    }
}
