# integrations/mrz

I02 MRZ parser: ICAO 9303 TD1/TD2/TD3 with checksum verification.

Pure in-process Rust library. No network calls. Fuzz targets available
via `cargo fuzz` (see `adapter/eel-adapter-mrz-rs/fuzz/`).

Exposed as `cdylib` for Dart FFI via optional feature flag.

## PII classes (ADR-0016)

| Field | Tier |
|---|---|
| name | A (Restricted) |
| document_number | A (Restricted) |
| date_of_birth | A (Restricted) |
| nationality | B (Operational) |

## No double needed

Pure library; all validation is deterministic. ICAO test vectors serve as
the contract.

## Contract tests

Run: `cargo test -p eel-adapter-mrz-contract-tests`

Expected: all ICAO 9303 Appendix A vectors pass checksum and field extraction.
