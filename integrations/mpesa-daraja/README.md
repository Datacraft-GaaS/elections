# integrations/mpesa-daraja

M-Pesa Daraja C2B + B2C payments adapter

## Wire protocol

Safaricom Daraja REST API; OAuth2 client_credentials; C2B RegisterURL/Validate/Confirm; B2C PaymentRequest.

### Endpoints

| Path | Description |
|---|---|
| `/mpesa/c2b/register` | Register C2B URL |
| `/mpesa/c2b/callback` | Receive payment notification |
| `/mpesa/b2c/pay` | Initiate B2C payment |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| phoneNumber | A (Restricted) |
| amount | B (Internal) |
| accountReference | B (Internal) |
| transactionId | B (Internal) |

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
| MPESA_DARAJA_LATENCY_MS | 0 | ms delay added to every response |
| MPESA_DARAJA_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
