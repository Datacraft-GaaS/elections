# NHIF-SHIF Contract Tests

Integration test suite validating adapter compliance with Social Health Authority REST API contract. Tests mTLS client certificate handling, API key authentication, SHIF contribution submission, and beneficiary verification.

## Purpose

Verifies adapter correctly formats all SHA API requests and parses responses. Tests mTLS certificate validation, API key header injection, bulk contribution formatting, national ID validation, and member eligibility checking. Ensures PII masking in logs.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_contribution_submit.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "beneficiary" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**mTLS Certificate:**
- Client certificate loaded from NHIF_CERT path
- Private key loaded from NHIF_KEY path
- Certificate CN matches employer identifier
- Server certificate validated against root CA

**API Key Header:**
- All requests include X-API-Key header
- API key value matches NHIF_API_KEY env var
- Missing API key returns 401 Unauthorized
- Invalid API key returns 403 Forbidden

**Contribution Submission:**
- Request body includes employer_code, month, employees list
- Each employee record has national_id, contribution_amount, insurance_class
- Response receipt_number captured for audit trail
- Response status parsed (ACCEPTED, REJECTED)
- Accepted and rejected counts reported
- Error details extracted for failed records
- Bulk submission (3000 employees) processed correctly
- Duplicate national IDs detected and rejected

**Beneficiary Verification:**
- Request uses correct national_id in URL path
- Response member status parsed (ACTIVE, INACTIVE)
- Member number retrieved
- Insurance class returned
- Enrollment date validated (ISO8601 format)
- Nonexistent member returns 404 with error message

**Error Handling:**
- Network timeout after NHIF_TIMEOUT_SEC triggers circuit breaker
- 429 Too Many Requests (rate limit) retry with exponential backoff
- 503 Service Unavailable triggers fallback queue
- Malformed JSON responses logged without crashing

**PII Protection:**
- National IDs masked in logs (showing only last 4 digits)
- Member numbers never logged in plaintext
- Contribution amounts logged only in structured metrics
- API key never logged or cached on disk
- Insurance class treated as restricted (logged as enum value only)

**Chaos Testing:**
- NHIF_CHAOS=true injects errors at configured rate
- Latency injection delays responses
- Contribution rejection rate injected correctly
- Member inactive rate injected correctly
- Circuit breaker opens after 2 failures
- Half-open state probes with single request
