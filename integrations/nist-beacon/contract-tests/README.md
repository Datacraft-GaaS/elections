# nist-beacon/contract-tests

Consumer/provider contracts for NIST Randomness Beacon adapter.

## Purpose

Verifies Pulse schema, chaining hash validation, caching behavior, and fallback-to-CSPRNG logic.

## Running tests

```bash
uv run pytest -vxs tests/ci
```

## What is verified

- Pulse schema (outputValue, timeStamp, chainIndex, chainValue, previousChainValue)
- SHA-512/256 chaining hash: sha512_256(prev.outputValue + current.outputValue) == current.chainValue
- HTTP 200 response from beacon.nist.gov (live test, requires network)
- Cache: first call to get_pulse() fetches; second call (< TTL) uses cache
- Fallback: if NIST unreachable, fallback to os.urandom (BEACON_FALLBACK_TO_CSPRNG=true)
- Response latency (cache hit: sub-ms, cache miss: < 2s HTTP)

## Live provider verification

To test against real NIST beacon:

1. Run pytest (default hits live NIST beacon).
2. Verify response schema matches expected Pulse format.
3. Check chaining hash validates correctly.

Note: test must tolerate network delays (retries with exponential backoff).

## PII in contract tests

None (public randomness, no sensitive data).

## Test audit integration

RLA audit module (M21) depends on deterministic pulse for reproducible draws. Use double mock in CI for deterministic results.
