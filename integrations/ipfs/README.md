# integrations/ipfs

Kenyan-hosted IPFS pinning adapter for M14/M17 transparency channel

**Implementation:** Rust (latency-critical transport adapter)

## Wire protocol

IPFS HTTP API v0; POST /api/v0/add for pin, POST /api/v0/pin/add for remote pin; content-addressed CIDs.

### Endpoints

| Path | Description |
|---|---|
| `/ipfs/pin` | Pin content to IPFS |
| `/ipfs/unpin` | Unpin CID |
| `/ipfs/status` | Check pin status |


### PII classes (ADR-0016)

| Field | Tier |
|---|---|
| content | C (Public) |
| cid | C (Public) |

## Circuit breaker

| Param | Value |
|---|---|
| failure threshold | 5 consecutive errors |
| open duration | 30 s |
| retry budget | 3 retries, exponential backoff, max 10 s |

## Latency injection (double)

| Env var | Default | Effect |
|---|---|---|
| IPFS_LATENCY_MS | 0 | ms delay added to every response |
| IPFS_ERROR_RATE | 0 | fraction of requests returning 503 |

## SLA / back-pressure

Transport-layer SLA: p99 < 500 ms (local IPFS/satellite node). Hard timeout: 5 s.
Circuit opens after 5 failures; callers receive 503 with `Retry-After: 30`.
