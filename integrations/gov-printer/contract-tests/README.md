# Government Printer Contract Tests

Integration test suite validating adapter compliance with Kenya Government Printer REST API and SFTP contract. Tests gazette submission, print order creation, status tracking, file integrity verification, and dual transport fallback.

## Purpose

Verifies adapter correctly formats gazette submissions and print orders, validates file SHA-256 hashes, handles dual transport (REST + SFTP fallback), and parses status responses. Tests API key authentication, print job tracking, and error recovery.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_gazette_submit.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "print" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**API Key Header:**
- All REST requests include X-API-Key header
- API key value matches GOV_PRINTER_API_KEY env var
- Missing API key returns 401 Unauthorized
- Invalid API key returns 403 Forbidden

**Gazette Submission:**
- Request includes gazette_date, issue_number, pdf_filename, file_size, content_hash_sha256
- SHA-256 hash computed correctly from PDF bytes
- File size matches actual PDF data
- Response submission_id captured for publication tracking
- Response status parsed (RECEIVED, PROCESSING, PUBLISHED)
- Publication URL returned for gazette access
- Publication date estimated correctly
- Malformed hash returns 400 error

**Print Order Creation:**
- Request includes document_id, print_copies, binding_type, delivery_address
- Copies count validated (min 1, max 10000)
- Binding type validated (hardcover, softcover, spiral)
- Delivery address required and non-empty
- Response order_id captured for tracking
- Response status parsed (ACCEPTED, REJECTED)
- Estimated delivery date calculated from order date + 10 business days
- Invalid document_id returns 400

**Print Status Query:**
- Request uses correct order_id in URL path
- Response status parsed (ACCEPTED, PRINTED, SHIPPED, DELIVERED)
- Current stage tracked (queued, printing, quality-check, packing, shipping)
- Estimated completion date retrieved
- Nonexistent order returns 404

**File Integrity:**
- SHA-256 hash of PDF data matches expected value
- File size byte count validated
- Corrupted file detected and rejected
- Adapter computes hash correctly for verification

**Dual Transport Fallback:**
- REST submission attempted first
- On REST timeout or 503, SFTP fallback triggered
- SFTP file placement correct (/incoming/gazette_*.pdf)
- SFTP key authentication works (private key loaded)
- Fallback success tracked in logs
- Both transports deliver same submission result

**Error Handling:**
- Network timeout after GOV_PRINTER_TIMEOUT_SEC triggers circuit breaker
- 503 Service Unavailable triggers SFTP fallback
- 429 Rate limit handled with backoff (REST limit is 10/min)
- Malformed JSON responses logged without crashing
- SFTP connection errors trigger email fallback

**Circuit Breaker:**
- Aggressive breaker opens after 1 failure (legacy system SLA)
- Open duration 120 seconds (long SLA = long recovery)
- Half-open state probes with single request

**Chaos Testing:**
- GOV_PRINTER_CHAOS=true injects errors at configured rate
- Submission rejection rate injected correctly
- Delivery date delays injected correctly
- Latency injection delays responses
- Timeout injection forces adapter fallback
