# integrations/legacy-abis

I05 legacy ABIS adapter: wraps the IEBC vendor SOAP or gRPC ABIS into the
eel-abis canonical schema.

## Endpoints

- `POST /abis/match/1n` - 1:N biometric search
- `POST /abis/match/11` - 1:1 biometric verification

## PII classes

| Field | Tier |
|---|---|
| probe_template | A (Restricted biometric) |
| reference_id | A (Restricted) |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 3 |
| open duration | 60 s |
| retry budget | 2, max 30 s |

## Double knobs

| Env var | Default |
|---|---|
| ABIS_LATENCY_MS | 0 |
| ABIS_MATCH_SCORE | 0.95 |

## SLA

ABIS 1:N p99 < 5 s. Adapter hard timeout 30 s.
