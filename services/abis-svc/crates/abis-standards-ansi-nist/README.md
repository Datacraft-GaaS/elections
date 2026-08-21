# eel-abis-standards-ansi-nist

Rust codec for **ANSI/NIST-ITL 1-2011** (NIST SP 500-290 Edition 3) biometric
transaction files.

## Standard

ANSI/NIST-ITL 1-2011 defines a binary interchange format for biometric data
used by law-enforcement and civil-identity systems. Each transaction contains
one or more typed records, where Record Type 1 is a mandatory header and types
2-99 carry biometric payload.

## Encoding details

All records are ASCII field-tagged. Separator bytes follow the NIST spec:

| Byte | Value | Role |
|------|-------|------|
| FS   | 0x1C  | Record delimiter |
| US   | 0x1F  | Field delimiter |
| GS   | 0x1D  | Subfield delimiter (used within values) |
| RS   | 0x1E  | Item delimiter (used within subfields) |

Field tag format: `RRR.NNN:value` where `RRR` is the record type and `NNN` is
the 3-digit zero-padded field number.

Field `T.001` carries the total record length in ASCII decimal (self-inclusive).
Field `T.002` carries the Image Designation Character (IDC).

Binary field values (images, BDB payloads) are hex-encoded so that separator
bytes never appear inside a field value, guaranteeing byte-identical round-trips.

## Record type coverage

| Type | Name | Fields parsed |
|------|------|---------------|
| 1 | Transaction information | TOT, DAT, PRY, DAI, ORI, TCN |
| 2 | User-defined text | UDF |
| 4 | High-res grayscale fingerprint | ISR, HLL, VLL, GCA, image |
| 9 | Minutiae (EFS) | IMP, FMT, minutiae points |
| 10 | Photographic body part | ISR, HLL, VLL, SLC, image |
| 13 | Variable-res latent | ISR, HLL, VLL, image |
| 14 | Variable-res fingerprint | ISR, HLL, VLL, image |
| 15 | Palmprint | ISR, HLL, VLL, image |
| 17 | Iris | ISR, HLL, VLL, image |
| 99 | CBEFF biometric | format_owner, format_type, BDB |
| 3-8, 11-12, 16, 18-22, 98 | Parseable as raw records | tag/value pairs |

## Usage

```rust
use abis_standards_ansi_nist::{
    NistRecord, NistTransaction, RecordType,
    types::{Minutiae, MinutiaePoint, TransactionInfo},
};

// Build a two-record transaction.
let info = TransactionInfo {
    tot: "CAR".to_string(),
    dat: "20260101".to_string(),
    pry: 4,
    dai: "DAI001".to_string(),
    ori: "ORI001".to_string(),
    tcn: "TCN0000001".to_string(),
};
let mut header = NistRecord::new(RecordType::Type1, 0);
header.fields = info.to_fields();

let min = Minutiae {
    imp: 0,
    fmt: "U".to_string(),
    minutiae: vec![MinutiaePoint { x: 100, y: 200, theta: 45, quality: 80, mtype: 1 }],
};
let mut fp_rec = NistRecord::new(RecordType::Type9, 1);
fp_rec.fields = min.to_fields();

let mut tx = NistTransaction::new();
tx.push_record(header);
tx.push_record(fp_rec);

let bytes = tx.serialize();
let tx2 = NistTransaction::parse(&bytes).unwrap();
assert_eq!(tx, tx2); // byte-identical round-trip
```

## Interoperability

The codec guarantees byte-identical parse-serialize-parse round-trips for all
record types listed above. The hex-encoding of binary fields means the wire
format is not binary-identical to native ANSI/NIST files produced by other
tools; it is intended as an internal interchange format for the `eel` ABIS
pipeline.
