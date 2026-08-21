# nist-beacon/double

Canned-response mock for NIST Randomness Beacon. Deterministic pulses for testing.

## Running the mock

```bash
python -m eel_double_nist_beacon --port 8086
```

Listens on localhost:8086. Responds with fixed canned pulses (no randomness).

## Chaos knobs

Environment variables for fault injection:

- BEACON_DOUBLE_CHAOS: none | stale | invalid_hash | malformed (default: none)
  - stale: return pulse from 5 min ago (cache miss scenario)
  - invalid_hash: corrupt chainValue (tests hash validation)
  - malformed: return invalid JSON

- BEACON_DOUBLE_PULSE_COUNT: number of pre-computed pulses (default: 10)

## Canned pulses

Returns same deterministic pulse set every time. All outputValues are valid SHA-512/256 hashes. Chain is internally consistent (each pulse's chainValue matches expected hash).

First call always returns last canned pulse. Subsequent calls respect cache TTL for deterministic testing.

## Deterministic for RLA testing

RLA audit draws must be reproducible. This double always returns the same entropy seed, enabling test audit reproducibility.
