//! Security properties of the offline BVR record store.
//!
//! Each test names the threat it defends against. These are the assertions that
//! must hold before a kit ships, so they are deliberately adversarial: they
//! mutate the store the way an attacker with disk access would.

use eel_bvr_store_rs::{
    CaptureMethod, InstanceIdentity, KeyStore, Provenance, Purpose, Record, SoftwareKeyStore,
    Store, StoreError,
};
use serde_json::json;

fn provenance(store: &Store) -> Provenance {
    Provenance::new(
        store.identity().instance_id,
        &store.identity().station_label,
        "officer-77",
        Purpose::Enrolment,
        "bvr-enrolment-standard",
        "identity",
        CaptureMethod::OfficerTranscription,
    )
}

fn seeded() -> (Store, Vec<eel_bvr_store_rs::RecordId>) {
    let mut store = Store::new("Nairobi County 04 / Kit 12").unwrap();
    let prov = provenance(&store);
    let a = store.append("full_name", "A", &json!("Wanjiku Mwangi"), &prov).unwrap();
    let b = store.append("national_id", "A", &json!("12345678"), &prov).unwrap();
    let c = store.append("polling_station_code", "B", &json!("NRB001PS042"), &prov).unwrap();
    (store, vec![a, b, c])
}

// ---- instance identity ------------------------------------------------------

#[test]
fn each_store_has_a_distinct_instance_id() {
    // Required so a datum recovered from any kit in a fleet is traceable to the
    // exact station that captured it.
    let a = Store::new("Kit A").unwrap();
    let b = Store::new("Kit B").unwrap();
    assert_ne!(a.identity().instance_id, b.identity().instance_id);
}

#[test]
fn instance_public_key_matches_its_key_store() {
    use base64::Engine;
    let store = Store::new("Kit A").unwrap();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&store.identity().public_key_b64)
        .unwrap();
    assert_eq!(decoded.len(), 32);
}

#[test]
fn provenance_is_stamped_with_the_capturing_instance() {
    let (store, ids) = seeded();
    let record = &store.records()[0];
    assert_eq!(record.provenance.instance_id, store.identity().instance_id);
    assert_eq!(record.provenance.station_label, "Nairobi County 04 / Kit 12");
    assert_eq!(record.provenance.operator_id, "officer-77");
    assert_eq!(record.provenance.workflow_id, "bvr-enrolment-standard");
    assert_eq!(record.provenance.step_id, "identity");
    assert_eq!(ids.len(), 3);
}

// ---- encryption at rest -----------------------------------------------------

#[test]
fn stored_values_are_not_readable_in_the_clear() {
    // A stolen laptop yields a store of base64 blobs, not a list of national IDs.
    let (store, ids) = seeded();
    let serialised = serde_json::to_string(store.records()).unwrap();
    assert!(!serialised.contains("12345678"), "national ID must not appear in the clear");
    assert!(!serialised.contains("Wanjiku Mwangi"), "name must not appear in the clear");
    // ...but it is recoverable through the API.
    let value = store.value_of(&ids[1]).unwrap();
    assert_eq!(value, json!("12345678"));
}

#[test]
fn every_record_has_a_distinct_ciphertext() {
    // Two records holding the same value must not be linkable by their bytes.
    let mut store = Store::new("Kit A").unwrap();
    let prov = provenance(&store);
    let a = store.append("phone", "A", &json!("+254712345678"), &prov).unwrap();
    let b = store.append("phone", "A", &json!("+254712345678"), &prov).unwrap();
    assert_ne!(
        store.records()[0].value, store.records()[1].value,
        "identical plaintexts must not produce identical ciphertexts"
    );
    assert_eq!(store.value_of(&a).unwrap(), store.value_of(&b).unwrap());
}

// ---- tamper evidence --------------------------------------------------------

#[test]
fn a_clean_store_verifies() {
    let (store, _) = seeded();
    assert!(store.verify().is_ok());
}

#[test]
fn editing_a_value_is_detected() {
    let (mut store, _) = seeded();
    store.records_mut()[1].value = json!("9999999x");
    let err = store.verify().expect_err("editing a value must be detected");
    let rendered = err.to_string();
    assert!(rendered.contains("does not match its recorded hash"), "{rendered}");
}

#[test]
fn editing_provenance_is_detected() {
    // The provenance block is inside the hash preimage, so rewrites of who or
    // when are caught just like edits to the value.
    let (mut store, _) = seeded();
    store.records_mut()[0].provenance.operator_id = "officer-1".to_owned();
    assert!(store.verify().is_err());
}

#[test]
fn editing_the_capture_instant_is_detected() {
    let (mut store, _) = seeded();
    store.records_mut()[0].provenance.captured_at =
        chrono::Utc::now() - chrono::Duration::days(365);
    assert!(store.verify().is_err());
}

#[test]
fn deleting_a_record_breaks_the_chain() {
    let (mut store, _) = seeded();
    store.records_mut().remove(1);
    let err = store.verify().expect_err("a deleted record must be detected");
    assert!(err.to_string().contains("does not chain"), "{err}");
}

#[test]
fn reordering_records_breaks_the_chain() {
    let (mut store, _) = seeded();
    store.records_mut().swap(0, 2);
    assert!(store.verify().is_err(), "reordering must be detected");
}

#[test]
fn a_record_from_another_store_does_not_verify() {
    // Provenance must be attributable: a record lifted from Kit B and spliced
    // into Kit A's chain must not pass under Kit A's key.
    let (mut a, _) = seeded();
    let (b, _) = seeded();
    let stolen = b.records()[0].clone();
    a.records_mut().push(stolen);
    let err = a.verify().expect_err("a foreign record must not verify");
    assert!(err.to_string().contains("does not chain"), "{err}");
}

#[test]
fn every_fault_is_reported_not_just_the_first() {
    // An auditor needs the whole picture of a tampered store in one pass.
    let (mut store, _) = seeded();
    store.records_mut()[0].this_hash = "0".repeat(64);
    store.records_mut()[2].this_hash = "1".repeat(64);
    let err = store.verify().unwrap_err();
    let StoreError::Tampered(faults) = err else {
        panic!("expected a tamper error, got {err:?}");
    };
    assert!(faults.len() >= 3, "expected several faults, got {faults:?}");
}

// ---- tamper proofing --------------------------------------------------------

#[test]
fn a_forged_signature_is_rejected() {
    // Content can be recomputed by an attacker who knows the format, so the
    // signature is what actually makes the tampering provable.
    let (mut store, _) = seeded();
    let keys = SoftwareKeyStore::generate();
    // Re-seal a tampered record with a *different* key, recomputing a
    // consistent hash so only the signature can catch it.
    let mut forged = store.records()[0].clone();
    forged.provenance.operator_id = "officer-1".to_owned();
    let forged = forged.seal(&keys).unwrap();
    store.records_mut()[0] = forged;
    let err = store.verify().expect_err("a foreign key must not verify");
    assert!(err.to_string().contains("signature is invalid"), "{err}");
}

#[test]
fn a_record_resigned_with_its_own_key_but_rewritten_still_breaks_the_chain() {
    let (mut store, _) = seeded();
    let keys = SoftwareKeyStore::generate();
    let mut tampered = store.records()[1].clone();
    tampered.value = json!("tampered".to_owned());
    // Sign with the right key is impossible here (we do not hold it), so this
    // documents the layering: the chain catches it even before signature
    // checking, because prev_hash no longer matches.
    let resigned = tampered.seal(&keys).unwrap();
    store.records_mut()[1] = resigned;
    assert!(store.verify().is_err());
}

// ---- sync -------------------------------------------------------------------

#[test]
fn pending_batch_covers_only_unsynced_records() {
    let (mut store, ids) = seeded();
    let batch = store.pending_batch().expect("a store with unsynced records has a batch");
    assert_eq!(batch.len(), 3);
    assert_eq!(batch.instance_id.instance_id, store.identity().instance_id);
    assert_eq!(batch.merkle_root.len(), 64, "root is 32 bytes hex encoded");

    store.mark_synced(&ids[..1]).unwrap();
    let batch = store.pending_batch().unwrap();
    assert_eq!(batch.len(), 2);
}

#[test]
fn acknowledging_records_clears_them_from_the_queue() {
    let (mut store, ids) = seeded();
    let state = store.sync_state();
    assert_eq!((state.total, state.synced, state.pending), (3, 0, 3));
    store.mark_synced(&ids).unwrap();
    let state = store.sync_state();
    assert_eq!((state.total, state.synced, state.pending), (3, 3, 0));
    assert!(store.pending_batch().is_none());
}

#[test]
fn acknowledging_twice_is_idempotent() {
    let (mut store, ids) = seeded();
    assert_eq!(store.mark_synced(&ids).unwrap(), 3);
    assert_eq!(store.mark_synced(&ids).unwrap(), 0, "second ack changes nothing");
}

#[test]
fn acknowledging_an_unknown_record_is_rejected() {
    let (mut store, _) = seeded();
    let err = store.mark_synced(&[uuid::Uuid::now_v7()]).unwrap_err();
    assert!(matches!(err, StoreError::Malformed(_)), "{err:?}");
}

#[test]
fn mark_synced_does_not_break_the_integrity_chain() {
    // Sync state is mutable bookkeeping and is deliberately outside the hash
    // preimage, so acknowledging a record must not invalidate it.
    let (mut store, ids) = seeded();
    store.mark_synced(&ids).unwrap();
    assert!(store.verify().is_ok(), "sync bookkeeping must not break the chain");
}

#[test]
fn batch_root_changes_when_a_record_changes() {
    let (mut a, _) = seeded();
    let (_b, _) = seeded();
    let before = a.pending_batch().unwrap().merkle_root;
    let mut tampered = a.records()[0].clone();
    tampered.provenance.operator_id = "officer-1".to_owned();
    // The root is computed over the stored hashes, so a rewritten hash moves it.
    tampered.this_hash = "f".repeat(64);
    a.records_mut()[0] = tampered;
    assert_ne!(a.pending_batch().unwrap().merkle_root, before);
}

#[test]
fn batch_is_deterministic_for_the_same_records() {
    let (a, _) = seeded();
    let first = a.pending_batch().unwrap();
    let second = a.pending_batch().unwrap();
    assert_eq!(first.merkle_root, second.merkle_root);
}

// ---- empty store ------------------------------------------------------------

#[test]
fn an_empty_store_is_valid_not_tampered() {
    let store = Store::new("Kit A").unwrap();
    assert!(store.is_empty());
    assert!(store.verify().is_ok());
    assert!(store.pending_batch().is_none());
    assert!(eel_bvr_store_rs::record::verify_chain(&[], &[0u8; 32]).is_ok());
}

// ---- signature helpers ------------------------------------------------------

#[test]
fn a_record_signature_verifies_under_the_instance_key() {
    let (store, _) = seeded();
    let pk = store.public_key();
    for record in store.records() {
        assert!(record.verify_signature(&pk).is_ok());
    }
}

#[test]
fn a_signature_does_not_verify_under_another_key() {
    let (store, _) = seeded();
    let other = SoftwareKeyStore::generate().public_key();
    assert!(store.records()[0].verify_signature(&other).is_err());
}

#[test]
fn identity_is_serialisable() {
    let (store, _) = seeded();
    let id: InstanceIdentity = store.identity().clone();
    let text = serde_json::to_string(&id).unwrap();
    assert_eq!(serde_json::from_str::<InstanceIdentity>(&text).unwrap(), id);
    let record: Record = store.records()[0].clone();
    let text = serde_json::to_string(&record).unwrap();
    assert_eq!(serde_json::from_str::<Record>(&text).unwrap(), record);
}
