# eel-abis-standards-ebts

Rust codec for **FBI EBTS 10.0** (Electronic Biometric Transmission
Specification) transactions and responses.

## Standard

EBTS 10.0 extends ANSI/NIST-ITL 1-2011 with FBI-specific Type-of-Transaction
(TOT) codes and wire-protocol conventions used between law-enforcement agencies
and the NGI (Next Generation Identification) system.

## TOT code table

| Code | Description |
|------|-------------|
| CAR  | Criminal 10-print Array (arrest submission) |
| SRE  | Search Result Entry |
| NFAP | Non-Fingerprint Array Print |
| IRQ  | Identity Request |
| ULF  | Unsolicited Latent Fingerprint |
| ULM  | Unsolicited Latent with Minutiae |
| LFS  | Latent Fingerprint Search |
| LFFS | Latent Fingerprint with Feature Set |
| ERM  | Error Response Message |
| ERU  | Error Response Update |
| ENU  | Electronic Notifier Update |
| ENE  | Electronic Notifier Entry |
| IDV  | Identity Verification |

## Binary layout

### Transaction

```
Type-1 header:
  [1]  record_type = 1
  [1]  idc = 0
  [2]  header_len: u16 (be)
  [N]  ASCII content: "TOT:<tot>\x1fDAI:<dai>\x1fORI:<ori>\x1fTCN:<tcn>"

Per additional record:
  [1]  record_type (ANSI/NIST type number, e.g. 4)
  [1]  idc
  [4]  data_len: u32 (be)
  [M]  hex-encoded record data
```

### Response

Same as transaction header but with `STATUS:ACCEPT`, `STATUS:REJECT:<msg>`,
or `STATUS:ERROR:<msg>` appended after the TCN field.

## Type-1 IDC linking

The Type-1 header record always has IDC=0 and is always the first record in
the serialised output. Additional records carry their own IDC values linking
biometric data (fingerprint image + minutiae sharing the same IDC belong to
the same finger).

## Coverage matrix

| Feature | Supported |
|---------|-----------|
| All 13 TOT codes | Yes |
| Transaction serialize/parse | Yes |
| Response serialize/parse | Yes |
| Accept/Reject/Error status | Yes |
| IDC-linked records | Yes |
| Arbitrary record type/data | Yes |

## Usage

```rust
use abis_standards_ebts::{EbtsTransaction, TypeOfTransaction};

let mut tx = EbtsTransaction::build(
    TypeOfTransaction::CAR,
    "FBIAD001",
    "FBIORI001",
    "TCN20260821001",
);
// Add a Type-4 fingerprint image record with IDC=1.
tx.add_record(1, 4, vec![0xDE, 0xAD, 0xBE, 0xEF]);

let bytes = tx.serialize();
let tx2 = EbtsTransaction::parse(&bytes).unwrap();
assert_eq!(tx, tx2);
```
