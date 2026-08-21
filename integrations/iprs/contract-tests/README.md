# iprs/contract-tests

Consumer/provider Pact contracts for IPRS adapter.

## Purpose

Verifies IPRS REST/SOAP schema, authentication headers, error codes, and rate-limit behavior.

## Running tests

```bash
uv run pytest -vxs tests/ci
```

## What is verified

- PersonLookupRequest schema (id_number: str, lookup_type: enum)
- PersonLookupResponse schema (full_name, dob, gender, phone_number, postal_address, registration_status)
- mTLS client certificate validation
- SAML 2.0 bearer token in Authorization header
- HTTP 500 and 503 error responses
- Rate-limit headers (X-RateLimit-Limit, X-RateLimit-Remaining)
- Response latency (p99 < 2s)

## Provider verification

To verify against real IPRS endpoint (QA only):

1. Set IPRS_BASE_URL, IPRS_CLIENT_CERT, IPRS_CLIENT_KEY, IPRS_SAML_IDP.
2. Run pytest with --pact-verify flag.
3. Check pact.json for consumer/provider mismatch.

## PII in contract tests

Do not commit real citizen data. Use synthetic test IDs (99999999-series).
