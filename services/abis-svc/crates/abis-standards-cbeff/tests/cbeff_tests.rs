use abis_standards_cbeff::{
    BiometricDataBlock, CbeffContainer, CbeffError, FormatRegistry, QualityRecord, SecurityBlock,
    StandardBiometricHeader,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_sbh(format_owner: u16, format_type: u16) -> StandardBiometricHeader {
    StandardBiometricHeader {
        format_owner,
        format_type,
        creation_date: *b"20260821",
        validity_period: None,
        biometric_type: 0x0000_0008,
        biometric_subtype: 0,
        product_id: 1,
        capture_device_id: 2,
        quality_records: vec![],
    }
}

fn make_container(payload: Vec<u8>) -> CbeffContainer {
    CbeffContainer::new(
        make_sbh(0x0101, 0x0007),
        BiometricDataBlock::new(payload),
        None,
    )
}

fn roundtrip(c: &CbeffContainer) -> CbeffContainer {
    CbeffContainer::parse(&c.serialize()).expect("roundtrip failed")
}

// ---------------------------------------------------------------------------
// SBH mandatory fields round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_sbh_mandatory_fields_roundtrip() {
    let sbh = make_sbh(0x0201, 0x0003);
    let c = CbeffContainer::new(sbh.clone(), BiometricDataBlock::new(vec![1, 2, 3]), None);
    let c2 = roundtrip(&c);
    assert_eq!(c2.sbh.format_owner, 0x0201);
    assert_eq!(c2.sbh.format_type, 0x0003);
    assert_eq!(c2.sbh.creation_date, *b"20260821");
    assert_eq!(c2, c);
}

// ---------------------------------------------------------------------------
// Quality records
// ---------------------------------------------------------------------------

#[test]
fn test_quality_record_roundtrip() {
    let mut sbh = make_sbh(0x0101, 0x0001);
    sbh.quality_records = vec![
        QualityRecord {
            algorithm_vendor: 0x0101,
            algorithm_product: 0x0002,
            quality_score: 85,
        },
        QualityRecord {
            algorithm_vendor: 0x0201,
            algorithm_product: 0x0003,
            quality_score: 72,
        },
    ];
    let c = CbeffContainer::new(sbh, BiometricDataBlock::new(vec![0xAA, 0xBB]), None);
    let c2 = roundtrip(&c);
    assert_eq!(c2.sbh.quality_records.len(), 2);
    assert_eq!(c2.sbh.quality_records[0].quality_score, 85);
    assert_eq!(c2.sbh.quality_records[1].quality_score, 72);
}

// ---------------------------------------------------------------------------
// Biometric type and subtype
// ---------------------------------------------------------------------------

#[test]
fn test_biometric_type_preserved() {
    let mut sbh = make_sbh(0x0001, 0x0001);
    sbh.biometric_type = 0xDEAD_BEEF;
    sbh.biometric_subtype = 0x7F;
    let c = CbeffContainer::new(sbh, BiometricDataBlock::new(vec![1]), None);
    let c2 = roundtrip(&c);
    assert_eq!(c2.sbh.biometric_type, 0xDEAD_BEEF);
    assert_eq!(c2.sbh.biometric_subtype, 0x7F);
}

// ---------------------------------------------------------------------------
// Product and device IDs
// ---------------------------------------------------------------------------

#[test]
fn test_product_and_capture_device_id_preserved() {
    let mut sbh = make_sbh(0x0301, 0x0002);
    sbh.product_id = 0xFFFF;
    sbh.capture_device_id = 0xABCD;
    let c = CbeffContainer::new(sbh, BiometricDataBlock::new(vec![0]), None);
    let c2 = roundtrip(&c);
    assert_eq!(c2.sbh.product_id, 0xFFFF);
    assert_eq!(c2.sbh.capture_device_id, 0xABCD);
}

// ---------------------------------------------------------------------------
// Creation date
// ---------------------------------------------------------------------------

#[test]
fn test_creation_date_bytes_preserved() {
    let mut sbh = make_sbh(0x0101, 0x0007);
    sbh.creation_date = *b"20991231";
    let c = CbeffContainer::new(sbh, BiometricDataBlock::new(vec![1, 2]), None);
    let c2 = roundtrip(&c);
    assert_eq!(c2.sbh.creation_date, *b"20991231");
}

// ---------------------------------------------------------------------------
// Validity period
// ---------------------------------------------------------------------------

#[test]
fn test_validity_period_optional_absent() {
    let c = make_container(vec![0x01]);
    let c2 = roundtrip(&c);
    assert!(c2.sbh.validity_period.is_none());
}

#[test]
fn test_validity_period_present_preserved() {
    let mut sbh = make_sbh(0x0101, 0x0001);
    let vp: [u8; 16] = *b"2026010120271231";
    sbh.validity_period = Some(vp);
    let c = CbeffContainer::new(sbh, BiometricDataBlock::new(vec![1]), None);
    let c2 = roundtrip(&c);
    assert_eq!(c2.sbh.validity_period, Some(vp));
}

// ---------------------------------------------------------------------------
// Security block -- optional
// ---------------------------------------------------------------------------

#[test]
fn test_security_block_absent() {
    let c = make_container(vec![0xAA, 0xBB, 0xCC]);
    let bytes = c.serialize();
    let c2 = CbeffContainer::parse(&bytes).expect("parse");
    assert!(c2.sb.is_none());
}

#[test]
fn test_security_block_present_roundtrip() {
    let sb = SecurityBlock::new(0x0001, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    let c = CbeffContainer::new(
        make_sbh(0x0101, 0x0001),
        BiometricDataBlock::new(vec![1, 2]),
        Some(sb),
    );
    let c2 = roundtrip(&c);
    let sb2 = c2.sb.as_ref().expect("sb must be present");
    assert_eq!(sb2.algorithm_id, 0x0001);
    assert_eq!(sb2.data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
}

// ---------------------------------------------------------------------------
// Invalid magic
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_magic_returns_error() {
    let bad = b"\x00\x00\x00\x00hello world";
    let err = CbeffContainer::parse(bad).expect_err("should fail");
    assert!(matches!(err, CbeffError::InvalidMagic));
}

#[test]
fn test_truncated_input_returns_error() {
    let bytes = make_container(vec![1, 2, 3]).serialize();
    // Keep only magic + 2 bytes
    let truncated = &bytes[..6];
    let err = CbeffContainer::parse(truncated).expect_err("should fail");
    assert!(matches!(
        err,
        CbeffError::InvalidMagic | CbeffError::InvalidLength
    ));
}

// ---------------------------------------------------------------------------
// BDB edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_bdb_zero_length() {
    let c = make_container(vec![]);
    let c2 = roundtrip(&c);
    assert!(c2.bdb.data.is_empty());
}

#[test]
fn test_bdb_larger_than_64kb() {
    let data: Vec<u8> = (0..70_000).map(|i: usize| (i % 256) as u8).collect();
    let c = make_container(data.clone());
    let c2 = roundtrip(&c);
    assert_eq!(c2.bdb.data.len(), 70_000);
    assert_eq!(c2.bdb.data, data);
}

// ---------------------------------------------------------------------------
// Nested CBEFF
// ---------------------------------------------------------------------------

#[test]
fn test_nested_cbeff_is_detected() {
    let inner = make_container(vec![0xFF, 0xFE]);
    let inner_bytes = inner.serialize();

    let outer = make_container(inner_bytes.clone());
    assert!(
        outer.is_nested(),
        "outer bdb starts with CBEFF magic -- must be detected as nested"
    );
}

#[test]
fn test_nested_cbeff_unwrap() {
    let inner = make_container(vec![0xAA, 0xBB, 0xCC]);
    let inner_bytes = inner.serialize();

    let outer = make_container(inner_bytes);
    let unwrapped = outer.unwrap_nested().expect("unwrap nested");
    assert_eq!(unwrapped.bdb.data, vec![0xAA, 0xBB, 0xCC]);
}

#[test]
fn test_nested_unwrap_fails_when_not_nested() {
    let c = make_container(vec![0x00, 0x01, 0x02]);
    assert!(!c.is_nested());
    let err = c.unwrap_nested().expect_err("must fail");
    assert!(matches!(err, CbeffError::MissingField(_)));
}

// ---------------------------------------------------------------------------
// Format registry
// ---------------------------------------------------------------------------

#[test]
fn test_format_registry_known_owners() {
    let reg = FormatRegistry::new();
    assert_eq!(reg.lookup_owner(0x0001), Some("ISO/IEC JTC1 SC37"));
    assert_eq!(reg.lookup_owner(0x0101), Some("FBI"));
    assert_eq!(reg.lookup_owner(0x0201), Some("NIST"));
    assert_eq!(reg.lookup_owner(0x0301), Some("Interpol"));
}

#[test]
fn test_format_registry_unknown_owner_returns_none() {
    let reg = FormatRegistry::new();
    assert!(reg.lookup_owner(0xFFFF).is_none());
    assert!(reg.lookup_owner(0x0000).is_none());
}
