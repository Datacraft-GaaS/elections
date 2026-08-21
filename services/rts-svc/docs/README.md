# rts-svc

Real-Time Results Transmission Service for eel. Ingests Form 34A tally sheets from polling stations, compiles Form 34B and 34C aggregates, and publishes the national presidential tally with a cryptographic transparency log entry.

## Crates

| Crate | Role |
|---|---|
| `rts-api` | HTTP (port 8080) and gRPC (port 50051) surface for form ingestion, snapshots, ICR, disputes |
| `rts-domain` | Aggregates (Form34A/B/C), event constants, validators (EC-DAT-10, EC-DAT-11), candidate status |
| `rts-tally` | Exactly-once Kafka consumer, 34A->34B->34C aggregator, BLAKE3 Merkle tree, drift detector |
| `rts-projections` | Read-model projectors: RO view, CRO view, IAB view, portal snapshots |
| `rts-publisher` | Scheduled publication: bundle creation, stub signing, stub MinIO upload |
| `rts-integration-tests` | Integration tests (requires Docker) |

## Python

| Package | Role |
|---|---|
| `py/rts-icr` | ICR/OCR worker for Form 34A handwriting recognition |

## SLA

- Ingest p99 latency: <= 2s local
- Sustained write throughput: 50k writes/sec
- See `docs/SLA.md` for full targets.

## Events

See `docs/EVENTS.md` for the full event catalog.

## Threat Model

See `docs/THREAT-MODEL.md`.
