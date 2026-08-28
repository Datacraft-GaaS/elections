//! Ed25519-signed artifact attestations binding a SHA-256 artifact digest to
//! a Rekor inclusion UUID and a named signer. Canonical bytes are the
//! pipe-joined fields so verification is independent of JSON formatting.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactAttestation {
    pub artifact_sha256: String,
    pub rekor_uuid: String,
    pub signer_name: String,
    pub attested_at: String,
}

/// Wire format: attestation fields plus hex signature and verifying key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAttestation {
    #[serde(flatten)]
    pub attestation: ArtifactAttestation,
    pub signature: String,
    pub signer_pubkey: String,
}

/// Validate a lowercase/uppercase 64-character hex SHA-256 digest.
pub fn validate_artifact_sha256(digest: &str) -> Result<(), String> {
    if digest.len() != 64 {
        return Err(format!(
            "artifact_sha256 must be 64 hex chars, got {}",
            digest.len()
        ));
    }
    if !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("artifact_sha256 must be hex".into());
    }
    Ok(())
}

impl ArtifactAttestation {
    pub fn new(
        artifact_sha256: &str,
        rekor_uuid: &str,
        signer_name: &str,
        attested_at: &str,
    ) -> Result<Self, String> {
        validate_artifact_sha256(artifact_sha256)?;
        Ok(Self {
            artifact_sha256: artifact_sha256.to_string(),
            rekor_uuid: rekor_uuid.to_string(),
            signer_name: signer_name.to_string(),
            attested_at: attested_at.to_string(),
        })
    }
}

/// Canonical signing bytes: `sha256 | rekor_uuid | signer_name | attested_at`.
pub fn canonical_attestation_bytes(a: &ArtifactAttestation) -> Vec<u8> {
    format!(
        "{}|{}|{}|{}",
        a.artifact_sha256, a.rekor_uuid, a.signer_name, a.attested_at
    )
    .into_bytes()
}

fn load_signing_key(seed_hex: &str) -> anyhow::Result<SigningKey> {
    let seed: [u8; 32] = hex::decode(seed_hex.trim())
        .map_err(|e| anyhow::anyhow!("signing key not hex: {e}"))?
        .try_into()
        .map_err(|_| anyhow::anyhow!("signing key must be a 32-byte Ed25519 seed"))?;
    Ok(SigningKey::from_bytes(&seed))
}

/// Verify `signature` (raw 64-byte Ed25519) over the canonical bytes of
/// `attestation` with the hex-encoded verifying key. Also re-validates the
/// digest format.
pub fn verify_attestation(
    attestation: &ArtifactAttestation,
    signature: &[u8],
    pubkey_hex: &str,
) -> anyhow::Result<bool> {
    validate_artifact_sha256(&attestation.artifact_sha256)
        .map_err(|e| anyhow::anyhow!(e))?;
    let pk: [u8; 32] = hex::decode(pubkey_hex.trim())
        .map_err(|e| anyhow::anyhow!("pubkey not hex: {e}"))?
        .try_into()
        .map_err(|_| anyhow::anyhow!("pubkey must be 32 bytes"))?;
    let vk = VerifyingKey::from_bytes(&pk)
        .map_err(|_| anyhow::anyhow!("invalid Ed25519 verifying key"))?;
    let sig: [u8; 64] = signature
        .try_into()
        .map_err(|_| anyhow::anyhow!("signature must be 64 bytes"))?;
    Ok(vk
        .verify(&canonical_attestation_bytes(attestation), &Signature::from_bytes(&sig))
        .is_ok())
}

/// Sign the canonical bytes and print the signed attestation as JSON.
pub async fn run(
    artifact_sha256: &str,
    rekor_uuid: &str,
    signer_name: &str,
    signing_key_hex: &str,
) -> anyhow::Result<()> {
    let att = ArtifactAttestation {
        artifact_sha256: artifact_sha256.to_string(),
        rekor_uuid: rekor_uuid.to_string(),
        signer_name: signer_name.to_string(),
        attested_at: chrono::Utc::now().to_rfc3339(),
    };
    validate_artifact_sha256(artifact_sha256).map_err(|e| anyhow::anyhow!(e))?;

    let key = load_signing_key(signing_key_hex)?;
    let sig = key.sign(&canonical_attestation_bytes(&att));
    let signed = SignedAttestation {
        attestation: att,
        signature: hex::encode(sig.to_bytes()),
        signer_pubkey: hex::encode(key.verifying_key().as_bytes()),
    };
    println!("{}", serde_json::to_string_pretty(&signed)?);
    Ok(())
}

/// Load a signed attestation from disk and verify it. Returns false when the
/// signature does not verify so the caller can exit non-zero.
pub async fn run_verify(attestation_file: &str, pubkey_hex: &str) -> anyhow::Result<bool> {
    let raw = std::fs::read_to_string(attestation_file)
        .map_err(|e| anyhow::anyhow!("cannot read {attestation_file}: {e}"))?;
    let signed: SignedAttestation = serde_json::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("invalid attestation JSON: {e}"))?;
    let sig_bytes = hex::decode(signed.signature.trim())
        .map_err(|e| anyhow::anyhow!("signature not hex: {e}"))?;
    // Prefer the embedded signer pubkey unless an explicit one was supplied.
    let pk = if pubkey_hex.is_empty() {
        signed.signer_pubkey.as_str()
    } else {
        pubkey_hex
    };
    verify_attestation(&signed.attestation, &sig_bytes, pk)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> SigningKey {
        SigningKey::from_bytes(&[9u8; 32])
    }

    fn sample() -> ArtifactAttestation {
        ArtifactAttestation::new(
            &"ab".repeat(32),
            "rekor-uuid-1",
            "iag-officer",
            "2026-08-25T00:00:00Z",
        )
        .unwrap()
    }

    #[test]
    fn rejects_short_digest() {
        assert!(validate_artifact_sha256("abcd").is_err());
        assert!(validate_artifact_sha256(&"ab".repeat(31)).is_err());
    }

    #[test]
    fn rejects_non_hex_digest() {
        assert!(validate_artifact_sha256(&"zz".repeat(32)).is_err());
    }

    #[test]
    fn canonical_bytes_are_stable_pipe_joined() {
        let a = sample();
        assert_eq!(
            String::from_utf8(canonical_attestation_bytes(&a)).unwrap(),
            format!("{}|rekor-uuid-1|iag-officer|2026-08-25T00:00:00Z", "ab".repeat(32))
        );
        assert_eq!(canonical_attestation_bytes(&a), canonical_attestation_bytes(&a));
    }

    #[test]
    fn sign_verify_round_trip() {
        let att = sample();
        let key = test_key();
        let sig = key.sign(&canonical_attestation_bytes(&att));
        let pk = hex::encode(key.verifying_key().as_bytes());
        assert!(
            verify_attestation(&att, &sig.to_bytes(), &pk).unwrap()
        );
    }

    #[test]
    fn tampered_field_fails_verification() {
        let att = sample();
        let key = test_key();
        let sig = key.sign(&canonical_attestation_bytes(&att));
        let pk = hex::encode(key.verifying_key().as_bytes());

        let mut tampered = att.clone();
        tampered.rekor_uuid = "other".into();
        assert!(!verify_attestation(&tampered, &sig.to_bytes(), &pk).unwrap());

        let mut bad_digest = att.clone();
        bad_digest.artifact_sha256 = "cd".repeat(32);
        assert!(!verify_attestation(&bad_digest, &sig.to_bytes(), &pk).unwrap());
    }
}
