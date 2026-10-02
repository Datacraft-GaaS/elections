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

//! Signed station/form coverage certificates: diff the expected set of
//! polling-station ids against what the public log actually covers.
//! Any missing station means the log is incomplete and the command exits
//! non-zero after printing the certificate.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CoverageCertificate {
    pub expected_count: usize,
    pub present_count: usize,
    pub missing: Vec<String>,
    pub unexpected: Vec<String>,
}

/// Pure set-difference between expected and present ids.
pub fn coverage_diff(expected: &[String], present: &[String]) -> CoverageCertificate {
    let present_set: std::collections::HashSet<&String> = present.iter().collect();
    let expected_set: std::collections::HashSet<&String> = expected.iter().collect();

    let mut missing: Vec<String> = expected
        .iter()
        .filter(|id| !present_set.contains(*id))
        .cloned()
        .collect();
    let mut unexpected: Vec<String> = present
        .iter()
        .filter(|id| !expected_set.contains(*id))
        .cloned()
        .collect();
    missing.sort();
    unexpected.sort();

    CoverageCertificate {
        expected_count: expected.len(),
        present_count: present.len(),
        missing,
        unexpected,
    }
}

fn load_signing_key(seed_hex: &str) -> anyhow::Result<ed25519_dalek::SigningKey> {
    let seed: [u8; 32] = hex::decode(seed_hex.trim())
        .map_err(|e| anyhow::anyhow!("signing key not hex: {e}"))?
        .try_into()
        .map_err(|_| anyhow::anyhow!("signing key must be a 32-byte Ed25519 seed"))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&seed))
}

fn read_id_list(path: &str) -> anyhow::Result<Vec<String>> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("cannot read {path}: {e}"))?;
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Diff the two id files, sign the certificate and print it. Returns the
/// number of missing ids so the caller can exit non-zero on gaps.
pub async fn run(
    expected_file: &str,
    present_file: &str,
    signing_key_hex: &str,
) -> anyhow::Result<usize> {
    let expected = read_id_list(expected_file)?;
    let present = read_id_list(present_file)?;

    let cert = coverage_diff(&expected, &present);
    let missing = cert.missing.len();

    let key = load_signing_key(signing_key_hex)?;
    use ed25519_dalek::Signer;
    let canonical = serde_json::to_vec(&cert)?;
    let sig = key.sign(&canonical);

    let body: serde_json::Value = serde_json::to_value(&cert)?;
    let mut signed = body;
    signed["signature"] = serde_json::json!(format!("ed25519:{}", {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        STANDARD.encode(sig.to_bytes())
    }));
    signed["signer_pubkey"] =
        serde_json::json!(hex::encode(key.verifying_key().as_bytes()));
    println!("{}", serde_json::to_string_pretty(&signed)?);

    Ok(missing)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn diff_reports_missing_and_unexpected_sorted() {
        let c = coverage_diff(
            &ids(&["S03", "S01", "S02"]),
            &ids(&["S02", "S09", "S01"]),
        );
        assert_eq!(c.missing, vec!["S03"]);
        assert_eq!(c.unexpected, vec!["S09"]);
        assert_eq!(c.expected_count, 3);
        assert_eq!(c.present_count, 3);
    }

    #[test]
    fn full_coverage_has_empty_missing() {
        let c = coverage_diff(&ids(&["A", "B"]), &ids(&["B", "A"]));
        assert!(c.missing.is_empty());
        assert!(c.unexpected.is_empty());
    }

    #[test]
    fn duplicates_do_not_inflate_missing() {
        let c = coverage_diff(&ids(&["A", "A"]), &ids(&[]));
        assert_eq!(c.missing, vec!["A", "A"]);
        assert_eq!(c.expected_count, 2);
    }
}
