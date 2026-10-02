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

use abis_standards_ebts::{EbtsResponse, EbtsTransaction, ResponseStatus, TypeOfTransaction};
use std::str::FromStr;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn roundtrip(tx: &EbtsTransaction) -> EbtsTransaction {
    let bytes = tx.serialize();
    EbtsTransaction::parse(&bytes).expect("parse failed")
}

fn simple_tx(tot: TypeOfTransaction) -> EbtsTransaction {
    EbtsTransaction::build(tot, "DAI001", "ORI001", "TCN0000001")
}

// ---------------------------------------------------------------------------
// TOT display and from_str
// ---------------------------------------------------------------------------

#[test]
fn test_tot_display_all_codes() {
    let cases = [
        (TypeOfTransaction::CAR, "CAR"),
        (TypeOfTransaction::SRE, "SRE"),
        (TypeOfTransaction::NFAP, "NFAP"),
        (TypeOfTransaction::IRQ, "IRQ"),
        (TypeOfTransaction::ULF, "ULF"),
        (TypeOfTransaction::ULM, "ULM"),
        (TypeOfTransaction::LFS, "LFS"),
        (TypeOfTransaction::LFFS, "LFFS"),
        (TypeOfTransaction::ERM, "ERM"),
        (TypeOfTransaction::ERU, "ERU"),
        (TypeOfTransaction::ENU, "ENU"),
        (TypeOfTransaction::ENE, "ENE"),
        (TypeOfTransaction::IDV, "IDV"),
    ];
    for (tot, expected) in cases {
        assert_eq!(tot.to_string(), expected);
    }
}

#[test]
fn test_tot_from_str_all_codes() {
    let codes = [
        "CAR", "SRE", "NFAP", "IRQ", "ULF", "ULM", "LFS", "LFFS", "ERM", "ERU", "ENU", "ENE", "IDV",
    ];
    for code in codes {
        let parsed = TypeOfTransaction::from_str(code).expect(code);
        assert_eq!(parsed.to_string(), code);
    }
}

#[test]
fn test_tot_from_str_unknown_returns_error() {
    let err = TypeOfTransaction::from_str("XYZ").expect_err("should fail");
    let msg = format!("{err}");
    assert!(msg.contains("XYZ"));
}

// ---------------------------------------------------------------------------
// Transaction round-trips
// ---------------------------------------------------------------------------

#[test]
fn test_car_transaction_roundtrip() {
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::CAR)),
        simple_tx(TypeOfTransaction::CAR)
    );
}

#[test]
fn test_sre_transaction_roundtrip() {
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::SRE)),
        simple_tx(TypeOfTransaction::SRE)
    );
}

#[test]
fn test_nfap_transaction_roundtrip() {
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::NFAP)),
        simple_tx(TypeOfTransaction::NFAP)
    );
}

#[test]
fn test_irq_transaction_roundtrip() {
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::IRQ)),
        simple_tx(TypeOfTransaction::IRQ)
    );
}

#[test]
fn test_ulf_ulm_roundtrips() {
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::ULF)),
        simple_tx(TypeOfTransaction::ULF)
    );
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::ULM)),
        simple_tx(TypeOfTransaction::ULM)
    );
}

#[test]
fn test_lfs_lffs_roundtrips() {
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::LFS)),
        simple_tx(TypeOfTransaction::LFS)
    );
    assert_eq!(
        roundtrip(&simple_tx(TypeOfTransaction::LFFS)),
        simple_tx(TypeOfTransaction::LFFS)
    );
}

#[test]
fn test_unknown_tot_string_returns_error() {
    // Build bytes with a deliberately invalid TOT code.
    let tx = EbtsTransaction::build(TypeOfTransaction::CAR, "D", "O", "T");
    let mut bytes = tx.serialize();
    // Replace "CAR" with "ZZZ" in the header content.
    let car_pos = bytes.windows(3).position(|w| w == b"CAR");
    if let Some(i) = car_pos {
        bytes[i] = b'Z';
        bytes[i + 1] = b'Z';
        bytes[i + 2] = b'Z';
    }
    let err = EbtsTransaction::parse(&bytes).expect_err("should fail on unknown TOT");
    let msg = format!("{err}");
    assert!(
        msg.contains("ZZZ") || msg.contains("unknown"),
        "error: {msg}"
    );
}

// ---------------------------------------------------------------------------
// add_record / IDC linking
// ---------------------------------------------------------------------------

#[test]
fn test_add_record_links_idc_correctly() {
    let mut tx = EbtsTransaction::build(TypeOfTransaction::CAR, "DAI001", "ORI001", "TCN001");
    tx.add_record(1, 4, vec![0xDE, 0xAD]);
    tx.add_record(1, 9, vec![0xBE, 0xEF]);

    let tx2 = roundtrip(&tx);
    assert_eq!(tx2.records.len(), 2);
    assert_eq!(tx2.records[0].idc, 1);
    assert_eq!(tx2.records[1].idc, 1);
    assert_eq!(tx2.records[0].record_type, 4);
    assert_eq!(tx2.records[1].record_type, 9);
}

// ---------------------------------------------------------------------------
// Type-1 IDC=0 always present
// ---------------------------------------------------------------------------

#[test]
fn test_type1_idc0_always_present_in_serialised_output() {
    let tx = simple_tx(TypeOfTransaction::CAR);
    let bytes = tx.serialize();

    // First 2 bytes must be [1, 0] = record_type=1, idc=0
    assert_eq!(bytes[0], 1, "first byte must be record_type=1");
    assert_eq!(bytes[1], 0, "second byte must be idc=0");
}

// ---------------------------------------------------------------------------
// Record data preserved
// ---------------------------------------------------------------------------

#[test]
fn test_record_data_with_arbitrary_bytes_roundtrip() {
    let data: Vec<u8> = (0u8..=255).collect();
    let mut tx = EbtsTransaction::build(TypeOfTransaction::IDV, "DAI", "ORI", "TCN");
    tx.add_record(2, 14, data.clone());

    let tx2 = roundtrip(&tx);
    assert_eq!(tx2.records[0].data, data);
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

#[test]
fn test_response_parse_accept_status() {
    let resp = EbtsResponse {
        tot: TypeOfTransaction::SRE,
        tcn: "TCN001".to_string(),
        status: ResponseStatus::Accept,
        records: vec![],
    };
    let bytes = resp.serialize();
    let parsed = EbtsResponse::parse(&bytes).expect("parse");
    assert_eq!(parsed.status, ResponseStatus::Accept);
    assert_eq!(parsed.tot, TypeOfTransaction::SRE);
    assert_eq!(parsed.tcn, "TCN001");
}

#[test]
fn test_response_parse_reject_status() {
    let resp = EbtsResponse {
        tot: TypeOfTransaction::CAR,
        tcn: "TCN002".to_string(),
        status: ResponseStatus::Reject("fingerprint quality insufficient".to_string()),
        records: vec![],
    };
    let bytes = resp.serialize();
    let parsed = EbtsResponse::parse(&bytes).expect("parse");
    match parsed.status {
        ResponseStatus::Reject(msg) => assert!(msg.contains("quality")),
        other => panic!("expected Reject, got {other:?}"),
    }
}

#[test]
fn test_response_parse_error_status() {
    let resp = EbtsResponse {
        tot: TypeOfTransaction::ERM,
        tcn: "TCN003".to_string(),
        status: ResponseStatus::Error("internal server error".to_string()),
        records: vec![],
    };
    let bytes = resp.serialize();
    let parsed = EbtsResponse::parse(&bytes).expect("parse");
    match parsed.status {
        ResponseStatus::Error(msg) => assert!(msg.contains("internal")),
        other => panic!("expected Error, got {other:?}"),
    }
}
