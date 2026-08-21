# legacy-android-shim/contract-tests

Consumer/provider contracts for legacy Android device gRPC shim.

## Purpose

Verifies schema translation (legacy JSON to CloudEvents), device certificate validation, signature chain preservation, and operation code mapping.

## Running tests

```bash
cargo test -p eel-contract-tests-legacy-android-shim
uv run pytest -vxs tests/ci
```

## What is verified

- LegacyPayload schema (kit_id, op enum, timestamp, data, device_signature)
- CloudEvent 1.0 output schema (spec_version, type, source, time, data)
- Operation mapping: REG -> voter.registration, AUTH -> voter.auth, etc.
- Device certificate validation against device-ca.pem
- Original device signature preserved in spec_version field
- Shim re-signature present and valid
- gRPC error codes: InvalidArgument (bad cert), DataLoss (sig validation fail), Unavailable (shim down)
- 20 golden test vectors in fixtures/legacy_payloads.json cover all op types
- Response latency (sub-millisecond, no network calls)

## Device certificate testing

Test certs in `adapter/testdata/`: valid device cert, expired cert, wrong CA cert.

## PII in contract tests

Do not commit real device IDs or biometric data. Use synthetic test vectors.

## Deprecated notice

This adapter deprecated. New devices use native CloudEvents. Sunset Q3 2027.
