# eel-abis-standards-interpol

Rust types and message envelope implementation for INTERPOL IABIS (International Automated Biometric Identification System) communications.

## Overview

INTERPOL connects National Central Bureaus (NCBs) in 195 member countries for criminal intelligence exchange. This crate models the message envelope structure used to transmit biometric and criminal entity data between NCBs, with purpose codes, response windows, and record payloads aligned to NIST/ANSI messaging conventions.

## NCB Code System

Each member country's National Central Bureau is identified by a three-letter ASCII alpha code. The `NcbCode` type enforces this constraint at construction time:

```rust
let sender = NcbCode::new("KEN")?;  // Kenya NCB
let receiver = NcbCode::new("INT")?; // INTERPOL General Secretariat
```

Invalid codes are rejected with `InterpolError::InvalidNcbCode`:

- Length must be exactly 3 characters
- All characters must be ASCII alphabetic
- Stored in uppercase

## Message Envelope

The `InterpolEnvelope` wraps one or more biometric records for transmission:

```rust
let mut env = InterpolEnvelope::build(
    NcbCode::new("KEN")?,
    NcbCode::new("INT")?,
    MessagePurpose::CriminalSearch,
    ResponseWindow::Standard,
)?;
env.add_record(4, 1, fingerprint_bytes);

let wire = env.serialize();
let decoded = InterpolEnvelope::parse(&wire)?;
```

### Message Purposes

| Variant | Use case |
|---|---|
| `CriminalSearch` | Latent/ten-print search against criminal databases |
| `PersonSearch` | Missing persons, identity verification |
| `LostDocumentSearch` | Stolen/lost passport and travel document checks |
| `StolenVehicleSearch` | Vehicle identification number queries |

### Response Windows

| Variant | Hours | Use |
|---|---|---|
| `Immediate` | 0 | Urgent border control hits |
| `Standard` | 24 | Routine criminal intelligence |
| `Relaxed` | 72 | Background investigation support |

## Record Structure

Each `EnvelopeRecord` carries a NIST record type, an Image Data Container (IDC) identifier, and raw record bytes. The IDC links records within a transaction:

```rust
env.add_record(
    4,            // NIST Type-4 high-resolution grayscale fingerprint
    1,            // IDC 1
    image_bytes,
);
```

## Entity Linkage

Criminal entities and person records can be linked for intelligence sharing:

```rust
let crime = CrimeEntity {
    reference_number: "CR-2024-001".into(),
    crime_type: "Fraud".into(),
    date_occurred: "2024-01-15".into(),
    ncb_origin: NcbCode::new("KEN")?,
};
let person = PersonEntity {
    family_name: "DOE".into(),
    given_name: "JOHN".into(),
    birth_date: "19800101".into(),
    nationality: "KEN".into(),
    ncb_origin: NcbCode::new("KEN")?,
};
let linked = link_crime_to_person(&crime, &person);
```

## Wire Format

`serialize()` produces a 4-byte big-endian length prefix followed by JSON. This is a simplified transport encoding; production deployments should use NIST-ITL binary (ANSI/NIST-ITL 1-2011) or EBTS for full standards conformance.

## Error Handling

- `InvalidNcbCode(String)` - code is not exactly 3 ASCII alpha characters
- `MissingField(String)` - required envelope field absent
- `InvalidFormat(String)` - parse failure on deserialization

## References

- INTERPOL IABIS Technical Guidelines
- ANSI/NIST-ITL 1-2011: Data Format for the Interchange of Fingerprint, Facial and Other Biometric Information
- INTERPOL MIND/FIND network documentation
