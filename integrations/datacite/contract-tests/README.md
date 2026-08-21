# DataCite Contract Tests

Integration test suite validating adapter compliance with DataCite international DOI registration API contract. Tests DOI minting, metadata updates, public resolution, and BasicAuth credential handling.

## Purpose

Verifies adapter correctly formats all DataCite JSON-API requests and parses responses. Tests BasicAuth credentials, DOI suffix generation, metadata completeness, public URL registration, and DOI state transitions. Ensures JSON-API compliance.

## How to Run

Run all contract tests:
```bash
cd contract-tests
uv run pytest tests/ -vxs
```

Run specific test file:
```bash
uv run pytest tests/test_doi_mint.py -vxs
```

Run tests matching pattern:
```bash
uv run pytest -k "metadata" -vxs
```

Type check:
```bash
uv run pyright tests/
```

## What Is Verified

**BasicAuth Credentials:**
- Authorization header format: "Basic base64(username:password)"
- Credentials set from DATACITE_USERNAME and DATACITE_PASSWORD env vars
- Invalid credentials return 401 Unauthorized
- Missing Authorization header returns 401

**DOI Prefix and Suffix:**
- DOI prefix matches DATACITE_PREFIX env var (10.eeelection)
- Suffix generated correctly (year + sequence)
- Full DOI format: prefix/suffix (10.eeelection/2027.001)
- Suffix uniqueness enforced (no duplicates)
- Invalid DOI format rejected with 400

**DOI Minting:**
- Request includes required fields: title, URL, creators, publication year
- JSON-API format correct (type: "dois", attributes: {...})
- Response includes DOI string and state
- State transitions: draft -> registered -> findable
- Minting confirms public URL reachability
- Metadata validation enforced

**Metadata Completeness:**
- Titles required (minimum 1, language specified)
- Creators list required (names and affiliations)
- Publication year required and valid (YYYY format)
- Resource type required and validated
- Descriptions optional but validated if present
- Subjects optional but validated if present

**Metadata Updates:**
- Request specifies only fields to update (partial updates supported)
- Existing metadata preserved if not in update
- Descriptions added/removed correctly
- Subjects (keywords) updated correctly
- Update timestamp recorded and returned
- DOI state may transition on update

**Public Resolution:**
- DOI registered and findable via public API
- No authentication required for GET /dois/{doi}
- Public URL resolves to actual content
- Metadata searchable in DataCite search index
- DOI landing page generated automatically

**Error Handling:**
- Network timeout after DATACITE_TIMEOUT_SEC triggers circuit breaker
- 429 Too Many Requests (rate limit) retry with exponential backoff
- 503 Service Unavailable triggers fallback queue
- Malformed JSON responses logged without crashing
- Validation errors include human-readable messages

**JSON-API Compliance:**
- Request Content-Type: application/vnd.api+json
- Response includes data and type fields
- Relationships supported (e.g., client, repository)
- Error responses follow JSON-API error format
- Sparse fieldsets supported in responses

**Chaos Testing:**
- DATACITE_CHAOS=true injects errors at configured rate
- Minting failure rate injected correctly
- Metadata update failure rate injected correctly
- Latency injection delays responses
- Circuit breaker opens after 3 failures
- Half-open state probes with single request
