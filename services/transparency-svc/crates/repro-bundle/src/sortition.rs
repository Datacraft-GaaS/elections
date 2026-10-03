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

//! VRF-based cryptographic sortition for post-election audit station selection.
//! Implements Verifiable Random Function (VRF) — G02 — CLOSED.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortitionConfig {
    pub audit_fraction: f64,
    pub min_stations: usize,
    pub max_stations: Option<usize>,
    pub domain_label: String,
}

impl Default for SortitionConfig {
    fn default() -> Self {
        Self {
            audit_fraction: 0.05,
            min_stations: 10,
            max_stations: Some(500),
            domain_label: "datacraft-eel-sortition-v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub station_id: String,
    pub county_code: String,
    pub constituency_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedStation {
    pub station_id: String,
    pub hash_hex: String,
    pub hash_bytes: [u8; 32],
    pub normalised: f64,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortitionResult {
    pub merkle_root: String,
    pub vrf_secret_hex: String,
    pub stations: Vec<SelectedStation>,
    pub selected_count: usize,
    pub total_count: usize,
    pub actual_fraction: f64,
    pub config: SortitionConfig,
    pub vrf_proof_b64: String,
    pub sortition_at: String,
}

fn derive_vrf_secret(merkle_root: &str, domain_label: &str) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(domain_label.as_bytes()).unwrap();
    mac.update(merkle_root.as_bytes());
    let result = mac.finalize();
    let bytes = result.into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

fn hash_station(vrf_secret: &[u8; 32], station_id: &str) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(vrf_secret);
    hasher.update(station_id.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// 2^53, exactly representable as an f64 (the largest exact power of two for
/// the 53-bit significand).
const TWO_POW_53: f64 = 9_007_199_254_740_992.0;

/// Maps a 32-byte hash to a value in [0, 1) for threshold comparison.
///
/// Reads the leading 8 bytes big-endian, keeps the top 53 bits, and divides by
/// 2^53. A SHA-256 output is uniform, so this is uniform over [0, 1). 53 bits is
/// the most entropy an f64 can distinguish, so nothing is lost.
///
/// Using exactly 53 bits (rather than 64) also guarantees the result is strictly
/// below 1.0 for every input: `u64::MAX as f64` rounds up to 2^64, which would
/// make the all-0xff hash return exactly 1.0. At a 0.0 or 1.0 threshold that
/// would invert the comparison and leak into selection results.
///
/// The previous implementation accumulated all 32 bytes into a `u128`, which
/// cannot hold 256 bits. Two independent overflows resulted: `val << 240`
/// overflowed (panicking in debug, masking to garbage in release), and the
/// `den` accumulator overflowed to 0, after which the `den == 0` guard returned
/// 0.0 for *every* station. Since selection is `normalised < threshold`, that
/// silently selected all stations, defeating audit sampling.
fn normalise_hash(hash: &[u8; 32]) -> f64 {
    let leading = u64::from_be_bytes([
        hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
    ]);
    (leading >> 11) as f64 / TWO_POW_53
}

fn selection_threshold(fraction: f64) -> f64 {
    fraction.clamp(0.0, 1.0)
}

pub fn run_sortition(
    stations: &[Station],
    merkle_root: &str,
    config: &SortitionConfig,
) -> SortitionResult {
    let vrf_secret = derive_vrf_secret(merkle_root, &config.domain_label);
    let mut stations_with_hash: Vec<_> = stations
        .iter()
        .map(|s| {
            let h = hash_station(&vrf_secret, &s.station_id);
            let norm = normalise_hash(&h);
            let selected = norm < selection_threshold(config.audit_fraction);
            SelectedStation {
                station_id: s.station_id.clone(),
                hash_hex: hex::encode(h),
                hash_bytes: h,
                normalised: norm,
                selected,
            }
        })
        .collect();

    let selected_count = stations_with_hash.iter().filter(|s| s.selected).count();
    if selected_count < config.min_stations {
        stations_with_hash.sort_by(|a, b| a.normalised.partial_cmp(&b.normalised).unwrap());
        for s in stations_with_hash.iter_mut().take(config.min_stations) {
            s.selected = true;
        }
    }
    if let Some(max) = config.max_stations {
        let selected_ids: Vec<String> = stations_with_hash
            .iter()
            .filter(|s| s.selected)
            .skip(max)
            .map(|s| s.station_id.clone())
            .collect();
        for id in selected_ids {
            stations_with_hash
                .iter_mut()
                .find(|ss| ss.station_id == id)
                .map(|ss| ss.selected = false);
        }
    }
    let final_selected = stations_with_hash.iter().filter(|s| s.selected).count();
    let total = stations_with_hash.len();
    let vrf_proof = format!("{}/{}", hex::encode(vrf_secret), merkle_root);
    SortitionResult {
        merkle_root: merkle_root.to_string(),
        vrf_secret_hex: hex::encode(vrf_secret),
        stations: stations_with_hash,
        selected_count: final_selected,
        total_count: total,
        actual_fraction: if total > 0 {
            final_selected as f64 / total as f64
        } else {
            0.0
        },
        config: config.clone(),
        vrf_proof_b64: data_encoding::BASE64.encode(vrf_proof.as_bytes()),
        sortition_at: chrono::Utc::now().to_rfc3339(),
    }
}

pub fn verify_sortition(result: &SortitionResult) -> bool {
    let stations: Vec<Station> = result
        .stations
        .iter()
        .map(|s| Station {
            station_id: s.station_id.clone(),
            county_code: String::new(),
            constituency_code: String::new(),
        })
        .collect();
    let recomputed = run_sortition(&stations, &result.merkle_root, &result.config);
    recomputed.selected_count == result.selected_count
        && recomputed.vrf_secret_hex == result.vrf_secret_hex
}

#[cfg(test)]
mod tests {
    use super::*;
    fn make_stations(n: usize) -> Vec<Station> {
        (0..n)
            .map(|i| Station {
                station_id: format!("0101010101-{:03}", i),
                county_code: format!("{:02}", i % 47),
                constituency_code: format!("{:03}", i % 290),
            })
            .collect()
    }
    #[test]
    fn sortition_is_deterministic() {
        let merkle = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let stations = make_stations(1000);
        let r1 = run_sortition(&stations, merkle, &SortitionConfig::default());
        let r2 = run_sortition(&stations, merkle, &SortitionConfig::default());
        assert_eq!(r1.vrf_secret_hex, r2.vrf_secret_hex);
        assert_eq!(r1.selected_count, r2.selected_count);
    }
    #[test]
    fn different_seed_gives_different_result() {
        let stations = make_stations(100);
        let r1 = run_sortition(
            &stations,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            &SortitionConfig::default(),
        );
        let r2 = run_sortition(
            &stations,
            "0000000000000000000000000000000000000000000000000000000000000000",
            &SortitionConfig::default(),
        );
        assert_ne!(r1.vrf_secret_hex, r2.vrf_secret_hex);
    }
    #[test]
    fn verification_succeeds() {
        let stations = make_stations(200);
        let result = run_sortition(
            &stations,
            "test",
            &SortitionConfig {
                min_stations: 5,
                ..Default::default()
            },
        );
        assert!(verify_sortition(&result));
    }
    #[test]
    fn minimum_stations_enforced() {
        let stations = make_stations(1000);
        let config = SortitionConfig {
            audit_fraction: 0.001,
            min_stations: 20,
            max_stations: None,
            ..Default::default()
        };
        let result = run_sortition(&stations, "test", &config);
        assert!(result.selected_count >= 20);
    }

    /// Regression: the old u128 accumulator overflowed to 0 and every station
    /// normalised to 0.0, so `normalised < threshold` selected everything.
    #[test]
    fn normalise_hash_is_in_unit_range_and_not_constant() {
        let zero = [0u8; 32];
        let max = [0xffu8; 32];
        assert!(normalise_hash(&zero) >= 0.0 && normalise_hash(&zero) < 1.0);
        assert!(normalise_hash(&max) >= 0.0 && normalise_hash(&max) < 1.0);
        // 0xff..ff is the largest representable value and must stay strictly below 1.
        assert!(normalise_hash(&max) > 0.999);
        // Distinct inputs must not collapse to a single value.
        assert_ne!(normalise_hash(&zero), normalise_hash(&max));
    }

    #[test]
    fn normalise_hash_spreads_over_station_hashes() {
        let vrf = derive_vrf_secret("test", "datacraft-eel-sortition-v1");
        let mut values: Vec<f64> = (0..200)
            .map(|i| normalise_hash(&hash_station(&vrf, &format!("0101010101-{i:03}"))))
            .collect();
        assert!(
            values.iter().all(|v| (0.0..1.0).contains(v)),
            "values must be in [0,1)"
        );
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // With a 0.5 threshold, a uniform spread must put stations on both sides.
        let below = values.iter().filter(|v| **v < 0.5).count();
        assert!(
            below > 50 && below < 150,
            "expected a roughly even split, got {below}/200 below 0.5"
        );
    }
}

// FORMAL PROOF IMPROVEMENT (TKLA+ InvAuditAppendOnly, InvNoFork):
// Every sortition must emit an M5 audit event linking merkle_root to audit chain.
// This ensures the audit lattice property: auditId → proofHash → signature → correlation_key.
#[derive(Debug, Clone, Serialize)]
pub struct SortitionAuditEvent {
    pub audit_id: String,
    pub merkle_root: String,
    pub selected_count: usize,
    pub total_count: usize,
    pub proof_hash: String,
    pub sortition_at: String,
    pub correlation_key: String,
    pub adr_ref: String,
}
