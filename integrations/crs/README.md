# integrations/crs

CRS civil registration system birth/death record adapter for M35 voter registration

## Wire protocol

CRS REST API; OAuth2 + mTLS; GET /person/{id}/record returns civil status.

### Endpoints

| Path | Description |
|---|---|
| `/crs/person/record` | Fetch civil registration record |
| `/crs/death/check` | Check death registration |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| nationalId | A (Restricted) |
| birthCertNo | A (Restricted) |
| deathCertNo | A (Restricted) |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors |
| open duration | 30 s |
| retry budget | 3 retries, exponential backoff, max 10 s |

Configured via `eel-resilience-py` `CircuitBreaker`.

## Latency injection (double)

| Env var | Default | Effect |
|---|---|---|
| CRS_LATENCY_MS | 0 | ms delay added to every response |
| CRS_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
