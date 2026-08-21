# IFMIS Contract Tests

Integration test suite validating adapter compliance with National Treasury IFMIS REST API contract. Tests OAuth token management, budget queries, voucher creation, and payment status tracking.

## Purpose

Verifies adapter correctly formats all IFMIS API requests and parses responses. Tests budget availability checks before voucher creation, error handling from invalid requests, and payment status polling. Ensures PII is not logged in plaintext.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_budget_check.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "voucher" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**OAuth Token Flow:**
- Token request includes valid client_id and client_secret
- Access token cached and reused within TTL (1800s)
- Token refresh triggered on expiry
- Invalid credentials return 401

**Budget Check Query:**
- Request includes vote head code and fiscal year
- Query parameter encoding correct
- Response parsing extracts allocation, committed, available amounts
- Budget code validation (returns 404 if not found)
- Available balance calculation correct (allocation - committed)

**Voucher Creation:**
- Request body includes vote_head, supplier_id, amount, description
- Request includes valid Authorization header
- Response voucher_id captured for status tracking
- Response status parsed (CREATED, PENDING, REJECTED)
- Timestamp format validated (RFC3339)
- Insufficient budget returns 422 with error message
- Invalid supplier_id returns 400

**Payment Status Query:**
- Request uses correct voucher_id in URL path
- Response status parsed correctly (PAID, PENDING, REJECTED)
- Paid amount and date extracted if available
- Nonexistent voucher returns 404
- Status polling (multiple queries) works correctly

**Error Handling:**
- Network timeout after IFMIS_TIMEOUT_SEC triggers circuit breaker
- 429 Too Many Requests (rate limit) retry with exponential backoff
- 503 Service Unavailable triggers fallback queue
- Malformed JSON responses logged without crashing
- Invalid vote head codes trigger application validation

**PII Protection:**
- Supplier names never logged in adapter debug logs
- Supplier account numbers masked in responses
- Voucher amounts logged only in structured metrics, not text logs
- OAuth client_secret not logged or cached on disk

**Chaos Testing:**
- IFMIS_CHAOS=true injects errors at configured rate
- Latency injection delays responses
- Budget check returns 404 at configured rate (vote head not found)
- Voucher creation returns 422 at configured rate (budget exhausted)
- Circuit breaker opens after 3 failures
- Half-open state probes with single request
