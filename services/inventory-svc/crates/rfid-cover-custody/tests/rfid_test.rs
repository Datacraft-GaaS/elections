//! Integration / property tests for eel-inventory-rfid-cover-custody.
//!
//! Targets: 30+ tests.
//! No real DB or HSM required; all tests exercise pure logic layers.

use chrono::{Duration, Utc};
use eel_inventory_rfid_cover_custody::{
    domain::{CoverState, ReaderType, Severity, TamperState},
    error::RfidError,
    hash_chain::{compute_this_hash, verify_chain},
    reader::{HmacChallenger, Reader, ReaderInfo, ReaderRegistry},
    replay_defence::{InMemoryNonceStore, NonceStore, REPLAY_WINDOW},
    state_machine::CoverStateMachine,
    tag::{issue_tag, pair_to_box, IssueTagRequest, PairToBoxRequest},
    tamper_alert::{build_alert, classify_severity, RaiseAlertRequest},
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helper: build a minimal CoverStateTransition for hash-chain tests
// ---------------------------------------------------------------------------

fn make_transition(
    tag_id: Uuid,
    from: CoverState,
    to: CoverState,
    prev_hash: Vec<u8>,
) -> eel_inventory_rfid_cover_custody::domain::CoverStateTransition {
    use eel_inventory_rfid_cover_custody::domain::CoverStateTransition;

    let mut t = CoverStateTransition {
        transition_id: Uuid::now_v7(),
        tag_id,
        from_state: from,
        to_state: to,
        triggered_by_read: None,
        officer_id: None,
        officer_svid: None,
        officer_signature: None,
        prev_hash: if prev_hash.is_empty() {
            None
        } else {
            Some(prev_hash.clone())
        },
        this_hash: vec![],
        transitioned_at: Utc::now(),
    };
    let hash = compute_this_hash(&prev_hash, &t);
    t.this_hash = hash.to_vec();
    t
}

// ---------------------------------------------------------------------------
// Tag issuance tests
// ---------------------------------------------------------------------------

#[test]
fn test_issue_tag_produces_valid_epc() {
    let lot_id = Uuid::now_v7();
    let req = IssueTagRequest {
        manufacturer: "NXP".to_owned(),
        lot_id,
        chip_family: "UCODE9".to_owned(),
        tag_secret_hsm_key_id: "hsm-slot-001".to_owned(),
    };
    let tag = issue_tag(req).unwrap();
    assert_eq!(tag.epc.len(), 12, "EPC must be 96 bits (12 bytes)");
    assert_eq!(tag.epc[0], 0x30, "EPC header byte must be SGTIN-96 (0x30)");
    assert_eq!(tag.lot_id, lot_id);
    assert!(tag.tag_id.get_version_num() == 7, "tag_id must be UUID v7");
}

#[test]
fn test_issue_tag_has_no_pairing() {
    let req = IssueTagRequest {
        manufacturer: "Impinj".to_owned(),
        lot_id: Uuid::now_v7(),
        chip_family: "MonzaR6P".to_owned(),
        tag_secret_hsm_key_id: "hsm-slot-002".to_owned(),
    };
    let tag = issue_tag(req).unwrap();
    assert!(tag.paired_with_box.is_none());
    assert!(tag.paired_at.is_none());
    assert_eq!(tag.tamper_state, TamperState::Intact);
}

#[test]
fn test_issue_tag_unique_ids() {
    let lot_id = Uuid::now_v7();
    let req1 = IssueTagRequest {
        manufacturer: "NXP".to_owned(),
        lot_id,
        chip_family: "UCODE9".to_owned(),
        tag_secret_hsm_key_id: "k1".to_owned(),
    };
    let req2 = IssueTagRequest {
        manufacturer: "NXP".to_owned(),
        lot_id,
        chip_family: "UCODE9".to_owned(),
        tag_secret_hsm_key_id: "k2".to_owned(),
    };
    let t1 = issue_tag(req1).unwrap();
    let t2 = issue_tag(req2).unwrap();
    assert_ne!(t1.tag_id, t2.tag_id, "each tag must get a unique UUID v7");
    assert_ne!(t1.epc, t2.epc, "each tag must get a unique EPC");
}

// ---------------------------------------------------------------------------
// Pair-to-box tests
// ---------------------------------------------------------------------------

#[test]
fn test_pair_to_box_succeeds() {
    let lot_id = Uuid::now_v7();
    let tag = issue_tag(IssueTagRequest {
        manufacturer: "NXP".to_owned(),
        lot_id,
        chip_family: "UCODE9".to_owned(),
        tag_secret_hsm_key_id: "k1".to_owned(),
    })
    .unwrap();

    let box_body_id = Uuid::now_v7();
    let paired = pair_to_box(
        tag,
        PairToBoxRequest {
            box_body_id,
            box_lot_id: lot_id,
            box_body_epc: vec![0x30, 0x34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        },
    )
    .unwrap();

    assert_eq!(paired.paired_with_box, Some(box_body_id));
    assert!(paired.paired_at.is_some());
}

#[test]
fn test_pair_to_box_rejects_cross_lot() {
    let lot_id = Uuid::now_v7();
    let tag = issue_tag(IssueTagRequest {
        manufacturer: "NXP".to_owned(),
        lot_id,
        chip_family: "UCODE9".to_owned(),
        tag_secret_hsm_key_id: "k1".to_owned(),
    })
    .unwrap();

    let different_lot = Uuid::now_v7();
    let err = pair_to_box(
        tag,
        PairToBoxRequest {
            box_body_id: Uuid::now_v7(),
            box_lot_id: different_lot,
            box_body_epc: vec![],
        },
    )
    .unwrap_err();

    assert!(
        matches!(err, RfidError::LotMismatch { .. }),
        "expected LotMismatch, got {err:?}"
    );
}

#[test]
fn test_pair_to_box_rejects_already_paired() {
    let lot_id = Uuid::now_v7();
    let tag = issue_tag(IssueTagRequest {
        manufacturer: "NXP".to_owned(),
        lot_id,
        chip_family: "UCODE9".to_owned(),
        tag_secret_hsm_key_id: "k1".to_owned(),
    })
    .unwrap();

    let box_id = Uuid::now_v7();
    let paired = pair_to_box(
        tag,
        PairToBoxRequest {
            box_body_id: box_id,
            box_lot_id: lot_id,
            box_body_epc: vec![],
        },
    )
    .unwrap();

    // Try to pair again.
    let err = pair_to_box(
        paired,
        PairToBoxRequest {
            box_body_id: Uuid::now_v7(),
            box_lot_id: lot_id,
            box_body_epc: vec![],
        },
    )
    .unwrap_err();

    assert!(matches!(err, RfidError::AlreadyPaired { .. }));
}

#[test]
fn test_pair_to_box_rejects_retired_tag() {
    let lot_id = Uuid::now_v7();
    let mut tag = issue_tag(IssueTagRequest {
        manufacturer: "NXP".to_owned(),
        lot_id,
        chip_family: "UCODE9".to_owned(),
        tag_secret_hsm_key_id: "k1".to_owned(),
    })
    .unwrap();
    tag.retired_at = Some(Utc::now());

    let err = pair_to_box(
        tag,
        PairToBoxRequest {
            box_body_id: Uuid::now_v7(),
            box_lot_id: lot_id,
            box_body_epc: vec![],
        },
    )
    .unwrap_err();

    assert!(matches!(err, RfidError::TagRetired(_)));
}

// ---------------------------------------------------------------------------
// HMAC challenge / response tests
// ---------------------------------------------------------------------------

#[test]
fn test_hmac_round_trip() {
    let secret = b"super-secret-32-bytes-long-000000";
    let nonce = b"0123456789abcdef"; // 16 bytes
    let response = HmacChallenger::compute_response(secret, nonce).unwrap();
    let valid = HmacChallenger::verify_response(secret, nonce, &response).unwrap();
    assert!(valid, "HMAC round-trip must verify");
}

#[test]
fn test_hmac_wrong_secret_rejected() {
    let secret = b"correct-secret-32-bytes-long-0000";
    let wrong = b"wrong---secret-32-bytes-long-0000";
    let nonce = b"0123456789abcdef";
    let response = HmacChallenger::compute_response(secret, nonce).unwrap();
    let valid = HmacChallenger::verify_response(wrong, nonce, &response).unwrap();
    assert!(!valid, "HMAC with wrong secret must not verify");
}

#[test]
fn test_hmac_wrong_nonce_rejected() {
    let secret = b"correct-secret-32-bytes-long-0000";
    let nonce1 = b"0123456789abcdef";
    let nonce2 = b"fedcba9876543210";
    let response = HmacChallenger::compute_response(secret, nonce1).unwrap();
    let valid = HmacChallenger::verify_response(secret, nonce2, &response).unwrap();
    assert!(
        !valid,
        "HMAC computed with nonce1 must not verify under nonce2"
    );
}

#[test]
fn test_hmac_challenger_reader_trait() {
    let secret = vec![0u8; 32];
    let challenger = HmacChallenger::new(secret.clone());
    let nonce = b"aaaaaaaaaaaaaaaa";
    let resp = challenger.challenge(nonce).unwrap();
    assert_eq!(resp.len(), 32, "HMAC-SHA256 produces 32 bytes");
    let valid = HmacChallenger::verify_response(&secret, nonce, &resp).unwrap();
    assert!(valid);
}

// ---------------------------------------------------------------------------
// ReaderRegistry / SVID tests
// ---------------------------------------------------------------------------

#[test]
fn test_reader_svid_accepted() {
    let mut reg = ReaderRegistry::new();
    reg.register(
        "RDR-001",
        ReaderInfo {
            svid: "spiffe://eel.local/reader/RDR-001".to_owned(),
            serial: "SN12345".to_owned(),
            location: "Warehouse-A".to_owned(),
        },
    );
    let result = reg.verify_svid("RDR-001", "spiffe://eel.local/reader/RDR-001");
    assert!(result.is_ok());
}

#[test]
fn test_reader_svid_rejected_when_missing() {
    let reg = ReaderRegistry::new();
    let err = reg
        .verify_svid("RDR-999", "spiffe://eel.local/reader/RDR-999")
        .unwrap_err();
    assert!(matches!(err, RfidError::ReaderSvidRejected(_)));
}

#[test]
fn test_reader_svid_rejected_on_mismatch() {
    let mut reg = ReaderRegistry::new();
    reg.register(
        "RDR-001",
        ReaderInfo {
            svid: "spiffe://eel.local/reader/RDR-001".to_owned(),
            serial: "SN1".to_owned(),
            location: "WH".to_owned(),
        },
    );
    let err = reg
        .verify_svid("RDR-001", "spiffe://eel.local/reader/RDR-999")
        .unwrap_err();
    assert!(matches!(err, RfidError::ReaderSvidRejected(_)));
}

// ---------------------------------------------------------------------------
// State machine tests
// ---------------------------------------------------------------------------

#[test]
fn test_state_machine_linear_path_allowed() {
    let linear = [
        (
            CoverState::Manufactured,
            CoverState::Printer,
            ReaderType::WarehouseGate,
        ),
        (
            CoverState::Printer,
            CoverState::Warehouse,
            ReaderType::WarehouseGate,
        ),
        (
            CoverState::Dispatched,
            CoverState::InTransit,
            ReaderType::TransitCheckpoint,
        ),
        (
            CoverState::InTransit,
            CoverState::StationReceived,
            ReaderType::StationArrival,
        ),
        (
            CoverState::StationOpened,
            CoverState::VotingActive,
            ReaderType::KitHandheld,
        ),
        (
            CoverState::SealedReturn,
            CoverState::TallyArrived,
            ReaderType::TransitCheckpoint,
        ),
        (
            CoverState::TallyArrived,
            CoverState::CountComplete,
            ReaderType::RoDesktop,
        ),
        (
            CoverState::CountComplete,
            CoverState::StorageArchive,
            ReaderType::ArchiveGate,
        ),
    ];
    for (from, to, rt) in &linear {
        let result = CoverStateMachine::validate(from, to, rt, Some("svid"));
        assert!(
            result.is_ok(),
            "expected {from} -> {to} via {rt} to be allowed, got {result:?}"
        );
    }
}

#[test]
fn test_state_machine_quarantine_always_allowed() {
    for state in CoverState::all_main() {
        let result = CoverStateMachine::validate(
            &state,
            &CoverState::Quarantine,
            &ReaderType::KitHandheld,
            None,
        );
        assert!(
            result.is_ok(),
            "QUARANTINE must be reachable from {state}, got {result:?}"
        );
    }
}

#[test]
fn test_state_machine_rejects_terminal_to_non_quarantine() {
    let err = CoverStateMachine::validate(
        &CoverState::Destroyed,
        &CoverState::StorageArchive,
        &ReaderType::ArchiveGate,
        Some("svid"),
    )
    .unwrap_err();
    assert!(matches!(err, RfidError::TransitionFromTerminal(_)));
}

#[test]
fn test_state_machine_rejects_quarantine_further_transitions() {
    let err = CoverStateMachine::validate(
        &CoverState::Quarantine,
        &CoverState::StorageArchive,
        &ReaderType::ArchiveGate,
        Some("svid"),
    )
    .unwrap_err();
    assert!(matches!(err, RfidError::TransitionFromTerminal(_)));
}

#[test]
fn test_state_machine_four_eyes_required() {
    // WAREHOUSE -> DISPATCHED requires four-eyes; no officer_svid should fail.
    let err = CoverStateMachine::validate(
        &CoverState::Warehouse,
        &CoverState::Dispatched,
        &ReaderType::WarehouseGate,
        None, // no officer SVID
    )
    .unwrap_err();
    assert!(matches!(err, RfidError::FourEyesRequired { .. }));
}

#[test]
fn test_state_machine_four_eyes_satisfied() {
    let result = CoverStateMachine::validate(
        &CoverState::Warehouse,
        &CoverState::Dispatched,
        &ReaderType::WarehouseGate,
        Some("spiffe://eel.local/officer/ABC123"),
    );
    assert!(result.is_ok());
}

#[test]
fn test_state_machine_property_illegal_transitions() {
    // Enumerate all 15x15 state pairs; those NOT in the table must be rejected
    // (except QUARANTINE target which is always allowed).
    let states = CoverState::all_main();
    let reader = ReaderType::KitHandheld;
    let mut illegal_count = 0usize;

    for from in &states {
        for to in &states {
            if to == &CoverState::Quarantine {
                continue; // always allowed
            }
            // Try the transition without officer svid so four-eyes won't mask the result.
            let allowed = CoverStateMachine::is_allowed(from, to, &reader);
            let validate_result = CoverStateMachine::validate(from, to, &reader, Some("some-svid"));

            if allowed {
                assert!(
                    validate_result.is_ok(),
                    "is_allowed says true but validate returned {validate_result:?} for {from}->{to}"
                );
            } else {
                assert!(
                    validate_result.is_err(),
                    "is_allowed says false but validate returned Ok for {from}->{to}"
                );
                illegal_count += 1;
            }
        }
    }

    // With 15 states and ~13 allowed transitions (excl. QUARANTINE target),
    // illegal pairs should exceed 100.
    assert!(
        illegal_count > 100,
        "expected many illegal transitions, found only {illegal_count}"
    );
}

#[test]
fn test_state_machine_wrong_reader_type_rejected() {
    // MANUFACTURED -> PRINTER requires warehouse_gate; kit_handheld must fail.
    let err = CoverStateMachine::validate(
        &CoverState::Manufactured,
        &CoverState::Printer,
        &ReaderType::KitHandheld,
        Some("svid"),
    )
    .unwrap_err();
    assert!(matches!(err, RfidError::TransitionNotAllowed { .. }));
}

// ---------------------------------------------------------------------------
// Hash chain tests
// ---------------------------------------------------------------------------

#[test]
fn test_hash_chain_genesis_computes() {
    let tag_id = Uuid::now_v7();
    let t = make_transition(
        tag_id,
        CoverState::Manufactured,
        CoverState::Printer,
        vec![],
    );
    assert_eq!(t.this_hash.len(), 32, "BLAKE3 hash must be 32 bytes");
    assert!(t.prev_hash.is_none());
}

#[test]
fn test_hash_chain_integrity_verify_ok() {
    let tag_id = Uuid::now_v7();
    let t1 = make_transition(
        tag_id,
        CoverState::Manufactured,
        CoverState::Printer,
        vec![],
    );
    let t2 = make_transition(
        tag_id,
        CoverState::Printer,
        CoverState::Warehouse,
        t1.this_hash.clone(),
    );
    let t3 = make_transition(
        tag_id,
        CoverState::Warehouse,
        CoverState::Dispatched,
        t2.this_hash.clone(),
    );

    let chain = vec![t1, t2, t3];
    verify_chain(&chain).expect("valid chain must verify without error");
}

#[test]
fn test_hash_chain_tampered_entry_breaks_chain() {
    let tag_id = Uuid::now_v7();
    let t1 = make_transition(
        tag_id,
        CoverState::Manufactured,
        CoverState::Printer,
        vec![],
    );
    let t2 = make_transition(
        tag_id,
        CoverState::Printer,
        CoverState::Warehouse,
        t1.this_hash.clone(),
    );
    let mut t2_tampered = t2.clone();
    // Corrupt this_hash by flipping the first byte.
    t2_tampered.this_hash[0] ^= 0xFF;

    let chain = vec![t1, t2_tampered];
    let err = verify_chain(&chain).unwrap_err();
    assert!(matches!(err, RfidError::ChainBroken { .. }));
}

#[test]
fn test_hash_chain_empty_is_ok() {
    verify_chain(&[]).expect("empty chain must be Ok");
}

#[test]
fn test_hash_chain_single_entry_ok() {
    let tag_id = Uuid::now_v7();
    let t = make_transition(
        tag_id,
        CoverState::Manufactured,
        CoverState::Printer,
        vec![],
    );
    verify_chain(&[t]).expect("single-entry chain must verify");
}

#[test]
fn test_hash_chain_different_tags_do_not_share_hashes() {
    let t1 = make_transition(
        Uuid::now_v7(),
        CoverState::Manufactured,
        CoverState::Printer,
        vec![],
    );
    let t2 = make_transition(
        Uuid::now_v7(),
        CoverState::Manufactured,
        CoverState::Printer,
        vec![],
    );
    // Hash includes tag_id so they should differ.
    assert_ne!(t1.this_hash, t2.this_hash);
}

// ---------------------------------------------------------------------------
// Replay defence tests
// ---------------------------------------------------------------------------

#[test]
fn test_replay_defence_fresh_nonce_accepted() {
    let mut store = InMemoryNonceStore::new();
    let tag_id = "tag-001";
    let nonce = b"unique-nonce-0001";
    let now = Utc::now();
    store
        .check_and_record(tag_id, nonce, now)
        .expect("fresh nonce must be accepted");
}

#[test]
fn test_replay_defence_duplicate_nonce_within_window_rejected() {
    let mut store = InMemoryNonceStore::new();
    let tag_id = "tag-001";
    let nonce = b"unique-nonce-0001";
    let now = Utc::now();

    store
        .check_and_record(tag_id, nonce, now)
        .expect("first call must succeed");

    let err = store
        .check_and_record(tag_id, nonce, now + Duration::seconds(10))
        .unwrap_err();
    assert!(matches!(err, RfidError::ReplayDetected { .. }));
}

#[test]
fn test_replay_defence_expired_nonce_accepted() {
    let mut store = InMemoryNonceStore::new();
    let tag_id = "tag-001";
    let nonce = b"unique-nonce-0001";
    let now = Utc::now();

    store
        .check_and_record(tag_id, nonce, now)
        .expect("first call must succeed");

    // After the replay window expires, the same nonce is fresh again.
    let future = now + REPLAY_WINDOW + Duration::seconds(1);
    store
        .check_and_record(tag_id, nonce, future)
        .expect("expired nonce must be accepted");
}

#[test]
fn test_replay_defence_different_tags_same_nonce_ok() {
    let mut store = InMemoryNonceStore::new();
    let nonce = b"shared-nonce";
    let now = Utc::now();

    store
        .check_and_record("tag-A", nonce, now)
        .expect("tag A must succeed");
    store
        .check_and_record("tag-B", nonce, now)
        .expect("tag B with same nonce must succeed");
}

#[test]
fn test_replay_defence_different_nonces_same_tag_ok() {
    let mut store = InMemoryNonceStore::new();
    let now = Utc::now();
    store
        .check_and_record("tag-001", b"nonce-1", now)
        .expect("nonce-1 must succeed");
    store
        .check_and_record("tag-001", b"nonce-2", now)
        .expect("nonce-2 must succeed");
}

// ---------------------------------------------------------------------------
// Tamper alert tests
// ---------------------------------------------------------------------------

#[test]
fn test_tamper_alert_build_critical_on_cover_lift() {
    let tag_id = Uuid::now_v7();
    let alert = build_alert(RaiseAlertRequest {
        tag_id,
        triggering_read: None,
        expected_state: Some("WAREHOUSE".to_owned()),
        observed_state: Some("cover_lifted".to_owned()),
        severity: Severity::Critical,
    });
    assert_eq!(alert.tag_id, tag_id);
    assert_eq!(alert.severity, Severity::Critical);
    assert!(!alert.ecc_dispatched);
    assert!(alert.resolved_at.is_none());
}

#[test]
fn test_severity_classifier_hmac_failure_is_critical() {
    let s = classify_severity(false, false, false);
    assert_eq!(s, Severity::Critical, "HMAC failure must be Critical");
}

#[test]
fn test_severity_classifier_tamper_indicator_is_critical() {
    let s = classify_severity(true, true, false);
    assert_eq!(s, Severity::Critical);
}

#[test]
fn test_severity_classifier_epc_mismatch_is_critical() {
    let s = classify_severity(false, true, true);
    assert_eq!(s, Severity::Critical);
}

#[test]
fn test_severity_classifier_clean_read_is_warn() {
    let s = classify_severity(false, true, false);
    assert_eq!(s, Severity::Warn, "clean read with no tamper must be Warn");
}

// ---------------------------------------------------------------------------
// Scale synthetic test: 1000-box issuance + pair + hash chain in < 5s
// ---------------------------------------------------------------------------

#[test]
fn test_scale_1000_boxes() {
    let start = std::time::Instant::now();
    let lot_id = Uuid::now_v7();

    for i in 0..1000u32 {
        // Issuance
        let tag = issue_tag(IssueTagRequest {
            manufacturer: "NXP".to_owned(),
            lot_id,
            chip_family: "UCODE9".to_owned(),
            tag_secret_hsm_key_id: format!("hsm-slot-{i}"),
        })
        .expect("issuance must succeed");

        // Pair
        let box_body_id = Uuid::now_v7();
        let paired = pair_to_box(
            tag,
            PairToBoxRequest {
                box_body_id,
                box_lot_id: lot_id,
                box_body_epc: vec![
                    0x30,
                    0x34,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    (i >> 8) as u8,
                    (i & 0xFF) as u8,
                ],
            },
        )
        .expect("pairing must succeed");

        // HMAC read
        let secret = format!("secret-{i:032}").into_bytes();
        let nonce: Vec<u8> = (0..16).map(|j| (i as u8).wrapping_add(j)).collect();
        let response =
            HmacChallenger::compute_response(&secret, &nonce).expect("HMAC compute must succeed");
        let valid = HmacChallenger::verify_response(&secret, &nonce, &response)
            .expect("HMAC verify must succeed");
        assert!(valid);

        // One transition (MANUFACTURED -> PRINTER)
        let t = make_transition(
            paired.tag_id,
            CoverState::Manufactured,
            CoverState::Printer,
            vec![],
        );
        verify_chain(&[t]).expect("chain must verify");
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 5,
        "1000-box sequence must complete in < 5s; took {elapsed:?}"
    );
}

// ---------------------------------------------------------------------------
// Additional edge-case tests (to reach 30+)
// ---------------------------------------------------------------------------

#[test]
fn test_cover_state_all_main_has_15_entries() {
    assert_eq!(CoverState::all_main().len(), 15);
}

#[test]
fn test_cover_state_destroyed_is_terminal() {
    assert!(CoverState::Destroyed.is_terminal());
}

#[test]
fn test_cover_state_quarantine_is_terminal() {
    assert!(CoverState::Quarantine.is_terminal());
}

#[test]
fn test_cover_state_manufactured_not_terminal() {
    assert!(!CoverState::Manufactured.is_terminal());
}

#[test]
fn test_reader_type_round_trip() {
    for rt in [
        ReaderType::WarehouseGate,
        ReaderType::TransitCheckpoint,
        ReaderType::StationArrival,
        ReaderType::KitHandheld,
        ReaderType::RoDesktop,
        ReaderType::ArchiveGate,
        ReaderType::AirlineCargo,
        ReaderType::MfaCustoms,
    ] {
        let s = rt.as_str();
        let parsed = ReaderType::from_str(s).expect("round-trip must succeed");
        assert_eq!(rt, parsed, "from_str(as_str()) must be identity for {s}");
    }
}

#[test]
fn test_cover_state_round_trip() {
    for state in CoverState::all_main() {
        let s = state.as_str();
        let parsed = CoverState::from_str(s).expect("round-trip must succeed");
        assert_eq!(state, parsed, "from_str(as_str()) must be identity for {s}");
    }
}

#[test]
fn test_hash_chain_long_chain_verifies() {
    let tag_id = Uuid::now_v7();
    let states = [
        (CoverState::Manufactured, CoverState::Printer),
        (CoverState::Printer, CoverState::Warehouse),
        (CoverState::Warehouse, CoverState::Dispatched),
        (CoverState::Dispatched, CoverState::InTransit),
        (CoverState::InTransit, CoverState::StationReceived),
    ];

    let mut chain = Vec::new();
    let mut prev: Vec<u8> = vec![];
    for (from, to) in &states {
        let t = make_transition(tag_id, from.clone(), to.clone(), prev.clone());
        prev = t.this_hash.clone();
        chain.push(t);
    }

    verify_chain(&chain).expect("5-step chain must verify");
}

#[test]
fn test_tamper_alert_has_uuid7_id() {
    let alert = build_alert(RaiseAlertRequest {
        tag_id: Uuid::now_v7(),
        triggering_read: None,
        expected_state: None,
        observed_state: None,
        severity: Severity::Info,
    });
    assert_eq!(
        alert.alert_id.get_version_num(),
        7,
        "alert_id must be UUID v7"
    );
}
