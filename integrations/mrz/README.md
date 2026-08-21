# integrations/mrz

I02 MRZ parser: ICAO 9303 TD1/TD2/TD3 machine readable zone with checksum verification.

## Purpose

Parses and validates MRZ data from travel documents (passports, travel permits, IDs). Used in field biometric verification and diaspora kit flows.

## Implementation

Pure in-process Rust library. No network calls. All validation deterministic. Fuzz targets available via `cargo fuzz` (see `adapter/eel-adapter-mrz-rs/fuzz/`).

Exposed as `cdylib` for Dart FFI via optional feature flag (legacy Android kit compatibility).

## Modules using this adapter

M01 (bvr-device), M05 (bvi-device kit), M24 (diaspora-kit).

## PII enforcement

High tier. MRZ contains full name, DOB, nationality, document number, expiration date. Field-level encryption for storage; no logging of raw MRZ.

## No double needed

Pure library; all validation is deterministic based on ICAO 9303 spec. ICAO test vectors serve as the contract. No mocking required.

## Contract tests

Run: `cargo test -p eel-adapter-mrz-contract-tests`

Expected: all ICAO 9303 Appendix A test vectors pass checksum and field extraction.

## Usage

```rust
use eel_adapter_mrz::MrzParser;
let result = MrzParser::parse(raw_mrz_str)?;
assert_eq!(result.name, "DOE, JANE");
```

## Chaos toggle

MRZ_CHAOS_ENABLED=true (adapter only, for chaos testing). Injects parse errors on rare checksums.
