//! SMS fallback codec: CBOR canonical + zstd.
//!
//! Encoding contract:
//!   1. Serialise `SmsFallbackForm` with `ciborium` (CBOR, RFC 8949). ciborium
//!      emits map keys in definition order and uses shortest integer encoding,
//!      giving deterministic output suitable as a signing scope (RFC 8785 intent
//!      applied to CBOR canonical form).
//!   2. Compress with zstd level 19 (maximum) to fit within the 5-SMS-segment
//!      budget (~800 bytes on a typical 34A summary).
//!
//! The `signature` field in `SmsFallbackForm` covers the canonical CBOR of all
//! fields *except* `signature` itself; callers must set `signature = vec![]`
//! before computing the signing scope.

use crate::models::SmsFallbackForm;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("CBOR serialisation failed: {0}")]
    CborSerialise(String),
    #[error("CBOR deserialisation failed: {0}")]
    CborDeserialise(String),
    #[error("zstd compression failed: {0}")]
    ZstdCompress(#[from] std::io::Error),
    #[error("payload exceeds SMS budget: {actual} bytes (limit {limit})")]
    PayloadTooLarge { actual: usize, limit: usize },
}

/// Maximum raw bytes after encode+compress for the 5-SMS-segment budget on 3G.
/// GSM7 encodes 160 chars/segment; binary SMS uses 140 bytes/segment.
/// 5 segments * 133 usable bytes (after UDH) = 665 bytes. We allow 700 to
/// account for encoding overhead; the SLA test validates end-to-end delivery
/// within the 10-minute budget.
pub const SMS_BUDGET_BYTES: usize = 700;

/// Encode a `SmsFallbackForm` to a CBOR+zstd blob ready for SMS transmission.
pub fn encode_sms(form: &SmsFallbackForm) -> Result<Vec<u8>, CodecError> {
    let cbor = cbor_serialise(form)?;
    let compressed = zstd::encode_all(cbor.as_slice(), 19).map_err(CodecError::ZstdCompress)?;
    if compressed.len() > SMS_BUDGET_BYTES {
        return Err(CodecError::PayloadTooLarge {
            actual: compressed.len(),
            limit: SMS_BUDGET_BYTES,
        });
    }
    Ok(compressed)
}

/// Decode a CBOR+zstd blob received from the SMS gateway back into a form.
pub fn decode_sms(bytes: &[u8]) -> Result<SmsFallbackForm, CodecError> {
    let cbor = zstd::decode_all(bytes).map_err(CodecError::ZstdCompress)?;
    cbor_deserialise(&cbor)
}

/// Return the canonical CBOR signing scope: the form with `signature = vec![]`.
pub fn signing_scope(form: &SmsFallbackForm) -> Result<Vec<u8>, CodecError> {
    let mut scope = form.clone();
    scope.signature = vec![];
    cbor_serialise(&scope)
}

// Internal CBOR helpers using ciborium -----------------------------------------

fn cbor_serialise(form: &SmsFallbackForm) -> Result<Vec<u8>, CodecError> {
    // ciborium serialises via serde; the output is deterministic because the
    // struct fields are in a fixed order and ciborium uses CBOR map with
    // integer-keyed definitions (smallest-integer-first canonical form).
    let mut buf = Vec::new();
    ciborium::into_writer(form, &mut buf).map_err(|e| CodecError::CborSerialise(e.to_string()))?;
    Ok(buf)
}

fn cbor_deserialise(bytes: &[u8]) -> Result<SmsFallbackForm, CodecError> {
    ciborium::from_reader(bytes).map_err(|e| CodecError::CborDeserialise(e.to_string()))
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CandidateTotal;

    fn sample_form() -> SmsFallbackForm {
        SmsFallbackForm {
            station_code: "47-001-001".into(),
            election_id: "GE-2027".into(),
            position_code: "PRES".into(),
            votes_cast: 450,
            valid_votes: 440,
            rejected_votes: 8,
            spoilt_ballots: 2,
            candidate_totals: vec![
                CandidateTotal {
                    candidate_id: "C01".into(),
                    votes: 230,
                },
                CandidateTotal {
                    candidate_id: "C02".into(),
                    votes: 210,
                },
            ],
            kit_serial: "KIT-2027-00042".into(),
            sequence_number: 1,
            transmitted_at_us: 1_800_000_000_000_000,
            signature: vec![0u8; 64],
        }
    }

    #[test]
    fn round_trip_cbor_zstd() {
        let form = sample_form();
        let encoded = encode_sms(&form).expect("encode should succeed");
        let decoded = decode_sms(&encoded).expect("decode should succeed");
        assert_eq!(decoded.station_code, form.station_code);
        assert_eq!(decoded.votes_cast, form.votes_cast);
        assert_eq!(decoded.candidate_totals.len(), form.candidate_totals.len());
    }

    #[test]
    fn signing_scope_clears_signature() {
        let form = sample_form();
        let scope_bytes = signing_scope(&form).expect("scope should succeed");
        let decoded_scope: SmsFallbackForm =
            ciborium::from_reader(scope_bytes.as_slice()).expect("scope should be valid CBOR");
        assert!(decoded_scope.signature.is_empty());
        assert_eq!(decoded_scope.votes_cast, form.votes_cast);
    }

    #[test]
    fn deterministic_encoding() {
        let form = sample_form();
        let a = encode_sms(&form).unwrap();
        let b = encode_sms(&form).unwrap();
        assert_eq!(a, b, "encoding must be deterministic");
    }
}
