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

//! KitCertEntry: the log leaf type for a kit certificate issuance event.
//!
//! Serialised to JSON and hashed (RFC 6962 leaf hash) before appending to the
//! KitCertTransparencyLog.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single entry in the kit-cert transparency log.
///
/// Every field is mandatory. The `cert_der` is stored raw (base64 in JSON) to
/// allow re-verification without a CA re-issuance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitCertEntry {
    /// Stable hardware identifier for the kit (UUID7 string).
    pub kit_id: String,

    /// DER-encoded X.509 certificate issued to this kit.
    #[serde(with = "base64_bytes")]
    pub cert_der: Vec<u8>,

    /// Time at which the CA signed the certificate.
    pub issued_at: DateTime<Utc>,

    /// Human-readable issuer CA name (e.g. "eel-step-ca").
    pub issuer_ca: String,

    /// SPIFFE ID bound to this certificate (e.g. "spiffe://eel.internal/kit/abc123").
    pub spiffe_id: String,

    /// Reference to a revocation record if the cert has been revoked, otherwise None.
    pub revocation_ref: Option<String>,
}

impl KitCertEntry {
    /// Serialise to canonical JSON bytes for Merkle leaf hashing.
    ///
    /// Keys are ordered by their Rust field declaration order because serde_json
    /// preserves that order for structs. This is deterministic across runs.
    pub fn to_leaf_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("KitCertEntry is always serialisable")
    }
}

// ---------------------------------------------------------------------------
// serde helper: base64 encoding for Vec<u8>
// ---------------------------------------------------------------------------

mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        use serde::de::{self, Visitor};
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Vec<u8>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("base64-encoded bytes")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<u8>, E> {
                STANDARD.decode(v).map_err(de::Error::custom)
            }
        }
        d.deserialize_str(V)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_entry() -> KitCertEntry {
        KitCertEntry {
            kit_id: "018f3c60-0000-7000-8000-000000000001".to_string(),
            cert_der: vec![0x30, 0x82, 0x01, 0x00],
            issued_at: Utc.with_ymd_and_hms(2027, 3, 1, 12, 0, 0).unwrap(),
            issuer_ca: "eel-step-ca".to_string(),
            spiffe_id: "spiffe://eel.internal/kit/abc123".to_string(),
            revocation_ref: None,
        }
    }

    #[test]
    fn roundtrip_json() {
        let e = sample_entry();
        let bytes = e.to_leaf_bytes();
        let decoded: KitCertEntry = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, e);
    }

    #[test]
    fn leaf_bytes_deterministic() {
        let e = sample_entry();
        assert_eq!(e.to_leaf_bytes(), e.to_leaf_bytes());
    }

    #[test]
    fn revocation_ref_round_trip() {
        let mut e = sample_entry();
        e.revocation_ref = Some("crl://eel.internal/revoke/abc".to_string());
        let decoded: KitCertEntry = serde_json::from_slice(&e.to_leaf_bytes()).unwrap();
        assert_eq!(decoded.revocation_ref, e.revocation_ref);
    }
}
