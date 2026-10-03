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

//! cosign keyed bundle verification.
//!
//! Parses a cosign bundle (JSON with `base64Signature`, optional `cert` and
//! `rekorBundle` fields) and cryptographically verifies the signature over
//! the artifact bytes under the provided PEM public key. Two algorithms are
//! supported, matching what `cosign sign --key` produces:
//!
//! - Ed25519 (RFC 8032): 64-byte signature over the raw artifact bytes.
//! - ECDSA P-256 with SHA-256: DER-encoded ASN.1 signature.
//!
//! Public keys are accepted as SPKI DER inside a PEM `PUBLIC KEY` wrapper
//! (the format produced by `cosign public-key`, OpenSSL, and sigstore key
//! pairs). The Ed25519 SPKI algorithm OID is `1.3.101.112`; P-256 uses
//! algorithm `1.2.840.10045.2.1` with curve parameter `1.2.840.10045.3.1.7`.
//!
//! A `rekorBundle` field is parsed for completeness but its inclusion proof
//! is NOT verified here: Rekor entry verification requires the trusted
//! Rekor signing key, which belongs to the transparency-svc witness tooling.
//!
//! # Feature
//!
//! Requires the `cosign` feature.

use base64::Engine as _;
use serde::Deserialize;

use crate::CryptoError;

/// A cosign bundle as produced by `cosign sign --bundle`.
///
/// # Example
///
/// ```
/// use eel_crypto_rs::cosign::CosignBundle;
/// let json = r#"{"base64Signature":"AAAA","cert":null,"rekorBundle":null}"#;
/// let _: CosignBundle = serde_json::from_str(json).unwrap();
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CosignBundle {
    /// Base64-encoded signature bytes.
    pub base64_signature: String,
    /// Optional PEM-encoded signing certificate.
    pub cert: Option<String>,
    /// Optional Rekor transparency-log bundle.
    pub rekor_bundle: Option<serde_json::Value>,
}

/// Parse and verify a cosign bundle against `public_key_pem` over
/// `artifact_bytes`.
///
/// Any well-formed bundle whose signature does not verify returns
/// [`CryptoError::Cosign`]; this function never accepts an unverified bundle.
pub fn verify_bundle(
    bundle_json: &str,
    public_key_pem: &str,
    artifact_bytes: &[u8],
) -> Result<CosignBundle, CryptoError> {
    let bundle: CosignBundle =
        serde_json::from_str(bundle_json).map_err(|e| CryptoError::Cosign(e.to_string()))?;

    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(&bundle.base64_signature)
        .map_err(|e| CryptoError::Cosign(format!("base64 decode: {e}")))?;

    match parse_spki_public_key(public_key_pem)? {
        PublicKey::Ed25519(vk) => {
            use ed25519_dalek::Verifier;
            if sig_bytes.len() != 64 {
                return Err(CryptoError::Cosign(format!(
                    "ed25519 signature must be 64 bytes, got {}",
                    sig_bytes.len()
                )));
            }
            let sig = ed25519_dalek::Signature::from_slice(&sig_bytes)
                .map_err(|e| CryptoError::Cosign(format!("ed25519 signature: {e}")))?;
            vk.verify(artifact_bytes, &sig)
                .map_err(|e| CryptoError::Cosign(format!("ed25519 verify failed: {e}")))?;
        }
        PublicKey::P256(vk) => {
            use p256::ecdsa::signature::Verifier;
            use p256::ecdsa::Signature as EcdsaSig;
            let sig = match EcdsaSig::from_der(&sig_bytes) {
                Ok(s) => s,
                Err(_) => EcdsaSig::try_from(sig_bytes.as_slice()).map_err(|_| {
                    CryptoError::Cosign("p256 signature is neither DER nor fixed-width".into())
                })?,
            };
            vk.verify(artifact_bytes, &sig)
                .map_err(|e| CryptoError::Cosign(format!("p256 verify failed: {e}")))?;
        }
    }
    Ok(bundle)
}

enum PublicKey {
    Ed25519(ed25519_dalek::VerifyingKey),
    P256(p256::ecdsa::VerifyingKey),
}

/// Minimal DER reader for the subset of structures an SPKI key can contain.
struct Der<'a> {
    buf: &'a [u8],
}

impl<'a> Der<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    /// Read one TLV; returns (tag, content) with content sliced off `buf`.
    fn tlv(&mut self) -> Result<(u8, &'a [u8]), CryptoError> {
        if self.buf.len() < 2 {
            return Err(CryptoError::Cosign("der truncated".into()));
        }
        let tag = self.buf[0];
        let mut idx = 1usize;
        let len0 = self.buf[idx] as usize;
        idx += 1;
        let len = if len0 & 0x80 == 0 {
            len0
        } else {
            let n = len0 & 0x7f;
            if n == 0 || n > 4 || self.buf.len() < idx + n {
                return Err(CryptoError::Cosign("der bad length".into()));
            }
            let mut l = 0usize;
            for b in &self.buf[idx..idx + n] {
                l = (l << 8) | *b as usize;
            }
            idx += n;
            l
        };
        if self.buf.len() < idx + len {
            return Err(CryptoError::Cosign("der truncated body".into()));
        }
        let content = &self.buf[idx..idx + len];
        self.buf = &self.buf[idx + len..];
        Ok((tag, content))
    }
}

fn oid_eq(content: &[u8], expected: &[u8]) -> bool {
    content == expected
}

const OID_ED25519: &[u8] = &[0x2b, 0x65, 0x70]; // 1.3.101.112
const OID_EC_PUBLIC_KEY: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01]; // 1.2.840.10045.2.1
const OID_P256_CURVE: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07]; // prime256v1

/// Decode a PEM block to DER bytes (first BEGIN/END section found).
fn pem_to_der(pem: &str) -> Result<Vec<u8>, CryptoError> {
    let start = pem
        .find("-----BEGIN")
        .ok_or_else(|| CryptoError::Cosign("missing PEM BEGIN marker".into()))?;
    let after_header = pem[start..]
        .find('\n')
        .map(|i| start + i + 1)
        .ok_or_else(|| CryptoError::Cosign("malformed PEM header".into()))?;
    let end = pem[after_header..]
        .find("-----END")
        .map(|i| after_header + i)
        .unwrap_or(pem.len());
    let body: String = pem[after_header..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    base64::engine::general_purpose::STANDARD
        .decode(body.as_bytes())
        .map_err(|e| CryptoError::Cosign(format!("pem base64 decode: {e}")))
}

fn parse_spki_public_key(pem: &str) -> Result<PublicKey, CryptoError> {
    let der = pem_to_der(pem)?;

    // SubjectPublicKeyInfo ::= SEQUENCE { algorithm AlgorithmIdentifier,
    //                                     subjectPublicKey BIT STRING }
    let mut top = Der::new(&der);
    let (tag, spki) = top.tlv()?;
    if tag != 0x30 {
        return Err(CryptoError::Cosign("expected SPKI SEQUENCE".into()));
    }

    let mut seq = Der::new(spki);
    let (alg_tag, alg_seq) = seq.tlv()?;
    if alg_tag != 0x30 {
        return Err(CryptoError::Cosign(
            "expected AlgorithmIdentifier SEQUENCE".into(),
        ));
    }
    let (bit_tag, bit_content) = seq.tlv()?;
    if bit_tag != 0x03 || bit_content.is_empty() {
        return Err(CryptoError::Cosign(
            "expected subjectPublicKey BIT STRING".into(),
        ));
    }
    let key_bits = &bit_content[1..]; // skip unused-bits octet

    let mut alg = Der::new(alg_seq);
    let (oid_tag, oid) = alg.tlv()?;
    if oid_tag != 0x06 {
        return Err(CryptoError::Cosign("expected algorithm OID".into()));
    }

    if oid_eq(oid, OID_ED25519) {
        let arr: [u8; 32] = key_bits
            .try_into()
            .map_err(|_| CryptoError::Cosign("ed25519 subjectPublicKey must be 32 bytes".into()))?;
        return Ok(PublicKey::Ed25519(
            ed25519_dalek::VerifyingKey::from_bytes(&arr)
                .map_err(|e| CryptoError::Cosign(format!("ed25519 key: {e}")))?,
        ));
    }
    if oid_eq(oid, OID_EC_PUBLIC_KEY) {
        let (_, curve_oid) = alg
            .tlv()
            .map_err(|_| CryptoError::Cosign("EC key missing named-curve parameter".into()))?;
        if !oid_eq(curve_oid, OID_P256_CURVE) {
            return Err(CryptoError::Cosign(
                "only the P-256 (prime256v1) curve is supported".into(),
            ));
        }
        let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(key_bits)
            .map_err(|e| CryptoError::Cosign(format!("p256 key: {e}")))?;
        return Ok(PublicKey::P256(vk));
    }
    Err(CryptoError::Cosign(
        "unsupported public-key algorithm".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    /// Build the canonical Ed25519 SPKI DER:
    /// SEQ( SEQ(OID ed25519), BITSTRING(0x00 || 32-byte key) )
    fn ed25519_spki(vk: &ed25519_dalek::VerifyingKey) -> Vec<u8> {
        // AlgId: 30 05 06 03 2b 65 70
        let alg: [u8; 7] = [0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70];
        let bit = [&[0x00u8][..], &vk.to_bytes()[..]].concat();
        let inner_len = alg.len() + 2 + bit.len();
        let mut der = vec![0x30, inner_len as u8];
        der.extend_from_slice(&alg);
        der.push(0x03);
        der.push(bit.len() as u8);
        der.extend_from_slice(&bit);
        der
    }

    /// Build the canonical P-256 SPKI DER for an uncompressed point.
    fn p256_spki(point_uncompressed: &[u8]) -> Vec<u8> {
        // AlgId: SEQ { OID ecPublicKey(7B), OID prime256v1(8B) } -> content 19 bytes
        let mut alg = vec![0x30, 0x13];
        alg.extend_from_slice(&[0x06, OID_EC_PUBLIC_KEY.len() as u8]);
        alg.extend_from_slice(OID_EC_PUBLIC_KEY);
        alg.extend_from_slice(&[0x06, OID_P256_CURVE.len() as u8]);
        alg.extend_from_slice(OID_P256_CURVE);

        let bit = [&[0x00u8][..], point_uncompressed].concat();
        let inner_len = alg.len() + 2 + bit.len();
        let mut der = vec![0x30, inner_len as u8];
        der.extend_from_slice(&alg);
        der.push(0x03);
        der.push(bit.len() as u8);
        der.extend_from_slice(&bit);
        der
    }

    fn pem_encode(der: &[u8]) -> String {
        let b64 = base64::engine::general_purpose::STANDARD.encode(der);
        let wrapped: Vec<String> = b64
            .as_bytes()
            .chunks(64)
            .map(|c| String::from_utf8_lossy(c).into_owned())
            .collect();
        format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
            wrapped.join("\n")
        )
    }

    #[test]
    fn parse_minimal_bundle() {
        let json = r#"{"base64Signature":"AAAA","cert":null,"rekorBundle":null}"#;
        let bundle: CosignBundle = serde_json::from_str(json).unwrap();
        assert_eq!(bundle.base64_signature, "AAAA");
    }

    #[test]
    fn verify_bundle_invalid_base64_errors() {
        let json = r#"{"base64Signature":"!!!not-base64!!!","cert":null,"rekorBundle":null}"#;
        assert!(verify_bundle(json, "", b"data").is_err());
    }

    #[test]
    fn ed25519_bundle_roundtrip_verifies() {
        use ed25519_dalek::Signer;
        let sk = ed25519_dalek::SigningKey::from_bytes(&[11u8; 32]);
        let vk = sk.verifying_key();
        let artifact = b"election-artifact-bytes";
        let sig = sk.sign(artifact);

        let b64 = base64::engine::general_purpose::STANDARD.encode(sig.to_bytes());
        let json = serde_json::json!({ "base64Signature": b64 }).to_string();
        let out = verify_bundle(&json, &pem_encode(&ed25519_spki(&vk)), artifact)
            .expect("well-signed bundle must verify");
        assert_eq!(out.base64_signature, b64);
    }

    #[test]
    fn tampered_artifact_fails_closed() {
        use ed25519_dalek::Signer;
        let sk = ed25519_dalek::SigningKey::from_bytes(&[12u8; 32]);
        let vk = sk.verifying_key();
        let sig = sk.sign(b"original");
        let json = serde_json::json!({
            "base64Signature":
                base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()),
        })
        .to_string();
        let err = verify_bundle(&json, &pem_encode(&ed25519_spki(&vk)), b"tampered")
            .expect_err("tampered artifact must not verify");
        assert!(matches!(err, CryptoError::Cosign(_)));
    }

    #[test]
    fn wrong_key_fails_verification() {
        use ed25519_dalek::Signer;
        let signer = ed25519_dalek::SigningKey::from_bytes(&[13u8; 32]);
        let other = ed25519_dalek::SigningKey::from_bytes(&[14u8; 32]);
        let sig = signer.sign(b"payload");
        let json = serde_json::json!({
            "base64Signature":
                base64::engine::general_purpose::STANDARD.encode(sig.to_bytes()),
        })
        .to_string();
        assert!(verify_bundle(
            &json,
            &pem_encode(&ed25519_spki(&other.verifying_key())),
            b"payload"
        )
        .is_err());
    }

    #[test]
    fn p256_bundle_roundtrip_verifies() {
        use p256::ecdsa::signature::Signer;
        use p256::ecdsa::{Signature as P256Sig, SigningKey as P256Sk};

        let sk = P256Sk::from_slice(&[9u8; 32]).unwrap();
        let vk = sk.verifying_key();
        let point = vk.to_encoded_point(false); // 0x04 || X || Y (65 bytes)

        let artifact = b"p256-artifact";
        let sig: P256Sig = sk.sign(artifact);
        let json = serde_json::json!({
            "base64Signature":
                base64::engine::general_purpose::STANDARD.encode(sig.to_der().as_bytes()),
        })
        .to_string();

        verify_bundle(&json, &pem_encode(&p256_spki(point.as_bytes())), artifact)
            .expect("p256 signed bundle must verify");
        assert!(verify_bundle(&json, &pem_encode(&p256_spki(point.as_bytes())), b"other").is_err());
    }

    #[test]
    fn non_pem_input_is_rejected() {
        let json = serde_json::json!({ "base64Signature": base64::engine::general_purpose::STANDARD.encode([0u8; 64]) }).to_string();
        assert!(verify_bundle(&json, "not a pem", b"x").is_err());
    }
}
