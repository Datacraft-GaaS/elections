# M-Pesa Daraja Contract Tests

Integration test suite validating adapter compliance with Safaricom Daraja API contract. Tests request formatting, response parsing, callback validation, and error handling.

## Purpose

Verifies adapter correctly serializes all Daraja API payloads and parses responses. Tests both happy path and failure scenarios (timeouts, invalid responses, network errors). Ensures PII masking in logs and callback signature validation.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_stk_push.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "oauth" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**OAuth Flow:**
- Token request includes valid consumer credentials
- Access token cached and reused within TTL
- Token refresh on expiry
- Invalid credentials return 403

**C2B Callback:**
- Callback payload correctly parsed (MSISDN, amount, transaction ID, timestamp)
- Callback signature validated using shared secret
- Duplicate callbacks identified via TransID
- Invalid signatures rejected
- PII (MSISDN, names) masked in adapter logs

**STK Push:**
- Request includes encoded password (timestamp+passkey+shortcode hashed)
- Response CheckoutRequestID captured for polling
- Request timeout triggers circuit breaker
- ResponseCode 0 indicates success, others are errors

**B2C Payment:**
- Request includes ConversationID for result correlation
- ResultURL endpoint is publicly accessible
- Async callback delivers OriginatorConversationID matching request
- B2C timeout (30s default) triggers fallback queue

**Error Handling:**
- Network timeouts return error after MPESA_TIMEOUT_SEC
- 503 responses trigger circuit breaker
- Invalid JSON responses logged without crashing
- Retry logic respects exponential backoff and max retries

**Chaos Testing:**
- MPESA_CHAOS=true injects errors at configured rate
- Latency injection delays responses by configured milliseconds
- Circuit breaker opens after threshold failures
- Half-open state allows probe requests

**Fallback Behavior:**
- Payment queued locally if adapter unavailable
- Manual reconciliation can check transaction status via Daraja lookup
- Operator dashboard shows pending transactions
