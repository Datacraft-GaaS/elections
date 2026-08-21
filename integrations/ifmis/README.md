# integrations/ifmis

IFMIS government financial management SOAP wrapper for M54 budget

## Wire protocol

SOAP/WSDL over HTTPS; WS-Security UsernameToken; operations: GetBudgetAllocation, PostExpenditure, GetPaymentStatus.

### Endpoints

| Path | Description |
|---|---|
| `/ifmis/budget/allocation` | Query budget allocation |
| `/ifmis/expenditure/post` | Post expenditure entry |
| `/ifmis/payment/status` | Check payment status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| supplierId | B (Internal) |
| invoiceAmount | B (Internal) |
| voteHead | B (Internal) |

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
| IFMIS_LATENCY_MS | 0 | ms delay added to every response |
| IFMIS_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Upstream SLA: 95th-percentile < 2 s. Adapter timeout: 10 s hard.
Back-pressure: circuit opens after 5 failures; callers receive 503 with
`Retry-After: 30`.
