# integrations/nist-beacon

I14 NIST Randomness Beacon v2 public entropy source for risk-limiting audit (RLA).

## Purpose

Provides public, verifiable randomness for RLA draws in vote tabulation module (M21). Fetches pulses from NIST beacon; verifies chaining hash; caches locally.

## Modules using this adapter

M21 (rla-audit-tool entropy source only). Deterministic for test audit draws; RLA randomness comes from NIST beacon in production.

## Protocol

REST (public HTTPS, no auth). Endpoint: https://beacon.nist.gov/beacon/2.0/pulse/last

PII enforcement

None (public randomness).

## Rate limits

Beacon outputs every 60 seconds. No rate limit (public API). Adapter caches aggressively; no polling spike.

## Circuit breaker

n/a (fallback to local CSPRNG if unreachable). Network failure: use cached pulse (TTL 5 min). After 3 consecutive failures, log degradation warning.

## Chaos toggle

BEACON_CHAOS_ENABLED=true (double only). Returns stale pulse, invalid hash, malformed JSON.

## MoU status

None (public API, NIST service). No SLA negotiation needed.

## Health probe

GET https://beacon.nist.gov/beacon/2.0/pulse/last -> 200 {outputValue, timeStamp, ...}.

## Related

See adapter/, double/, contract-tests/ subdirectories.
