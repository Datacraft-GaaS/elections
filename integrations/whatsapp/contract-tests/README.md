# WhatsApp Contract Tests

Integration test suite validating adapter compliance with Meta WhatsApp Business Cloud API contract. Tests message sending (text, template, interactive), webhook signature verification, delivery status tracking, and error handling.

## Purpose

Verifies adapter correctly formats all WhatsApp API requests and parses responses. Tests Bearer token management, message formatting validation, webhook HMAC verification, delivery status callbacks, and rate limit handling. Ensures PII masking in logs.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_send_message.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "webhook" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**Bearer Token:**
- Authorization header format: "Bearer {token}"
- Token set from WA_ACCESS_TOKEN env var
- Invalid token returns 403 Unauthorized
- Expired token handling (refresh if supported)

**Text Message:**
- Request includes messaging_product, to (phone), type, text.body
- Phone number validated (254-prefixed Kenyan format)
- Message body required and non-empty
- Response message ID captured for delivery tracking
- Invalid phone format returns 400

**Template Message:**
- Request includes messaging_product, to, type, template.name
- Template pre-approved by Meta (no real template submission tested)
- Template parameters validated against schema
- Missing required params returns 400
- Response message ID captured

**Interactive Message:**
- Request includes interactive.type (list or button)
- List/button options formatted correctly
- Header, body, footer components validated
- Action section with reply buttons validated
- Response message ID captured

**Webhook Verification:**
- Initial webhook verification: hub.challenge returned if hub.verify_token matches
- HMAC-SHA256 signature validated on incoming requests
- Signature computed from request body and WA_WEBHOOK_SECRET
- Invalid signature rejected (401 Unauthorized)

**Message Delivery Callback:**
- Webhook callback parsed for delivery status
- Message status (delivered, read, failed) extracted
- Recipient phone number and timestamp captured
- Status used to update message delivery tracking table

**Incoming Message Callback:**
- Webhook callback parsed for incoming message
- Sender phone, message type, and body extracted
- Message attributed to correct contact/user
- Duplicate message detection via message ID

**Rate Limiting:**
- 429 Too Many Requests response handled gracefully
- Retry-After header respected
- Exponential backoff applied to retry attempts
- Queue builds up when rate limit hit

**Error Handling:**
- Network timeout after WA_TIMEOUT_SEC triggers circuit breaker
- 503 Service Unavailable triggers fallback to SMS channel
- Invalid JSON responses logged without crashing
- Malformed webhook callbacks rejected with 400

**PII Protection:**
- Phone numbers masked in logs (showing last 4 digits only)
- Message bodies not logged verbatim, only metadata (status)
- Template parameters not logged if they contain PII
- Webhook signatures validated without logging request body

**Chaos Testing:**
- WA_CHAOS=true injects errors at configured rate
- Message failure rate injected correctly (400 errors)
- Rate limit rate injected correctly (429 responses)
- Latency injection delays responses
- Circuit breaker opens after 5 failures
- Half-open state probes with single request
