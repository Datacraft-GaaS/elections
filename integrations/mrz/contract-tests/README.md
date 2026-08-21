# mrz/contract-tests

Contract tests for MRZ ICAO 9303 compliance.

## Purpose

Verifies MRZ parser against ICAO 9303 specification. No network tests (local library).

## Running tests

```bash
cargo test -p eel-contract-tests-mrz --release
uv run pytest -vxs tests/ci  # Python integration tests
```

## Test vectors

ICAO 9303 Appendix A contains reference MRZ strings with known checksums. All vectors must:

- Parse without error
- Checksum validation pass
- Extract correct name, DOB, nationality, document number
- Handle optional fields correctly

## TD1, TD2, TD3 formats

- TD1: ID card (90 characters, 3 lines)
- TD2: visa (72 characters, 2 lines)
- TD3: passport (88 characters, 2 lines)

Contract tests cover all three formats.

## Fuzz testing

Continuous fuzz campaign via libFuzzer. See `adapter/fuzz/` for corpus and harness.

Expected: parser never panics on malformed input.
