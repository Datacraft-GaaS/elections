# integrations/card-manufacturer

Voter card manufacturer order and dispatch adapter

## Wire protocol

Card manufacturer REST API; mTLS + API key; POST /card/order, GET /card/order/{ref}/status, POST /card/dispatch.

### Endpoints

| Path | Description |
|---|---|
| `/card/order/submit` | Submit card production order |
| `/card/order/status` | Check order status |
| `/card/dispatch/confirm` | Confirm card dispatch |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| voterNo | A (Restricted) |
| fullName | A (Restricted) |
| cardRef | B (Internal) |

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
| CARD_MANUFACTURER_LATENCY_MS | 0 | ms delay added to every response |
| CARD_MANUFACTURER_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
