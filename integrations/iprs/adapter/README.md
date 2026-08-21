# iprs/adapter

Python package wrapping IPRS REST/SOAP API.

## Layout

- `src/eel_adapter_iprs/__init__.py` - public exports
- `src/eel_adapter_iprs/service.py` - IprsAdapter class, sync/async methods
- `src/eel_adapter_iprs/views.py` - Pydantic models (PersonLookupRequest, PersonLookupResponse)
- `src/eel_adapter_iprs/saml.py` - SAML 2.0 token generation and refresh

## Key classes

IprsAdapter: wraps REST/SOAP endpoints, handles auth, circuit breaker, PII field masking.

PersonLookupRequest: id_number (str), lookup_type (Literal["national_id", "passport"]).

PersonLookupResponse: full_name, dob, gender, phone_number, postal_address, registration_status.

## Environment variables

- IPRS_BASE_URL: REST endpoint base URL (required)
- IPRS_CLIENT_CERT: path to mTLS client certificate PEM
- IPRS_CLIENT_KEY: path to mTLS private key
- IPRS_SAML_IDP: SAML 2.0 identity provider endpoint
- IPRS_CACHE_TTL_SECS: fallback cache TTL, default 14400 (4 hours)

## Build and test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## mTLS setup

Client cert and key must be provisioned by operations via MoU NDA process. Store in secure vault (not git).
