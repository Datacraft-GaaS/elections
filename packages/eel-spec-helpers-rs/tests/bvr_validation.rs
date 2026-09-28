//! Pins the BVR `Voter` pattern predicates to the literals in
//! `packages/openapi-contracts/bvr/openapi.yaml`.
//!
//! These predicates are hand-rolled (the workspace has no regex crate), so they
//! can drift from the contract silently. Each case below quotes the spec's own
//! `example` and the boundaries around its `pattern`.

use eel_spec_helpers_rs::matches_bvr_pattern;

#[test]
fn national_id_accepts_spec_example() {
    // pattern: ^[0-9]{8}$   example: "12345678"
    assert!(matches_bvr_pattern("national_id", "12345678"));
}

#[test]
fn national_id_rejects_wrong_length_and_non_digits() {
    assert!(!matches_bvr_pattern("national_id", "1234567")); // 7 digits
    assert!(!matches_bvr_pattern("national_id", "123456789")); // 9 digits
    assert!(!matches_bvr_pattern("national_id", "1234567a"));
    assert!(!matches_bvr_pattern("national_id", " 1234567"));
    assert!(!matches_bvr_pattern("national_id", ""));
}

#[test]
fn phone_accepts_spec_example() {
    // pattern: ^\+254[0-9]{9}$   example: "+254712345678"
    assert!(matches_bvr_pattern("phone", "+254712345678"));
}

#[test]
fn phone_requires_the_kenyan_e164_prefix() {
    assert!(!matches_bvr_pattern("phone", "+25471234567")); // 8 digits after prefix
    assert!(!matches_bvr_pattern("phone", "+2547123456789")); // 10 digits after prefix
    assert!(!matches_bvr_pattern("phone", "+255712345678")); // wrong country code
    assert!(!matches_bvr_pattern("phone", "254712345678")); // missing +
    assert!(!matches_bvr_pattern("phone", "+25471234567a"));
}

#[test]
fn polling_station_code_accepts_spec_example() {
    // pattern: ^[A-Z0-9]{6,12}$   example: "NRB-001-PS-042"
    // Note the example contains hyphens, so the contract's own example does not
    // satisfy its own pattern. We pin the *pattern*, which is the enforced
    // rule; see polling_station_code_pattern_rejects_hyphens below.
    assert!(!matches_bvr_pattern("polling_station_code", "NRB-001-PS-042"));
    assert!(matches_bvr_pattern("polling_station_code", "NRB001PS042"));
}

#[test]
fn polling_station_code_enforces_length_bounds() {
    assert!(matches_bvr_pattern("polling_station_code", "ABC123")); // 6, lower bound
    assert!(!matches_bvr_pattern("polling_station_code", "ABC12")); // 5, too short
    assert!(matches_bvr_pattern("polling_station_code", "ABCDEFGHIJKL")); // 12, valid
    assert!(!matches_bvr_pattern("polling_station_code", "ABCDEFGHIJKLM")); // 13
}

#[test]
fn polling_station_code_pattern_rejects_hyphens() {
    assert!(!matches_bvr_pattern("polling_station_code", "ABC-123"));
    assert!(!matches_bvr_pattern("polling_station_code", "abc123")); // lowercase
}

#[test]
fn unknown_field_is_permissive() {
    // Callers validating a subset must not be forced to enumerate the contract.
    assert!(matches_bvr_pattern("full_name", "anything at all"));
    assert!(matches_bvr_pattern("", ""));
}
