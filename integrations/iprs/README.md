# integrations/iprs

I01 IPRS national-ID validation adapter.

## Wire protocol

IPRS exposes an OAuth 2.0 client-credentials token endpoint and a REST/SOAP
verification endpoint. This adapter wraps both behind an internal
`POST /iprs/verify` endpoint.

### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| national_id | A (Restricted) |
| full_name | A (Restricted) |
| dob | A (Restricted) |

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
| IPRS_LATENCY_MS | 0 | ms delay added to every response |
| IPRS_ERROR_RATE | 0 | fraction of requests returning 503 |
| IPRS_DUPLICATE_MODE | false | sets duplicate=true (EC-DAT-15 test) |

## SLA / back-pressure

Upstream IPRS SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
