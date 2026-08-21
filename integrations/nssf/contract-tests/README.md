# NSSF Contract Tests

Integration test suite validating adapter compliance with National Social Security Fund REST API contract. Tests mTLS client certificate handling, API key authentication, contribution submission, member verification, and statement retrieval.

## Purpose

Verifies adapter correctly formats all NSSF API requests and parses responses. Tests mTLS certificate validation, API key header injection, bulk contribution formatting, member ID validation, and statement document handling. Ensures PII masking in logs.

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
uv run pytest -k "member" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**mTLS Certificate:**
- Client certificate loaded from NSSF_CERT path
- Private key loaded from NSSF_KEY path
- Certificate CN matches employer identifier
- Server certificate validated against root CA
- Certificate pinning verified

**API Key Header:**
- All requests include X-API-Key header
- API key value matches NSSF_API_KEY env var
- Missing API key returns 401 Unauthorized
- Invalid API key returns 403 Forbidden

**Contribution Submission:**
- Request body includes employer_code, period_month, contributions list
- Each contribution record has member_id, amount, deduction_date
- Response submission_id captured for audit trail
- Response status parsed (ACCEPTED, REJECTED)
- Accepted and rejected counts reported
- Error details extracted for failed records
- Bulk submission (5000 members) processed correctly
- Duplicate member IDs rejected

**Member Verification:**
- Request uses correct member_id in URL path
- Response member status parsed (ACTIVE, DORMANT, SUSPENDED)
- Account balance retrieved
- Last contribution date validated (ISO8601 format)
- Nonexistent member returns 404 with error message
- Suspended members handled correctly

**Statement Retrieval:**
- Request includes fiscal year parameter
- Response document URL is HTTPS accessible
- Base64-encoded PDF extracted correctly
- Generated date timestamp validated
- Invalid fiscal year returns 400
- Statement availability by year enforced

**Error Handling:**
- Network timeout after NSSF_TIMEOUT_SEC triggers circuit breaker
- 429 Too Many Requests (rate limit) retry with exponential backoff
- 503 Service Unavailable triggers fallback queue
- Malformed JSON responses logged without crashing
- Invalid employer code triggers application validation

**PII Protection:**
- Member IDs masked in logs (showing only first 3 chars, last 2)
- National IDs never logged or cached
- Member names logged as initials only
- Contribution amounts logged only in structured metrics
- API key never logged or cached on disk

**Chaos Testing:**
- NSSF_CHAOS=true injects errors at configured rate
- Latency injection delays responses
- Submission rejection rate injected correctly
- Member not found rate injected correctly
- Circuit breaker opens after 2 failures
- Half-open state probes with single request
