//! Tests for physical-seal-binding (HARD-16). 15+ cases.

use chrono::{Duration, Utc};
use eel_inventory_physical_seal_binding::{
    bind::CustodyEvent,
    bind_custody_event,
    seal_photo::SealPhotoSignature,
    verify::{make_test_signature, verify_bound_event},
    SealBindError,
};

const SEAL: &str = "SEAL-KE-0042";
const KIT: &str = "KIT-NAI-0007";

fn photo_bytes() -> Vec<u8> {
    b"fake photo data for testing purposes".to_vec()
}

fn make_photo_sig(seal_id: &str, kit_id: &str, photo: &[u8]) -> SealPhotoSignature {
    let photo_sha256 = SealPhotoSignature::compute_photo_hash(photo);
    let captured_at = Utc::now() - Duration::seconds(5);
    let mut sig = SealPhotoSignature {
        seal_id: seal_id.to_owned(),
        photo_sha256,
        kit_id: kit_id.to_owned(),
        captured_at,
        kit_hsm_signature: String::new(),
        geo_lat: -1.2921,
        geo_lon: 36.8219,
    };
    sig.kit_hsm_signature = make_test_signature(&sig);
    sig
}

fn make_event(seal_id: &str, kit_id: &str, ts_hlc: i64) -> CustodyEvent {
    CustodyEvent {
        event_id: uuid::Uuid::now_v7().to_string(),
        seal_id: seal_id.to_owned(),
        kit_id: kit_id.to_owned(),
        ts_hlc,
        from_custodian: "warehouse-nbi".to_owned(),
        to_custodian: "ro-kiambu".to_owned(),
        location: "-1.1722,36.9371".to_owned(),
    }
}

// ---- happy-path ----

#[test]
fn happy_path_bind_succeeds() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let bound = bind_custody_event(event, sig).expect("bind should succeed");
    assert_eq!(bound.event.seal_id, SEAL);
    assert_eq!(bound.photo_sig.kit_id, KIT);
}

#[test]
fn happy_path_verify_succeeds() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let bound = bind_custody_event(event, sig).unwrap();
    verify_bound_event(&bound, &photo).expect("verify should succeed");
}

#[test]
fn bind_returns_bound_event_with_correct_fields() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let event_id = event.event_id.clone();
    let bound = bind_custody_event(event, sig).unwrap();
    assert_eq!(bound.event.event_id, event_id);
    assert!(!bound.photo_sig.photo_sha256.is_empty());
    assert!(!bound.photo_sig.kit_hsm_signature.is_empty());
}

// ---- tampered photo ----

#[test]
fn tampered_photo_rejected_at_verify() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let bound = bind_custody_event(event, sig).unwrap();
    let tampered = b"tampered photo".to_vec();
    let err = verify_bound_event(&bound, &tampered).unwrap_err();
    assert!(matches!(err, SealBindError::PhotoHashMismatch { .. }));
}

#[test]
fn photo_sha256_field_tampered_after_bind_detected() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let mut bound = bind_custody_event(event, sig.clone()).unwrap();
    // Attacker overwrites the hash in the bound record.
    bound.photo_sig.photo_sha256 = "deadbeef".repeat(8);
    let err = verify_bound_event(&bound, &photo).unwrap_err();
    assert!(matches!(err, SealBindError::PhotoHashMismatch { .. }));
}

// ---- kit sig mismatch ----

#[test]
fn kit_sig_mismatch_rejected_at_bind() {
    let photo = photo_bytes();
    let mut sig = make_photo_sig(SEAL, KIT, &photo);
    sig.kit_hsm_signature = "00".repeat(32); // wrong sig
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let err = bind_custody_event(event, sig).unwrap_err();
    assert!(matches!(err, SealBindError::SignatureInvalid));
}

#[test]
fn kit_sig_corrupt_hex_rejected() {
    let photo = photo_bytes();
    let mut sig = make_photo_sig(SEAL, KIT, &photo);
    sig.kit_hsm_signature = "not-valid-hex!!".to_owned();
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let err = bind_custody_event(event, sig).unwrap_err();
    assert!(matches!(err, SealBindError::SignatureInvalid));
}

#[test]
fn kit_sig_mismatch_rejected_at_verify() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let mut bound = bind_custody_event(event, sig).unwrap();
    bound.photo_sig.kit_hsm_signature = "00".repeat(32);
    let err = verify_bound_event(&bound, &photo).unwrap_err();
    assert!(matches!(err, SealBindError::SignatureInvalid));
}

// ---- seal_id mismatch ----

#[test]
fn seal_id_mismatch_rejected() {
    let photo = photo_bytes();
    let sig = make_photo_sig("SEAL-WRONG", KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let err = bind_custody_event(event, sig).unwrap_err();
    assert!(matches!(err, SealBindError::SealIdMismatch { .. }));
}

#[test]
fn seal_id_tampered_in_bound_event_detected() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let mut bound = bind_custody_event(event, sig).unwrap();
    bound.event.seal_id = "SEAL-TAMPERED".to_owned();
    let err = verify_bound_event(&bound, &photo).unwrap_err();
    assert!(matches!(err, SealBindError::SealIdMismatch { .. }));
}

// ---- kit_id mismatch ----

#[test]
fn kit_id_mismatch_rejected() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, "KIT-OTHER", &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let err = bind_custody_event(event, sig).unwrap_err();
    assert!(matches!(err, SealBindError::KitIdMismatch { .. }));
}

// ---- out-of-order timestamp ----

#[test]
fn event_before_photo_capture_rejected() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    // Event timestamp is 10 seconds before the photo.
    let ts = (sig.captured_at - Duration::seconds(10)).timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let err = bind_custody_event(event, sig).unwrap_err();
    assert!(matches!(err, SealBindError::TimestampOutOfOrder { .. }));
}

#[test]
fn event_exactly_at_capture_accepted() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = sig.captured_at.timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    bind_custody_event(event, sig).expect("exact timestamp should be accepted");
}

#[test]
fn event_after_capture_accepted() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = (sig.captured_at + Duration::minutes(2)).timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    bind_custody_event(event, sig).expect("later timestamp should be accepted");
}

// ---- geo-fields preserved ----

#[test]
fn geo_fields_preserved_in_bound_event() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let bound = bind_custody_event(event, sig).unwrap();
    assert!((bound.photo_sig.geo_lat - (-1.2921_f64)).abs() < 1e-4);
    assert!((bound.photo_sig.geo_lon - 36.8219_f64).abs() < 1e-4);
}

// ---- serialisation round-trip ----

#[test]
fn bound_event_json_round_trips() {
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let bound = bind_custody_event(event, sig).unwrap();
    let json = serde_json::to_string(&bound).unwrap();
    let parsed: eel_inventory_physical_seal_binding::bind::BoundCustodyEvent =
        serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.event.seal_id, SEAL);
}

// ---- api layer ----

#[test]
fn api_handle_bind_seal_happy_path() {
    use eel_inventory_physical_seal_binding::api::{handle_bind_seal, BindSealRequest};
    let photo = photo_bytes();
    let sig = make_photo_sig(SEAL, KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let req = BindSealRequest {
        event,
        photo_sig: sig,
    };
    let resp = handle_bind_seal(req).expect("api bind should succeed");
    assert_eq!(resp.bound.event.seal_id, SEAL);
}

#[test]
fn api_handle_bind_seal_rejects_mismatch() {
    use eel_inventory_physical_seal_binding::api::{handle_bind_seal, BindSealRequest};
    let photo = photo_bytes();
    let sig = make_photo_sig("WRONG", KIT, &photo);
    let ts = Utc::now().timestamp_millis();
    let event = make_event(SEAL, KIT, ts);
    let req = BindSealRequest {
        event,
        photo_sig: sig,
    };
    assert!(handle_bind_seal(req).is_err());
}
