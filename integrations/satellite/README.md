# integrations/satellite

Starlink/VSAT fallback bearer adapter for edge kits

**Implementation:** Rust (latency-critical transport adapter)

## Wire protocol

Starlink gRPC API + VSAT AT-command bridge; satellite link status, bandwidth, latency metrics.

### Endpoints

| Path | Description |
|---|---|
| `/satellite/status` | Get satellite link status |
| `/satellite/failover/activate` | Activate fallback bearer |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| deviceId | B (Internal) |
| linkMetrics | C (Public) |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors |
| open duration | 30 s |
| retry budget | 3 retries, exponential backoff, max 10 s |

## Latency injection (double)

| Env var | Default | Effect |
|---|---|---|
| SATELLITE_LATENCY_MS | 0 | ms delay added to every response |
| SATELLITE_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Transport-layer SLA: p99 < 500 ms (local IPFS/satellite node). Hard timeout: 5 s.
Circuit opens after 5 failures; callers receive 503 with `Retry-After: 30`.
