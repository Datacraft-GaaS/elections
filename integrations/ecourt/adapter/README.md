# ecourt/adapter

FastAPI proxy for Judiciary eCourt petition filing and document handoff.

## Purpose

Wraps eCourt REST/SOAP API endpoints with mTLS authentication, session token management, circuit breaker, and PII field masking. Provides synchronous and asynchronous client interfaces.

## Layout

- `src/eel_adapter_ecourt/__init__.py` - public exports
- `src/eel_adapter_ecourt/service.py` - FastAPI app, ecourt_petition_file, ecourt_document_upload endpoints
- `src/eel_adapter_ecourt/views.py` - Pydantic models (EcourtPetitionRequest, EcourtPetitionResponse, etc.)
- `src/eel_adapter_ecourt/auth.py` - mTLS cert loading and session token refresh

## Key classes

**EcourtAdapter:** Main service class wrapping REST/SOAP endpoints, session token management, circuit breaker, PII masking.

**EcourtPetitionRequest:** petitionerName, respondentName, grounds (enum), briefStatement, requestedRelief.

**EcourtPetitionResponse:** caseNo, filingDate, estimatedHearing, status (enum), docketURL.

## Environment variables

- ECOURT_BASE_URL: REST endpoint base URL (required)
- ECOURT_CERT: path to mTLS client certificate PEM (required)
- ECOURT_KEY: path to mTLS private key (required)
- ECOURT_SESSION_TOKEN: bearer token for session auth (required)
- ECOURT_CACHE_TTL_SECS: case status cache TTL, default 3600
- ECOURT_TIMEOUT_SECS: request timeout, default 10
- ECOURT_CHAOS: chaos injection mode (timeout/error/slow)

## Build and test

```bash
uv sync
uv run pytest -vxs tests/ci
uv run pyright
```

## mTLS setup

Client cert and key must be provisioned by operations via NDA process. Store in secure vault (not git). Rotate annually per IEBC security policy.
