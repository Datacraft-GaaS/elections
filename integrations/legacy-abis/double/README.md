# legacy-abis/double

Canned-response mock for legacy ABIS. Simulates SOAP API responses for testing.

## Running the mock

```bash
python -m eel_double_legacy_abis --port 8085
```

Listens on localhost:8085. Responds to SOAP requests with synthetic match results.

## Chaos knobs

Environment variables for fault injection:

- ABIS_DOUBLE_CHAOS: none | timeout | error | mismatch (default: none)
  - timeout: return 504 after 30s (slow legacy system)
  - error: return SOAP fault
  - mismatch: return inconsistent match score for same probe (device instability simulation)

- ABIS_DOUBLE_MATCH_SCORE: default 0.95 (high confidence)

- ABIS_DOUBLE_FALSE_NEGATIVE_RATE: fraction of actual matches to reject (default 0.0)

## Canned responses

Default 1:N search: returns 3 candidate matches (scores 0.95, 0.87, 0.72).

Default 1:1 verification: returns match_score=0.95, confidence=high.

Reference ID "invalid-ref" returns SOAP fault (not found).

## SOAP format

Responds to standard ABIS SOAP envelope. WS-Security headers honored but not validated (dev).
