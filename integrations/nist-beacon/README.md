# integrations/nist-beacon

NIST Randomness Beacon 2.0 client library.

Fetches pulses from `https://beacon.nist.gov/beacon/2.0/pulse/last` and
by-timestamp queries. Verifies outputValue chaining hash. Caches recent
pulses in memory. Used by rla-audit-tool.

## Circuit breaker

Network failures fall back to the most recent cached pulse (cache TTL: 5 min).
After 3 consecutive failures the rla-audit-tool logs a warning; randomness
source degraded.

## Double knobs

Deterministic canned pulses. No env knobs (always returns the same pulse set).

## SLA

Beacon period: 60 s. Cache miss: up to 2 s HTTP. Cache hit: sub-ms.
