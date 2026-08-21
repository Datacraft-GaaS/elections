# KRA iTax Contract Tests

Integration test suite validating adapter compliance with Kenya Revenue Authority iTax SOAP gateway contract. Tests SOAP envelope construction, WS-Security header injection, SOAP fault parsing, and mTLS certificate handling.

## Purpose

Verifies adapter correctly serializes SOAP requests with WS-Security UsernameToken headers and parses SOAP fault responses. Tests mTLS client certificate validation, PIN masking, TIN verification, and error recovery from legacy SOAP faults. Ensures PII is not logged in plaintext.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_tax_compliance.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "paye" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**SOAP Envelope Construction:**
- Envelope includes SOAP 1.1 namespace declaration
- Body contains correct operation (TaxComplianceCheck, PAYESubmission, P9AGenerate)
- Namespace prefixes resolve correctly

**WS-Security Header:**
- UsernameToken present in SOAP header
- Username field contains KRA PIN
- Password element has correct Type attribute
- Timestamp element included with Creation time
- No embedded credentials logged

**mTLS Certificate:**
- Client certificate loaded from KRA_ITAX_CERT path
- Private key loaded from KRA_ITAX_KEY path
- Certificate CN matches expected KRA agent identifier
- Server certificate validated against root CA

**Tax Compliance Check:**
- Request includes PINin UsernameToken, not request body
- Response status correctly parsed (COMPLIANT, NON_COMPLIANT)
- Certificate URL extracted from response
- Expiry date validated format (YYYY-MM-DD)
- Invalid PIN returns SOAP fault with faultcode "InvalidInput"

**PAYE Submission:**
- Request includes employer TIN in operation body
- Deduction list correctly formatted with employee PINs and amounts
- Response submission_id captured for audit trail
- Duplicate submission detected via idempotency key

**P9A Generation:**
- Request specifies fiscal year
- Response document_url is HTTP(S) accessible
- Base64-encoded PDF extracted and saved locally
- Year mismatch returns SOAP fault

**Error Handling:**
- SOAP fault with faultcode parsed correctly
- faultstring message extracted and logged (without credentials)
- Timeout after KRA_ITAX_TIMEOUT_SEC triggers circuit breaker
- Connection refused returns adapter error (not SOAP fault)

**PII Masking:**
- PINs logged as X...YY (first masked, last 2 visible)
- TINs never logged in adapter debug logs
- Names logged as initials only
- SOAP payloads in logs redacted for sensitive fields

**Chaos Testing:**
- KRA_ITAX_CHAOS=true injects SOAP faults at configured rate
- Latency injection delays SOAP responses
- Timeout injection forces adapter retry
- Circuit breaker opens after 2 failures
- Half-open state probes with single request
