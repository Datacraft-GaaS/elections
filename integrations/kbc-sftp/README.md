# integrations/kbc-sftp

KBC SFTP daily hash bulletin publish adapter for M14/M17 transparency

## Wire protocol

SFTP over port 22; key-based auth; daily upload of signed hash bulletin file to /incoming/iebc/.

### Endpoints

| Path | Description |
|---|---|
| `/kbc/bulletin/publish` | Upload hash bulletin via SFTP |
| `/kbc/bulletin/status` | Check last upload status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| bulletinContent | B (Internal) |

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
| KBC_SFTP_LATENCY_MS | 0 | ms delay added to every response |
| KBC_SFTP_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
