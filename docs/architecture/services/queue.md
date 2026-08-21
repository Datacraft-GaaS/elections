# queue-svc

## Purpose

Monitors voter queue lengths at polling stations by fusing computer-vision headcounts from kit cameras, GPS-anchored crowd signals, and voluntary queue snapshot reports. Flags potential disenfranchisement events when queues at closing time are large enough to trigger Section 14A recommendations to the ECC.

## Modules Owned

| M-ID | Slug | Description |
|---|---|---|
| M19 | queue-transparency | Queue snapshots, disenfranchisement flags, S.14A recommendations |

## Key Aggregates and State Stores

- `QueueSnapshot` (PostgreSQL, time-partitioned): per-station queue depth estimates with confidence bands.
- `DisenfranchisementFlag` (PostgreSQL): stations where closing-time queue exceeds threshold.
- `S14ARecommendation` (PostgreSQL): formal recommendations forwarded to ECC.

## Service Dependencies

| Direction | Service | Reason |
|---|---|---|
| Upstream | bvi-svc | Station identification rates (proxy for voter flow) |
| Upstream | field-mobile-svc | Field support and observer queue reports |
| Upstream | ecc-svc | Alert acknowledgement and S.14A status |
| Downstream | ecc-svc | Queue snapshots and disenfranchisement flags |

## Events Emitted

| Event | Topic | Description |
|---|---|---|
| `eel.queue.snapshot` | `eel.queue` | Station queue depth snapshot |
| `eel.queue.disenfranchisement.flagged` | `eel.queue` | Station flagged for late-closing risk |
| `eel.queue.s14a.recommended` | `eel.queue` | Section 14A recommendation raised |

## Events Consumed

| Event | Producer |
|---|---|
| `eel.bvi.identification.event` | bvi-svc |
| `eel.bvi.turnout.rollup` | bvi-svc |

## Primary APIs

- `packages/openapi-contracts/queue/openapi.yaml` - queue snapshot query, disenfranchisement feed, S.14A

## SLA Targets (ADR-0018, ADR-0022)

| Metric | Target |
|---|---|
| Snapshot freshness | less than 2 min |
| Availability | 99.5% |
| RTO | 30 min |
| RPO | 10 min (best-effort snapshots) |

## Degradation Modes

- If computer-vision feed is unavailable, queue estimates fall back to identification-rate proxy.
- S.14A recommendations are not blocked by snapshot staleness; ECC operators make final call.

## Runbook Links

- [docs/runbooks/degradation-per-service.md](../../runbooks/degradation-per-service.md)
