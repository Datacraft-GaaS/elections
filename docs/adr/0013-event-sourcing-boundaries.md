# ADR 0013: Event-sourcing boundaries

Status: accepted
Date: 2026-08-21

## Context

Event sourcing offers immutable audit, natural transparency-log integration, decoupled schema evolution, and time-travel, at the cost of projections, snapshots, replay logic, versioned events, and CQRS discipline. Not every module earns that cost. Under a minimise-complexity constraint, event sourcing is reserved for the modules where the event log itself is the deliverable, court evidence, statutory audit artefact.

## Decision

**Only four bounded contexts are event-sourced.** Everything else uses state + immutable audit shadow table.

Event-sourced:

| module | event log role |
|---|---|
| **M08/M09/M10 RTS** | The results log IS the election. Every capture, transmission, RO amendment, tally recomputation must be forensically reconstructable. |
| **M20 chain of custody** | Evidence in court. "Where was kit X between 06:00 and 18:00" must answer from signed events. |
| **M63 ballot serial chain** | Ballot integrity trail. Each serial's issuance/spoilage/use is legal evidence. |
| **M51 declaration** | Legal artefact of who declared what, when, signed by whom. |

Demoted to state + immutable audit shadow table (trigger-populated `audit.<table>_history`):

- M02 BVR register (22M-row table, low mutation rate on election day; audit table sufficient).
- M06 BVI turnout (turnout is derivable from vote counts in M10; audit table sufficient).
- M21 RLA audit (methodology publishes intermediate stages; state + audit adequate).
- M27 intent preservation (moderate volume; state + audit adequate).
- All other modules.

**Event store shape** (per event-sourced context, hosted in the appropriate T1 cluster per ADR-0012):

```
events                -- append-only, partitioned by month (hourly on election day)
  id UUID PK (UUIDv7)
  stream_id UUID
  stream_version INT
  event_type TEXT       -- versioned: 'RegisterEntryAdded.v1'
  payload JSONB         -- canonical JSON per RFC 8785
  metadata JSONB
  producer_signature BYTEA         -- Ed25519 (SPIFFE key)
  hsm_countersignature BYTEA       -- reserved for very-tier-1 events only
  recorded_at TIMESTAMPTZ
  event_time TIMESTAMPTZ
  hlc BIGINT
  prev_hash BYTEA
  UNIQUE (stream_id, stream_version)

snapshots             -- periodic aggregate state
outbox                -- what leaves via Kafka (Debezium reads this)
```

**Rules**:
- Never modify or delete an event. Corrections are new compensating events.
- `prev_hash` chain per stream is tamper-evident before the global transparency log ever hashes the events.
- HLC for causal ordering across services; wall-clock `time` NTP-authenticated for legal timestamps.
- UUIDv7 primary keys everywhere.

## Consequences

Positive: complexity confined to the four modules where the audit trail IS the deliverable; other 65+ modules stay simple; projections, replay, and CQRS discipline required in only four bounded contexts; faster day-1 delivery.

Negative: if a court demands full event-source-quality trail for a module we chose state+audit, we cannot retroactively provide it. Chosen scope reflects the legal requirements as understood; expandable via new ADR if requirements change.

## Alternatives rejected

- All 8 originally proposed modules event-sourced: complexity was not justified for M02/M06/M21/M27.
- No event sourcing anywhere: RTS/custody/declaration/ballot-serial legally require reconstructable trails.
- Event sourcing via EventStoreDB: vendor complexity; PG native event tables suffice.

## Related

ADR-0012 (T1 clusters host these), ADR-0015 (outbox → Kafka), ADR-0016 (PII in event payloads), ADR-0017 (transparency log ingests).
