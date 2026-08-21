# legacy-abis/contract-tests

Consumer/provider Pact contracts for legacy ABIS adapter.

## Purpose

Verifies SOAP schema, WS-Security authentication, error codes, and biometric match result format.

## Running tests

```bash
uv run pytest -vxs tests/ci
```

## What is verified

- SOAP envelope structure (soap:Body, soap:Fault)
- BiometricMatchRequest schema (probe_template, reference_id, match_type)
- MatchResult schema (match_score 0-1 float, matched_reference_id, confidence_level enum)
- WS-Security UsernameToken in SOAP header
- mTLS client certificate validation
- HTTP 500 (SOAP fault), timeout (30s), rate limit (429)
- Match score stability (same probe returns same score)
- Response latency (p99 < 5s for 1:N, < 2s for 1:1)

## Provider verification

To verify against real ABIS (QA only):

1. Set ABIS_SOAP_ENDPOINT, ABIS_WS_USERNAME, ABIS_WS_PASSWORD, ABIS_CLIENT_CERT, ABIS_CLIENT_KEY.
2. Run pytest with --pact-verify flag.
3. Check pact.json for vendor response mismatches.

## PII in contract tests

Do not commit real fingerprints or reference IDs. Use synthetic test templates.

## Sunset note

This adapter is deprecated. New biometric work uses eel-biometric-svc (Rust). Legacy ABIS tests sunset Q3 2027.
