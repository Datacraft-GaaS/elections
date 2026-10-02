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

//! 3G-emulated SMS fallback decode test (M08-INT-SMS-001).
//!
//! Validates the full CBOR+zstd encode-decode round-trip against the 3G SLA:
//!   - Compressed payload <= SMS_BUDGET_BYTES (700 bytes / 5-segment cap)
//!   - Decoded fields match source form bit-for-bit
//!   - Deterministic encoding (same output on repeated encode)
//!
//! The "3G emulation" is the payload-size gate: on a 3G link the SMS segments
//! are delivered within the 10-minute SLA if and only if the compressed payload
//! fits within the 5-segment limit. This test asserts that a representative
//! Form 34A summary (8 candidates, all fields populated) still fits.

use eel_trace_rs::trace;
use rts_device_api::codec::{decode_sms, encode_sms, signing_scope, SMS_BUDGET_BYTES};
use rts_device_api::models::{CandidateTotal, SmsFallbackForm};

fn full_form() -> SmsFallbackForm {
    SmsFallbackForm {
        station_code: "47-015-022".into(),
        election_id: "GE-2027".into(),
        position_code: "PRES".into(),
        votes_cast: 698,
        valid_votes: 682,
        rejected_votes: 12,
        spoilt_ballots: 4,
        candidate_totals: vec![
            CandidateTotal {
                candidate_id: "CAND-001".into(),
                votes: 320,
            },
            CandidateTotal {
                candidate_id: "CAND-002".into(),
                votes: 180,
            },
            CandidateTotal {
                candidate_id: "CAND-003".into(),
                votes: 92,
            },
            CandidateTotal {
                candidate_id: "CAND-004".into(),
                votes: 45,
            },
            CandidateTotal {
                candidate_id: "CAND-005".into(),
                votes: 28,
            },
            CandidateTotal {
                candidate_id: "CAND-006".into(),
                votes: 11,
            },
            CandidateTotal {
                candidate_id: "CAND-007".into(),
                votes: 4,
            },
            CandidateTotal {
                candidate_id: "CAND-008".into(),
                votes: 2,
            },
        ],
        kit_serial: "KIT-2027-00099".into(),
        sequence_number: 3,
        transmitted_at_us: 1_800_123_456_789_000,
        signature: vec![0xABu8; 64],
    }
}

#[trace("FR-M08-019", "EC-CON-03")]
#[test]
fn sms_payload_fits_3g_budget() {
    let form = full_form();
    let encoded = encode_sms(&form).expect("encode must succeed");
    assert!(
        encoded.len() <= SMS_BUDGET_BYTES,
        "3G budget exceeded: {} bytes (limit {})",
        encoded.len(),
        SMS_BUDGET_BYTES
    );
    println!(
        "3G SMS payload: {} bytes (budget {})",
        encoded.len(),
        SMS_BUDGET_BYTES
    );
}

#[trace("FR-M08-018")]
#[test]
fn decode_round_trip_all_fields() {
    let form = full_form();
    let encoded = encode_sms(&form).expect("encode");
    let decoded = decode_sms(&encoded).expect("decode");

    assert_eq!(decoded.station_code, form.station_code);
    assert_eq!(decoded.election_id, form.election_id);
    assert_eq!(decoded.position_code, form.position_code);
    assert_eq!(decoded.votes_cast, form.votes_cast);
    assert_eq!(decoded.valid_votes, form.valid_votes);
    assert_eq!(decoded.rejected_votes, form.rejected_votes);
    assert_eq!(decoded.spoilt_ballots, form.spoilt_ballots);
    assert_eq!(decoded.kit_serial, form.kit_serial);
    assert_eq!(decoded.sequence_number, form.sequence_number);
    assert_eq!(decoded.transmitted_at_us, form.transmitted_at_us);
    assert_eq!(decoded.candidate_totals.len(), form.candidate_totals.len());
    for (a, b) in decoded
        .candidate_totals
        .iter()
        .zip(form.candidate_totals.iter())
    {
        assert_eq!(a.candidate_id, b.candidate_id);
        assert_eq!(a.votes, b.votes);
    }
}

#[test]
fn arithmetic_invariant_votes_cast_equals_sum() {
    let form = full_form();
    let candidate_sum: u32 = form.candidate_totals.iter().map(|c| c.votes).sum();
    // valid_votes must equal sum of candidate votes
    assert_eq!(
        form.valid_votes, candidate_sum,
        "valid_votes ({}) != sum of candidate votes ({})",
        form.valid_votes, candidate_sum
    );
    // votes_cast == valid_votes + rejected_votes + spoilt_ballots
    assert_eq!(
        form.votes_cast,
        form.valid_votes + form.rejected_votes + form.spoilt_ballots,
    );
}

#[trace("FR-M08-018", "FR-M08-005")]
#[test]
fn signing_scope_excludes_signature_field() {
    let form = full_form();
    let scope = signing_scope(&form).expect("scope");
    // Scope must be decodable CBOR with empty signature.
    let decoded: SmsFallbackForm =
        ciborium::from_reader(scope.as_slice()).expect("scope must be valid CBOR");
    assert!(
        decoded.signature.is_empty(),
        "signing scope must zero signature field"
    );
    assert_eq!(decoded.votes_cast, form.votes_cast);
}

#[trace("FR-M08-018")]
#[test]
fn deterministic_encode_same_bytes() {
    let form = full_form();
    let a = encode_sms(&form).unwrap();
    let b = encode_sms(&form).unwrap();
    assert_eq!(a, b, "CBOR+zstd output must be deterministic");
}
