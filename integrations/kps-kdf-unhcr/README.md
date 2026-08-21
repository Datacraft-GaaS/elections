# integrations/kps-kdf-unhcr

KPS/KDF/UNHCR displaced persons and refugee voter coordination adapter

## Wire protocol

Unified REST gateway; mTLS; POST /displaced/register, GET /displaced/{id}/status for refugee voter coordination.

### Endpoints

| Path | Description |
|---|---|
| `/displaced/register` | Register displaced voter |
| `/displaced/status` | Check displaced voter status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| refugeeId | A (Restricted) |
| campLocation | B (Internal) |
| nationalId | A (Restricted) |

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
| KPS_KDF_UNHCR_LATENCY_MS | 0 | ms delay added to every response |
| KPS_KDF_UNHCR_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
