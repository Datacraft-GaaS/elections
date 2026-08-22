# BVI backend (M06)

Module M06 (`bvi-backend`) owned by `bvi-svc`. Server-side identification-event ingest across 64 Kafka partitions (one per constituency cluster), turnout rollups via materialised views, kit certificate 60 s revoke SLA, and honey-mode routing for suspicious kits. Provides the authoritative turnout view used by the Election Command Centre and the public portal.

## Contents

1. What M06 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M06 does

M06 is the back-end counterpart to M05. It receives signed IdentificationEvent batches from BVI kits, verifies signatures, deduplicates, and projects national turnout rollups in near-real-time. It manages kit certificate lifecycle (issue, revoke within 60 s), and routes kits flagged as compromised to a honey-mode sandbox where events appear accepted but are quarantined.

## 2. Data plane

Migrations `V1`-`V8` under `services/bvi-svc/migrations/`:

| Aggregate | Purpose | Data class |
|---|---|---|
| **IdentificationEvent** | event_id, voter_id, station_code, modality_used, auth_outcome, ts_hlc, kit_serial, clerk_id, Ed25519 sig | A |
| **TurnoutRollup** | rollup_id, scope (station/ward/constituency/county/national), total_identified, last_updated, seq | B |
| **KitCertificate** | kit_serial, cert_pem, issued_at, revoked_at, revoke_reason | C |
| **VoterLookupAudit** | lookup_id, requester_role, voter_id_hash, ts, outcome | C |
| **KitObjection** | objection_id, kit_serial, raised_by, reason, status, ts | B |

County partitioning: 64 Kafka partitions mapped to constituency clusters; `V3__identification_events.sql` partitions by station hash. Turnout materialised views: `V4__turnout_rollups.sql`.

## 3. Ingest sources

| Source | Auth | Notes |
|---|---|---|
| BVI kits (M05) via gRPC | SVID mTLS | Signed IdentificationEvent batch; idempotent by `eel_idempotency_key` |
| Kit session seal (M05) | SVID mTLS | Dual-control KitSessionEvent; triggers station-close projection |
| Kit attestation (M01/M05 shared) | SVID mTLS | TPM quote on every boot; 60 s revoke SLA on mismatch |
| Objection API | JWT + SpiceDB `PRESIDING_OFFICER` | Flag kit for investigation; triggers honey-mode routing |

## 4. Pipeline

Crates: `services/bvi-svc/crates/` (see `src/` and migrations).

```
kit POST /bvi/v1/identification (SVID mTLS, signed batch)
      -> verify Ed25519 per-event signature + kit-cert binding
             cert revoked? -> honey-mode routing:
                events accepted (200) but quarantined in honey partition
                eel.bvi.honey_event.v1 emitted to ECC
      -> Idempotency-Key dedup (PG-durable)
      -> store IdentificationEvent rows (V3 partition)
      -> project TurnoutRollup via REFRESH MATERIALIZED VIEW CONCURRENTLY
             update fired per batch; view scoped to station/ward/.../national
      -> emit eel.bvi.identification_event.v1 (Kafka eel.bvi.v1)
      -> emit eel.bvi.turnout_updated.v1 per affected scope

kit POST /bvi/v1/session/seal
      -> verify dual-control KitSessionEvent (two officer SVIDs)
      -> emit eel.bvi.session_sealed.v1
      -> trigger final station TurnoutRollup snapshot

kit certificate lifecycle:
      POST /bvi/v1/certs/issue   -> issue + store in V5__kit_certs.sql
      POST /bvi/v1/certs/revoke  -> revoke within 60 s SLA
             -> notify SPIRE (integrations/kit-pki/)
             -> all in-flight events from revoked kit -> honey partition
```

Turnout rollups are served to ECC (M16) and portal (M11) via SSE + REST. Near-real-time refresh target: rollup lag < 30 s per station.

## 5. API surface

Contract: `packages/openapi-contracts/bvi/openapi.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /bvi/v1/identification` (gRPC) | Kit (SVID mTLS) | Ingest signed IdentificationEvent batch |
| `POST /bvi/v1/session/seal` (gRPC) | Kit (SVID mTLS) | Submit dual-control session-close event |
| `GET /bvi/v1/turnout/{scope}` | ECC (JWT) | Near-real-time turnout rollup by scope |
| `GET /bvi/v1/voters/{voter_id_hash}/status` | Presiding Officer (JWT + SpiceDB) | Voter identification status (IDENTIFIED / NOT_YET) |
| `POST /bvi/v1/certs/issue` | PKI admin (JWT + SpiceDB) | Issue kit certificate |
| `POST /bvi/v1/certs/revoke` | PKI admin (JWT + SpiceDB) | Revoke kit certificate; triggers 60 s propagation |
| `POST /bvi/v1/objections` | Presiding Officer (JWT + SpiceDB) | Flag kit; triggers honey-mode routing |
| `GET /bvi/v1/lookup/audit` | Commission admin (JWT + SpiceDB) | Voter lookup audit trail (V7__voter_lookup_audit.sql) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| ECC turnout panel (M16) | ecc-svc consuming `GET /bvi/v1/turnout/{scope}` | Real-time national turnout visualisation |
| Public portal live counts (M11) | portal-svc consuming `eel.bvi.turnout_updated.v1` | Public turnout display (aggregate only; no voter IDs) |
| Presiding Officer app | BVI kit + field-mobile-svc | Station-level status and objection filing |

No standalone back-office UI; all human surfaces are consumers of the bvi-backend API.

## 7. Integrity + anti-tamper

- **Per-event Ed25519 signature** verified on every ingest; tampered events rejected; `eel_tamper` alert emitted.
- **Kit certificate 60 s revoke SLA** (`V5__kit_certs.sql`): revocation propagated to SPIRE within 60 s; all subsequent events from revoked kit routed to honey partition.
- **Honey-mode routing** -- compromised kits receive 200 responses (no alert to attacker); events quarantined; ECC alerted via `eel.bvi.honey_event.v1`.
- **Idempotency-Key dedup** -- PG-durable; duplicate IdentificationEvent for same voter at same station returns cached 200; voter cannot be counted twice.
- **Dual-control session seal** -- KitSessionEvent requires two officer SVIDs; both re-validated at commit via SpiceDB.
- **Voter lookup audit** (`V7__voter_lookup_audit.sql`) -- every `GET /bvi/v1/voters/{hash}/status` logged with requester role and outcome.
- **Outbox CDC** (`V8__outbox.sql`) -- Debezium picks up events; no loss if Kafka is temporarily unreachable.
- **Turnout rollup audit trail** -- every REFRESH logged with seq and updated_at; projection replayable from event stream.
- **Timing-constant voter status response** -- response shape identical for IDENTIFIED vs NOT_YET; no information leaked via timing.

## 8. Kenya statute alignment

- **Elections Act s.44A** -- `auth_outcome = S44A` accepted and stored; counted in turnout; no voter excluded from identification record.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- voter lookup API exposes only hashed voter IDs; no biographic fields; public turnout is aggregate only.
- **Kenya Data Protection Act 2019 s.31 (security)** -- field-level encryption on identification events; TLS 1.3; SpiceDB role enforcement.
- **Elections (Technology) Regulations 2017** -- signed identification events anchored in transparency log; kit certificate lifecycle auditable.

## 9. Honest caveats + external prerequisites

- **60 s revoke SLA** depends on SPIRE propagation latency to 65,000 kits; field trial required to confirm.
- **Honey-mode quarantine** is server-side only; a kit that has lost connectivity sends events into its local queue, which will be submitted once connectivity returns. Quarantine applies on server receipt.
- **Turnout rollup lag** (< 30 s) assumes REFRESH MATERIALIZED VIEW CONCURRENTLY completes within that window under full-election load; load test required at 46k-station scale.
- **64 Kafka partitions** mapped to constituency clusters; partition count cannot be changed post-deploy without a full consumer-group reset; chosen conservatively.
- `services/bvi-svc` has no standalone `docs/CAPABILITIES.md`; capability catalogue is embedded in `services/biometric-edge-svc/docs/CAPABILITIES.md` (M05/M06 share the kit protocol).

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (signature verifier, honey-mode router, turnout rollup projection, revoke propagation) | nextest | M06-UNIT- |
| Property (no double-count under arbitrary reorder; rollup monotone-increasing) | proptest | M06-PROP- |
| Contract (M05 ingest; ECC turnout consumer; portal turnout consumer) | Pact | M06-PACT- |
| Integration (SPIRE double; Kafka testcontainer; PG materialised view) | nextest + testcontainers | M06-INT- |
| Load (46k station events/hr; turnout rollup lag at 5x) | k6 | M06-LOAD- |
| Chaos (SPIRE kill; Kafka ISR shrink; PG primary failover) | Chaos Mesh | M06-CHAOS- |

Runbook (to create): `docs/runbooks/bvi-svc-degradation.md` (cert revoke SLA breach; honey-mode activation; rollup lag alarm).

Full spec: `docs/plans/modules/bvi-backend.md`. Service crates: `services/bvi-svc/crates/`. Migrations: `services/bvi-svc/migrations/V1`-`V8`.
