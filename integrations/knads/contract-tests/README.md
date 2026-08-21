# KNADS Contract Tests

Integration test suite validating adapter compliance with Kenya National Archives and Documentation Service REST API contract. Tests document ingestion, retrieval, metadata handling, and preservation verification.

## Purpose

Verifies adapter correctly formats all KNADS API requests and parses responses. Tests API key authentication, document metadata enrichment, base64 encoding, archival reference tracking, and preservation status polling. Ensures PII masking in logs.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_record_ingest.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "preservation" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**API Key Header:**
- All requests include X-API-Key header
- API key value matches KNADS_API_KEY env var
- Missing API key returns 401 Unauthorized
- Invalid API key returns 403 Forbidden

**Record Ingestion:**
- Request includes collection_id, document_title, document_type, content_type
- Metadata dict includes creator, subject, date_range
- Payload properly base64-encoded for binary content
- Response ingest_id captured for audit trail
- Response status parsed (ACCEPTED, PROCESSING, ARCHIVED)
- Archival reference assigned and returned
- Preservation URL is HTTP(S) accessible
- Ingest timestamp validated (RFC3339 format)
- Invalid document_type returns 400

**Record Retrieval:**
- Request uses correct archival_reference in URL path
- Response document title and metadata extracted
- Preservation format returned (e.g., PDF/A-3, MIAOU)
- Retrieval URL is HTTP(S) accessible
- Access restrictions (UNRESTRICTED, RESTRICTED, CLASSIFIED) returned
- Nonexistent reference returns 404
- Content type correctly reported

**Preservation Verification:**
- Request uses correct archival_reference
- Response status parsed (VERIFIED, DEGRADED, LOST)
- Last verified date retrieved and formatted correctly
- Verification method returned (FIXITY_CHECK, VISUAL_INSPECTION)
- SHA-256 fixity hash extracted and validated
- Degraded status triggers manual verification workflow
- Lost status escalates to archival recovery team

**Metadata Handling:**
- Document title required and non-empty
- Collection ID validated
- Metadata fields (creator, subject) captured
- Date range formatted correctly (start_date-end_date or single date)
- Custom metadata stored and retrieved

**Error Handling:**
- Network timeout after KNADS_TIMEOUT_SEC triggers circuit breaker
- 429 Too Many Requests (rate limit) retry with exponential backoff
- 503 Service Unavailable triggers local queue fallback
- Malformed JSON responses logged without crashing
- Invalid content type returns 400

**PII Protection:**
- Document titles not logged verbatim if containing PII
- Metadata values not logged except enum values
- Archival references logged for audit trail
- API key never logged or cached on disk

**Chaos Testing:**
- KNADS_CHAOS=true injects errors at configured rate
- Ingestion failure rate injected correctly
- Preservation degraded rate injected correctly
- Latency injection delays responses
- Circuit breaker opens after 3 failures
- Half-open state probes with single request
