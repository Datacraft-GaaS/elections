# ecourt/contract-tests

Consumer/provider Pact contracts for eCourt adapter.

## Purpose

Verifies eCourt REST/SOAP schema, authentication headers, error codes, and rate-limit behavior. Runs against mock (default) or live endpoint (QA).

## Run command

```bash
uv run pytest -vxs tests/ci
```

To verify against live eCourt endpoint (QA only):

```bash
ECOURT_BASE_URL=https://api.ecourt.go.ke ECOURT_CERT=$CERT ECOURT_KEY=$KEY \
  ECOURT_SESSION_TOKEN=$TOKEN \
  uv run pytest -vxs tests/ci --pact-verify
```

## What is verified

- EcourtPetitionRequest schema (petitionerName, respondentName, grounds enum, briefStatement)
- EcourtPetitionResponse schema (caseNo, filingDate, estimatedHearing, status)
- mTLS client certificate validation
- Bearer token in Authorization header
- HTTP 400, 500, 503 error responses
- Rate-limit headers (X-RateLimit-Limit, X-RateLimit-Remaining)
- Circuit breaker behavior (2 failures -> open for 60s)
- Response latency (p99 < 2s on happy path)

## PII in contract tests

Do not commit real citizen data. Use synthetic test petitioner names (Jane Doe, John Smith). Real case numbers from QA environment are acceptable for rate-limit testing.

