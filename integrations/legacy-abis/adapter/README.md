# legacy-abis/adapter

Python package wrapping legacy ABIS vendor SOAP/gRPC API.

## Layout

- `src/eel_adapter_legacy_abis/__init__.py` - public exports
- `src/eel_adapter_legacy_abis/service.py` - AbisAdapter, legacy protocol handling
- `src/eel_adapter_legacy_abis/views.py` - Pydantic models (BiometricMatchRequest, MatchResult)
- `src/eel_adapter_legacy_abis/soap.py` - SOAP 1.1 client wrapper
- `src/eel_adapter_legacy_abis/grpc.py` - optional gRPC client (not actively used)

## Key classes

AbisAdapter: wraps SOAP/gRPC endpoints, handles auth, circuit breaker.

BiometricMatchRequest: probe_template (base64 fingerprint), reference_id, match_type (1n | 11).

MatchResult: match_score (float 0-1), matched_reference_id, confidence_level (high | medium | low).

## Environment variables

- ABIS_SOAP_ENDPOINT: vendor SOAP service URL (required)
- ABIS_WS_USERNAME, ABIS_WS_PASSWORD: WS-Security credentials
- ABIS_CLIENT_CERT, ABIS_CLIENT_KEY: mTLS client certificate/key
- ABIS_TIMEOUT_SECS: default 30
- ABIS_MAX_RETRIES: default 2

## Build and test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## PII handling

Never log raw templates. Mask match results in audit logs.
