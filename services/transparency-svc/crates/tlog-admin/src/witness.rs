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

//! `tlog-admin witness-cross-check` — verify all witnesses signed consistent STHs.
//!
//! Fetches the witness report from the API (recent STHs, their cosignatures,
//! and the registered witness verification keys), verifies every Ed25519
//! cosignature over `concat(tree_size_le64, root_hash_bytes)` per the witness
//! protocol (transparency-svc docs, Appendix D), and fails loudly on any
//! invalid signature or split view (two different roots claimed for the same
//! tree size).

use std::collections::{BTreeMap, HashMap};

use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
struct Report {
    sths: Vec<SthRow>,
    cosignatures: Vec<CosigRow>,
    witnesses: Vec<WitnessKeyRow>,
}

#[derive(Deserialize)]
struct SthRow {
    id: Uuid,
    tree_size: i64,
    root_hash: String,
}

#[derive(Deserialize)]
struct CosigRow {
    sth_id: Uuid,
    witness_id: String,
    signature: String,
}

#[derive(Deserialize)]
struct WitnessKeyRow {
    witness_id: String,
    public_key_hex: String,
}

/// Parse a registered witness key: 32 raw Ed25519 bytes, hex encoded.
fn parse_witness_key(public_key_hex: &str) -> Result<VerifyingKey> {
    let bytes = hex::decode(public_key_hex.trim())
        .context("witness public key is not hex")?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("witness public key must be exactly 32 bytes"))?;
    Ok(VerifyingKey::from_bytes(&arr)?)
}

/// Canonical cosignature message per the witness protocol.
fn cosign_message(tree_size: i64, root_hash_hex: &str) -> Result<Vec<u8>> {
    let root = hex::decode(root_hash_hex).context("sth root_hash is not hex")?;
    let size = u64::try_from(tree_size).context("sth tree_size is negative")?;
    let mut msg = Vec::with_capacity(8 + root.len());
    msg.extend_from_slice(&size.to_le_bytes());
    msg.extend_from_slice(&root);
    Ok(msg)
}

/// Verify one cosignature; returns false rather than erroring so one bad row
/// cannot hide the status of the rest.
fn verify_cosignature(
    key: &VerifyingKey,
    tree_size: i64,
    root_hash_hex: &str,
    signature_hex: &str,
) -> bool {
    let Ok(msg) = cosign_message(tree_size, root_hash_hex) else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(signature_hex.trim()) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(&sig_bytes) else {
        return false;
    };
    key.verify(&msg, &sig).is_ok()
}

/// Distinct roots per tree size across all reported STHs; any entry with more
/// than one root is a split view (the log was forked or witnesses were shown
/// inconsistent heads).
fn split_views(sths: &[SthRow]) -> Vec<(i64, Vec<String>)> {
    let mut by_size: BTreeMap<i64, Vec<String>> = BTreeMap::new();
    for s in sths {
        let roots = by_size.entry(s.tree_size).or_default();
        if !roots.contains(&s.root_hash) {
            roots.push(s.root_hash.clone());
        }
    }
    by_size
        .into_iter()
        .filter(|(_, roots)| roots.len() > 1)
        .collect()
}

pub async fn run(api_url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/transparency/witnesses/report?limit=200", api_url);
    let report: Report = client.get(&url).send().await?.error_for_status()?.json().await?;

    println!("Witnesses: {}", report.witnesses.len());
    println!("STHs in window: {}", report.sths.len());
    println!("Cosignatures: {}", report.cosignatures.len());

    // Split-view check across every reported STH, cosigned or not.
    let splits = split_views(&report.sths);
    if !splits.is_empty() {
        for (size, roots) in &splits {
            eprintln!("SPLIT VIEW at tree_size {}: {} distinct roots {:?}", size, roots.len(), roots);
        }
        bail!("split view detected: refusing to certify witness consistency");
    }

    let keys: HashMap<&str, VerifyingKey> = report
        .witnesses
        .iter()
        .map(|w| Ok((w.witness_id.as_str(), parse_witness_key(&w.public_key_hex)?)))
        .collect::<Result<_>>()?;

    let sth_by_id: HashMap<Uuid, &SthRow> =
        report.sths.iter().map(|s| (s.id, s)).collect();

    let mut verified = 0usize;
    let mut failures: Vec<String> = Vec::new();
    for c in &report.cosignatures {
        match (sth_by_id.get(&c.sth_id), keys.get(c.witness_id.as_str())) {
            (Some(sth), Some(key)) => {
                if verify_cosignature(key, sth.tree_size, &sth.root_hash, &c.signature) {
                    verified += 1;
                } else {
                    failures.push(format!(
                        "invalid cosignature: witness {} over sth {}",
                        c.witness_id, c.sth_id
                    ));
                }
            }
            (None, _) => failures.push(format!(
                "cosignature references unknown sth {} (witness {})",
                c.sth_id, c.witness_id
            )),
            (_, None) => failures.push(format!(
                "cosignature from unregistered witness {}",
                c.witness_id
            )),
        }
    }

    if !failures.is_empty() {
        for f in &failures {
            eprintln!("{f}");
        }
        bail!(
            "{} of {} cosignatures failed verification",
            failures.len(),
            report.cosignatures.len()
        );
    }

    println!(
        "witness-cross-check: no split views detected; {}/{} cosignatures verified",
        verified,
        report.cosignatures.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_and_sig(tree_size: i64, root_hex: &str) -> (VerifyingKey, String) {
        use ed25519_dalek::Signer;
        let sk = ed25519_dalek::SigningKey::from_bytes(&[3u8; 32]);
        let msg = cosign_message(tree_size, root_hex).unwrap();
        (sk.verifying_key(), hex::encode(sk.sign(&msg).to_bytes()))
    }

    #[test]
    fn valid_cosignature_verifies() {
        let (key, sig) = key_and_sig(42, "aabbcc");
        assert!(verify_cosignature(&key, 42, "aabbcc", &sig));
    }

    #[test]
    fn tampered_root_fails_verification() {
        let (key, sig) = key_and_sig(42, "aabbcc");
        assert!(!verify_cosignature(&key, 42, "aabbcd", &sig));
    }

    #[test]
    fn malformed_signature_is_false_not_panic() {
        let (key, _) = key_and_sig(1, "aa");
        assert!(!verify_cosignature(&key, 1, "aa", "zz"));
    }
    #[test]
    fn witness_keys_must_be_32_raw_bytes() {
        use ed25519_dalek::Signer;
        let sk = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        assert!(parse_witness_key(&hex::encode(sk.verifying_key().to_bytes())).is_ok());
        assert!(parse_witness_key(&hex::encode([7u8; 31])).is_err());
    }

    #[test]
    fn split_view_detection_flags_conflicting_roots_only() {
        let mk = |id: Uuid, size: i64, root: &str| SthRow {
            id,
            tree_size: size,
            root_hash: root.into(),
        };
        let clean = vec![
            mk(Uuid::now_v7(), 10, "aa"),
            mk(Uuid::now_v7(), 11, "bb"),
            mk(Uuid::now_v7(), 11, "bb"),
        ];
        assert!(split_views(&clean).is_empty());

        let forked = vec![
            mk(Uuid::now_v7(), 12, "cc"),
            mk(Uuid::now_v7(), 12, "dd"),
        ];
        let flagged = split_views(&forked);
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].0, 12);
        assert_eq!(flagged[0].1.len(), 2);
    }
}
