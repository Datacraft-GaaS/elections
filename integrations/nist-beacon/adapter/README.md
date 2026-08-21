# nist-beacon/adapter

Python package wrapping NIST Randomness Beacon v2 API.

## Layout

- `src/eel_adapter_nist_beacon/__init__.py` - public exports
- `src/eel_adapter_nist_beacon/service.py` - BeaconAdapter, HTTP client
- `src/eel_adapter_nist_beacon/views.py` - Pydantic models (Pulse, PulseResponse)
- `src/eel_adapter_nist_beacon/hash.py` - SHA-512/256 chain verification

## Key classes

BeaconAdapter: fetches latest pulse, verifies chaining hash, manages local cache.

Pulse: outputValue (hex), timeStamp (Unix), outputLength, chainIndex, chainValue, previousChainValue.

## Environment variables

- BEACON_CACHE_TTL_SECS: default 300 (5 minutes)
- BEACON_TIMEOUT_SECS: default 10
- BEACON_FALLBACK_TO_CSPRNG: default true (use os.urandom if NIST unreachable)

## Build and test

```bash
uv sync
uv run pytest -vxs tests/ci
```

## Hash verification

Each pulse includes SHA-512/256 chain value. Verify: sha512_256(prev_pulse.outputValue + current_pulse.outputValue) == current_pulse.chainValue.

## Caching

Recent pulses cached in memory (TTL 5 min). No persistence to disk (public data, no PII).
