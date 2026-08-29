//! Register-extract bundle anchoring (wave-2 N2).
//!
//! Prevents voter-register substitution on kits: each register bundle's
//! Merkle root is anchored in the transparency log before poll-open; the kit
//! caches `{bundle_id, merkle_root, log_anchor}` in its config and verifies
//! OFFLINE during the day — no runtime connectivity required.
//!
//! Offline model: roots are computed once over the distributed bundle
//! entries; the config response below is served from an env-provided
//! registry cached at startup (`EEL_REGISTER_BUNDLES`), so it remains
//! answerable even when upstream services are unreachable (views are flagged
//! `from_cache` rather than hidden).
#![warn(missing_docs)]

use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// One register bundle as provisioned to kits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BundleView {
    pub bundle_id: String,
    /// BLAKE3 Merkle root over the bundle's entry hashes (hex).
    pub merkle_root_hex: String,
    /// Transparency-log anchor id covering this root (empty until anchored).
    pub log_anchor: String,
    /// True when this view was served from the local cache (offline-safe).
    pub from_cache: bool,
}

/// Hash one bundle entry (canonical bytes of a register record).
pub fn entry_hash(entry_bytes: &[u8]) -> [u8; 32] {
    *Hasher::new().update(entry_bytes).finalize().as_bytes()
}

/// Merkle root over bundle entries (BLAKE3 pairwise, odd node promoted),
/// matching the promotion style used across the platform.
pub fn build_root(entry_hashes: &[[u8; 32]]) -> [u8; 32] {
    if entry_hashes.is_empty() {
        return *blake3::hash(b"eel-empty-register-bundle").as_bytes();
    }
    let mut level: Vec<[u8; 32]> = entry_hashes.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        let mut i = 0;
        while i < level.len() {
            if i + 1 < level.len() {
                let mut h = Hasher::new();
                h.update(&level[i]);
                h.update(&level[i + 1]);
                next.push(*h.finalize().as_bytes());
            } else {
                next.push(level[i]);
            }
            i += 2;
        }
        level = next;
    }
    level[0]
}

/// Convenience: root directly over raw entry bytes.
pub fn root_over_entries(entries: &[&[u8]]) -> String {
    let hashes: Vec<[u8; 32]> = entries.iter().map(|e| entry_hash(e)).collect();
    hex::encode(build_root(&hashes))
}

/// Kit-side OFFLINE verification: the kit holds its cached view from poll-open
/// config and compares against the locally-trusted registry distributed with
/// that same config. No network involved; a mismatch means the on-device
/// register file does not match what was provisioned — refuse poll-open.
pub fn verify_kit_view(
    cached: &BundleView,
    trusted_roots: &[(String, String)], // bundle_id -> root_hex
) -> Result<(), RegisterAnchorError> {
    let (_, expected) = trusted_roots
        .iter()
        .find(|(id, _)| id == &cached.bundle_id)
        .ok_or_else(|| RegisterAnchorError::UnknownBundle(cached.bundle_id.clone()))?;
    if cached.merkle_root_hex != *expected {
        return Err(RegisterAnchorError::RootMismatch {
            bundle_id: cached.bundle_id.clone(),
            expected: expected.clone(),
            actual: cached.merkle_root_hex.clone(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RegisterAnchorError {
    #[error("bundle {0} has no anchored root in the trusted registry")]
    UnknownBundle(String),
    #[error("bundle {bundle_id} root mismatch: expected {expected}, got {actual}")]
    RootMismatch {
        bundle_id: String,
        expected: String,
        actual: String,
    },
}

/// Parse `EEL_REGISTER_BUNDLES` ("bundle_id:root_hex" comma list).
pub fn parse_registry(spec: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    for entry in spec.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        let parts: Vec<&str> = entry.split(':').collect();
        if parts.len() != 2 || parts[1].len() != 64 {
            return Err(format!("bad EEL_REGISTER_BUNDLES entry {entry:?}"));
        }
        out.push((parts[0].to_string(), parts[1].to_string()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_order_sensitive_and_stable() {
        let a = root_over_entries(&[b"voter-1" as &[u8], b"voter-2"]);
        let b = root_over_entries(&[b"voter-2" as &[u8], b"voter-1"]);
        assert_ne!(a, b, "register order matters");
        assert_eq!(a, root_over_entries(&[b"voter-1" as &[u8], b"voter-2"]));
    }

    #[test]
    fn single_entry_root_is_entry_hash() {
        let e = entry_hash(b"only");
        assert_eq!(build_root(&[e]), e);
    }

    #[test]
    fn empty_bundle_has_defined_root() {
        assert_eq!(build_root(&[]), build_root(&[]));
    }

    #[test]
    fn kit_verification_accepts_match_and_rejects_substitution() {
        let root = root_over_entries(&[b"voter-1" as &[u8], b"voter-2", b"voter-3"]);
        let trusted = vec![("REG-2027-A".to_string(), root.clone())];

        // Honest kit cache matches.
        let good = BundleView {
            bundle_id: "REG-2027-A".into(),
            merkle_root_hex: root.clone(),
            log_anchor: "leaf-abc".into(),
            from_cache: true,
        };
        assert!(verify_kit_view(&good, &trusted).is_ok());

        // A substituted register file fails offline.
        let swapped_root =
            root_over_entries(&[b"voter-1" as &[u8], b"voter-3", b"voter-2"]);
        let swapped = BundleView {
            merkle_root_hex: swapped_root.clone(),
            ..good.clone()
        };
        assert_eq!(
            verify_kit_view(&swapped, &trusted),
            Err(RegisterAnchorError::RootMismatch {
                bundle_id: "REG-2027-A".into(),
                expected: root,
                actual: swapped_root,
            })
        );

        // An unprovisioned bundle is unknown.
        let rogue = BundleView {
            bundle_id: "ROGUE".into(),
            ..good
        };
        assert_eq!(
            verify_kit_view(&rogue, &trusted),
            Err(RegisterAnchorError::UnknownBundle("ROGUE".into()))
        );
    }

    #[test]
    fn registry_parsing_rejects_garbage() {
        assert!(parse_registry("no-colon").is_err());
        assert!(parse_registry("id:short").is_err());
        let ok = parse_registry("A:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,B:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
        assert_eq!(ok.len(), 2);
    }
}
