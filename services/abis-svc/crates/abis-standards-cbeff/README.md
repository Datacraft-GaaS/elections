# eel-abis-standards-cbeff

Rust codec for **ISO/IEC 19785-1 CBEFF v3** (Common Biometric Exchange
Formats Framework) containers.

## Standard

CBEFF defines a three-part binary envelope for biometric data:

- **SBH** (Standard Biometric Header): metadata identifying format, device, quality.
- **BDB** (Biometric Data Block): the opaque biometric payload.
- **SB** (Security Block, optional): cryptographic protection.

## Binary layout

All multi-byte integers are big-endian.

```
[4]  magic: 0xCB 0xEF 0x00 0x01
[N]  SBH (variable length; see SBH field table below)
[4]  bdb_length: u32
[M]  bdb_data
[1]  sb_present: 0=absent, 1=present
if sb_present:
  [2]  algorithm_id: u16
  [4]  sb_length: u32
  [K]  sb_data
```

### SBH field table

| Field | Size | Description |
|-------|------|-------------|
| format_owner | 2 | IBIA-registered format owner ID |
| format_type | 2 | Owner-defined format type |
| creation_date | 8 | ASCII YYYYMMDD |
| vp_flag | 1 | 1 if validity_period present |
| validity_period | 16 (optional) | Start+end YYYYMMDD |
| biometric_type | 4 | ISO/IEC 19785-1 Table 3 flags |
| biometric_subtype | 1 | Subtype within biometric_type |
| product_id | 2 | Vendor product identifier |
| capture_device_id | 2 | Capture device identifier |
| quality_count | 2 | Number of quality records |
| quality_records | 5*N | algorithm_vendor + algorithm_product + quality_score |

## Format Owner Registry

| ID | Owner |
|----|-------|
| 0x0001 | ISO/IEC JTC1 SC37 |
| 0x0101 | FBI |
| 0x0201 | NIST |
| 0x0301 | Interpol |

## Nested containers

The `CbeffContainer::is_nested()` method detects when the BDB itself contains
another CBEFF container (BDB data starts with magic `0xCB 0xEF 0x00 0x01`).
`unwrap_nested()` parses and returns the inner container.

## Usage

```rust
use abis_standards_cbeff::{
    BiometricDataBlock, CbeffContainer, QualityRecord, StandardBiometricHeader,
};

let sbh = StandardBiometricHeader {
    format_owner: 0x0101,
    format_type:  0x0007,
    creation_date: *b"20260821",
    validity_period: None,
    biometric_type: 0x0000_0008,
    biometric_subtype: 0,
    product_id: 1,
    capture_device_id: 2,
    quality_records: vec![
        QualityRecord { algorithm_vendor: 0x0101, algorithm_product: 0x0002, quality_score: 90 },
    ],
};

let bdb = BiometricDataBlock::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
let container = CbeffContainer::new(sbh, bdb, None);

let bytes = container.serialize();
let parsed = CbeffContainer::parse(&bytes).unwrap();
assert_eq!(parsed, container);
```
