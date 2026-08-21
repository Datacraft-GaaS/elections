# eel-abis-standards-icao

Rust implementation of ICAO 9303 Machine Readable Travel Documents (MRTD) Logical Data Structure (LDS) parsing and Basic Access Control (BAC) key derivation.

## Overview

ICAO 9303 defines the standard for electronic passports (eMRTDs). The document stores biometric and personal data in a Logical Data Structure encoded on an embedded chip. This crate covers parsing of data groups DG1-DG4, DG14, DG15, the Document Security Object (SOD), and the BAC/PACE key derivation stubs.

## Data Groups

### DG1 - Machine Readable Zone

The MRZ contains the identity data printed on the passport data page. This crate handles TD3 format (88 characters, two lines of 44):

- Line 1 (positions 0-43): document type, issuing country, primary identifier (surname + given names separated by `<<`)
- Line 2 (positions 44-87): document number, check digit, nationality, birth date, sex, expiry date, personal number, composite check digit

Accessor methods extract and trim filler characters (`<`):

```rust
let mrz = MachineReadableZone::parse(&bytes)?;
println!("{}", mrz.surname()?);        // "ERIKSSON"
println!("{}", mrz.given_names()?);    // "ANNA<MARIA"
println!("{}", mrz.document_number()?); // "L898902C3"
println!("{}", mrz.birth_date()?);     // "740812"
println!("{}", mrz.expiry_date()?);    // "120415"
```

### Check Digit Algorithm

The ICAO check digit uses weights 7, 3, 1 repeating over the field characters. Character mapping: `0-9` -> 0-9, `A-Z` -> 10-35, `<` -> 0.

```rust
assert_eq!(check_digit(b"L898902C3"), 6);  // document number
assert_eq!(check_digit(b"740812"), 2);     // birth date
```

### DG2 - Facial Image

Stores the reference face image. Supports JPEG (magic `FF D8`) and JPEG 2000 codestream (magic `FF 4F`):

```rust
let face = FaceImageData::parse(&bytes)?;
assert_eq!(face.image_format, ImageFormat::Jpeg);
```

### DG3 - Fingerprints

Contains one or more fingerprint images. Finger position follows ANSI/NIST-ITL coding (1 = right thumb, 2 = right index, etc.).

### DG4 - Iris

Iris image data for left or right eye per ISO/IEC 19794-6.

### DG14 - Security Options

Lists supported security protocols (PACE variants, Chip Authentication, Terminal Authentication) as OID strings.

### DG15 - Active Authentication Public Key

Stores the public key used in Active Authentication. Key is DER-encoded; algorithm identified by OID string.

## Document Security Object (SOD)

The SOD binds the data groups to a PKI signature chain:

```rust
let sod = DocumentSecurityObject::parse(&bytes)?;
let ok = sod.verify_passive_auth(1, &dg1_bytes)?;
```

Passive authentication computes SHA-256 of the data group bytes and compares to the stored hash. A full implementation would also verify the SOD's CMS signature against a country-signing CA certificate.

## BAC / PACE Key Derivation

Basic Access Control prevents unauthorized reading of the chip. The key derivation function combines the MRZ info (document number + birth date + expiry date with check digits):

```rust
let key = derive_key(mrz_info, KeyMode::Encryption)?;
let mac = derive_key(mrz_info, KeyMode::MacComputation)?;
```

This crate uses SHA-256 (truncated to 16 bytes) as the KDF hash. Production ICAO 9303 specifies SHA-1; swap the hasher when strict chip interoperability is needed.

PACE key derivation is a stub returning zeroed bytes. Wire in the full PACE implementation (ICAO 9303 Part 11 section 4.2) before connecting to live chips.

## Error Handling

All operations return `Result<_, IcaoError>`:

- `InvalidTag(u8)` - unexpected TLV tag
- `InvalidLength` - truncated data
- `InvalidMrz(String)` - MRZ format violation (wrong length, non-UTF8)
- `UnsupportedFormat(String)` - unrecognized image magic or encoding
- `SignatureVerificationFailed` - passive/active auth failure
- `KeyDerivationFailed(String)` - KDF internal error

## References

- ICAO Doc 9303 Part 3: MRTD specifications
- ICAO Doc 9303 Part 10: Logical Data Structure
- ICAO Doc 9303 Part 11: Security Mechanisms (BAC, PACE, AA, PA)
- ISO/IEC 19794-2: Fingerprint minutiae data
- ISO/IEC 19794-5: Face image data
- ISO/IEC 19794-6: Iris image data
