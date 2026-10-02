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

use abis_standards_ansi_nist::{
    types::{
        CbeffBiometric, HighResGrayscaleFp, Iris, Minutiae, MinutiaePoint, Palmprint,
        PhotographicBodyPart, TransactionInfo, UserDefinedText, VariableResFp, VariableResLatent,
    },
    NistField, NistRecord, NistTransaction, RecordType,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_type1(tot: &str) -> NistRecord {
    let info = TransactionInfo {
        tot: tot.to_string(),
        dat: "20260101".to_string(),
        pry: 4,
        dai: "DAI001".to_string(),
        ori: "ORI001".to_string(),
        tcn: "TCN0000001".to_string(),
    };
    let mut rec = NistRecord::new(RecordType::Type1, 0);
    rec.fields = info.to_fields();
    rec
}

fn roundtrip(tx: &NistTransaction) -> NistTransaction {
    let bytes = tx.serialize();
    NistTransaction::parse(&bytes).expect("parse failed")
}

// ---------------------------------------------------------------------------
// RecordType
// ---------------------------------------------------------------------------

#[test]
fn test_record_type_as_u8_known_values() {
    assert_eq!(RecordType::Type1.as_u8(), 1);
    assert_eq!(RecordType::Type9.as_u8(), 9);
    assert_eq!(RecordType::Type22.as_u8(), 22);
    assert_eq!(RecordType::Type98.as_u8(), 98);
    assert_eq!(RecordType::Type99.as_u8(), 99);
}

#[test]
fn test_record_type_from_u8_all_known() {
    for n in [
        1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 98, 99,
    ] {
        let rt = RecordType::from_u8(n).expect("should be known");
        assert_eq!(rt.as_u8(), n);
    }
}

#[test]
fn test_record_type_from_u8_unknown_returns_error() {
    let err = RecordType::from_u8(50).expect_err("should fail");
    let msg = format!("{err}");
    assert!(msg.contains("50"), "error should mention the value");
}

#[test]
fn test_record_type_from_u8_zero_unknown() {
    assert!(RecordType::from_u8(0).is_err());
}

// ---------------------------------------------------------------------------
// Type1 round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_type1_roundtrip() {
    let original = TransactionInfo {
        tot: "CAR".to_string(),
        dat: "20260821".to_string(),
        pry: 7,
        dai: "DEST_AGENCY".to_string(),
        ori: "ORIG_AGENCY".to_string(),
        tcn: "TCN_ABC_123".to_string(),
    };
    let fields = original.to_fields();
    let parsed = TransactionInfo::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

#[test]
fn test_transaction_roundtrip_type1_only() {
    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("CAR"));
    assert_eq!(roundtrip(&tx), tx);
}

// ---------------------------------------------------------------------------
// Golden fixture: build known Type1 bytes, parse, verify TOT field
// ---------------------------------------------------------------------------

#[test]
fn test_golden_type1_tot_field() {
    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("SRE"));
    let bytes = tx.serialize();

    // Bytes must contain the ASCII text "SRE"
    assert!(
        bytes.windows(3).any(|w| w == b"SRE"),
        "serialised bytes must contain TOT value 'SRE'"
    );

    let parsed = NistTransaction::parse(&bytes).expect("parse");
    let rec = &parsed.records[0];
    let info = TransactionInfo::parse(&rec.fields).expect("type1 parse");
    assert_eq!(info.tot, "SRE");
}

// ---------------------------------------------------------------------------
// Type2 round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_type2_roundtrip() {
    let original = UserDefinedText {
        udf: b"hello world".to_vec(),
    };
    let fields = original.to_fields();
    let parsed = UserDefinedText::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

#[test]
fn test_type2_in_transaction_roundtrip() {
    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("CAR"));

    let udt = UserDefinedText {
        udf: b"custom-data-payload".to_vec(),
    };
    let mut rec = NistRecord::new(RecordType::Type2, 1);
    rec.fields = udt.to_fields();
    tx.push_record(rec);

    assert_eq!(roundtrip(&tx), tx);
}

// ---------------------------------------------------------------------------
// Type4 image data preserved
// ---------------------------------------------------------------------------

#[test]
fn test_type4_image_data_roundtrip() {
    let image_data: Vec<u8> = (0u8..=255).collect();
    let original = HighResGrayscaleFp {
        isr: 10,
        hll: 512,
        vll: 512,
        gca: 1,
        image_data: image_data.clone(),
    };
    let fields = original.to_fields();
    let parsed = HighResGrayscaleFp::parse(&fields).expect("parse");
    assert_eq!(parsed.image_data, image_data);
    assert_eq!(parsed, original);
}

#[test]
fn test_type4_in_transaction_image_preserved() {
    let image_data: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x1F, 0x1C];
    let fp = HighResGrayscaleFp {
        isr: 5,
        hll: 32,
        vll: 32,
        gca: 0,
        image_data: image_data.clone(),
    };
    let mut rec = NistRecord::new(RecordType::Type4, 1);
    rec.fields = fp.to_fields();

    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("CAR"));
    tx.push_record(rec);

    let tx2 = roundtrip(&tx);
    let fp2 = HighResGrayscaleFp::parse(&tx2.records[1].fields).expect("parse");
    assert_eq!(fp2.image_data, image_data);
}

// ---------------------------------------------------------------------------
// Type9 minutiae round-trip with 5 points
// ---------------------------------------------------------------------------

#[test]
fn test_type9_roundtrip_5_minutiae() {
    let pts = vec![
        MinutiaePoint {
            x: 100,
            y: 200,
            theta: 45,
            quality: 80,
            mtype: 1,
        },
        MinutiaePoint {
            x: 150,
            y: 250,
            theta: 90,
            quality: 75,
            mtype: 2,
        },
        MinutiaePoint {
            x: 200,
            y: 300,
            theta: 135,
            quality: 90,
            mtype: 1,
        },
        MinutiaePoint {
            x: 50,
            y: 50,
            theta: 0,
            quality: 60,
            mtype: 3,
        },
        MinutiaePoint {
            x: 400,
            y: 400,
            theta: 270,
            quality: 85,
            mtype: 2,
        },
    ];
    let original = Minutiae {
        imp: 0,
        fmt: "U".to_string(),
        minutiae: pts,
    };
    let fields = original.to_fields();
    let parsed = Minutiae::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
    assert_eq!(parsed.minutiae.len(), 5);
}

#[test]
fn test_type9_minutiae_count_preserved_in_transaction() {
    let pts: Vec<MinutiaePoint> = (0..10)
        .map(|i| MinutiaePoint {
            x: i * 10,
            y: i * 20,
            theta: i * 30,
            quality: 70,
            mtype: 1,
        })
        .collect();
    let min = Minutiae {
        imp: 2,
        fmt: "M".to_string(),
        minutiae: pts,
    };
    let mut rec = NistRecord::new(RecordType::Type9, 1);
    rec.fields = min.to_fields();

    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("NFAP"));
    tx.push_record(rec);

    let tx2 = roundtrip(&tx);
    let min2 = Minutiae::parse(&tx2.records[1].fields).expect("parse");
    assert_eq!(min2.minutiae.len(), 10);
}

#[test]
fn test_type9_minutiae_point_values_preserved() {
    let pt = MinutiaePoint {
        x: 1234,
        y: 5678,
        theta: 359,
        quality: 99,
        mtype: 4,
    };
    let min = Minutiae {
        imp: 1,
        fmt: "U".to_string(),
        minutiae: vec![pt.clone()],
    };
    let fields = min.to_fields();
    let parsed = Minutiae::parse(&fields).expect("parse");
    assert_eq!(parsed.minutiae[0], pt);
}

// ---------------------------------------------------------------------------
// Type10
// ---------------------------------------------------------------------------

#[test]
fn test_type10_roundtrip() {
    let original = PhotographicBodyPart {
        isr: 3,
        hll: 640,
        vll: 480,
        slc: 1,
        image_data: vec![0xFF, 0xD8, 0xFF, 0xE0],
    };
    let fields = original.to_fields();
    let parsed = PhotographicBodyPart::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

// ---------------------------------------------------------------------------
// Type13
// ---------------------------------------------------------------------------

#[test]
fn test_type13_roundtrip() {
    let original = VariableResLatent {
        isr: 2,
        hll: 800,
        vll: 600,
        image_data: vec![0x89, 0x50, 0x4E, 0x47],
    };
    let fields = original.to_fields();
    let parsed = VariableResLatent::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

// ---------------------------------------------------------------------------
// Type14
// ---------------------------------------------------------------------------

#[test]
fn test_type14_roundtrip() {
    let original = VariableResFp {
        isr: 4,
        hll: 500,
        vll: 700,
        image_data: vec![0x00, 0x01, 0x02, 0x03, 0x1C, 0x1F],
    };
    let fields = original.to_fields();
    let parsed = VariableResFp::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

// ---------------------------------------------------------------------------
// Type15
// ---------------------------------------------------------------------------

#[test]
fn test_type15_roundtrip() {
    let original = Palmprint {
        isr: 1,
        hll: 1000,
        vll: 800,
        image_data: vec![0xAB, 0xCD, 0xEF],
    };
    let fields = original.to_fields();
    let parsed = Palmprint::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

// ---------------------------------------------------------------------------
// Type17
// ---------------------------------------------------------------------------

#[test]
fn test_type17_roundtrip() {
    let original = Iris {
        isr: 2,
        hll: 640,
        vll: 480,
        image_data: vec![0x01, 0x02, 0x03, 0x04],
    };
    let fields = original.to_fields();
    let parsed = Iris::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

// ---------------------------------------------------------------------------
// Type99 CBEFF wrapping
// ---------------------------------------------------------------------------

#[test]
fn test_type99_roundtrip() {
    let original = CbeffBiometric {
        format_owner: 0x0101,
        format_type: 0x0007,
        bdb: vec![0xCB, 0xEF, 0x00, 0x01, 0xAA, 0xBB],
    };
    let fields = original.to_fields();
    let parsed = CbeffBiometric::parse(&fields).expect("parse");
    assert_eq!(parsed, original);
}

#[test]
fn test_type99_in_transaction_roundtrip() {
    let cbeff = CbeffBiometric {
        format_owner: 0x0201,
        format_type: 0x0001,
        bdb: (0u8..64).collect(),
    };
    let mut rec = NistRecord::new(RecordType::Type99, 2);
    rec.fields = cbeff.to_fields();

    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("IRQ"));
    tx.push_record(rec);

    assert_eq!(roundtrip(&tx), tx);
}

// ---------------------------------------------------------------------------
// Error cases
// ---------------------------------------------------------------------------

#[test]
fn test_parse_empty_bytes_returns_error() {
    let result = NistTransaction::parse(b"");
    assert!(result.is_err(), "empty input should fail");
}

#[test]
fn test_parse_truncated_input_returns_error() {
    // Valid record lacks FS terminator and has no fields parseable.
    let result = NistTransaction::parse(b"garbage-no-colon");
    assert!(result.is_err());
}

#[test]
fn test_parse_unknown_record_type_returns_error() {
    // Craft bytes that look like a record with record type 50.
    let fake = b"50.001:0010\x1f50.002:0\x1f50.003:val\x1c";
    let result = NistTransaction::parse(fake);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("50"), "error must name the unknown type");
}

// ---------------------------------------------------------------------------
// IDC linking
// ---------------------------------------------------------------------------

#[test]
fn test_idc_linking_across_records() {
    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("CAR"));

    let mut rec1 = NistRecord::new(RecordType::Type4, 1);
    rec1.fields = HighResGrayscaleFp {
        isr: 1,
        hll: 10,
        vll: 10,
        gca: 0,
        image_data: vec![0xAA],
    }
    .to_fields();

    let mut rec2 = NistRecord::new(RecordType::Type9, 1);
    rec2.fields = Minutiae {
        imp: 0,
        fmt: "U".to_string(),
        minutiae: vec![MinutiaePoint {
            x: 1,
            y: 2,
            theta: 3,
            quality: 4,
            mtype: 5,
        }],
    }
    .to_fields();

    tx.push_record(rec1);
    tx.push_record(rec2);

    let tx2 = roundtrip(&tx);
    // Both records share IDC = 1
    assert_eq!(tx2.records[1].idc, 1);
    assert_eq!(tx2.records[2].idc, 1);
}

// ---------------------------------------------------------------------------
// Separator bytes in output
// ---------------------------------------------------------------------------

#[test]
fn test_separator_bytes_present_in_output() {
    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("CAR"));
    let bytes = tx.serialize();

    // FS (0x1C) terminates each record
    assert!(bytes.contains(&0x1C_u8), "FS byte must be present");
    // US (0x1F) separates fields
    assert!(bytes.contains(&0x1F_u8), "US byte must be present");
}

// ---------------------------------------------------------------------------
// Multi-record transaction
// ---------------------------------------------------------------------------

#[test]
fn test_transaction_multiple_records_roundtrip() {
    let mut tx = NistTransaction::new();
    tx.push_record(make_type1("CAR"));

    for idc in 1u8..=3 {
        let fp = HighResGrayscaleFp {
            isr: idc,
            hll: 100,
            vll: 100,
            gca: 0,
            image_data: vec![idc; 16],
        };
        let mut rec = NistRecord::new(RecordType::Type4, idc);
        rec.fields = fp.to_fields();
        tx.push_record(rec);
    }

    let tx2 = roundtrip(&tx);
    assert_eq!(tx2.records.len(), 4);
    for (i, rec) in tx2.records[1..].iter().enumerate() {
        let fp = HighResGrayscaleFp::parse(&rec.fields).expect("parse");
        assert_eq!(fp.image_data, vec![(i + 1) as u8; 16]);
    }
}

#[test]
fn test_nist_field_missing_required_returns_error() {
    // Build a Type4 record with no image field (tag 999)
    let fields = vec![
        NistField::new(3, b"5"),
        NistField::new(4, b"512"),
        NistField::new(5, b"512"),
        NistField::new(6, b"1"),
        // tag 999 (image) intentionally omitted
    ];
    let result = HighResGrayscaleFp::parse(&fields);
    assert!(result.is_err());
}
