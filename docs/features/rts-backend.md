# RTS backend (M10)

Module M10 (`rts-backend`) owned by `rts-svc`. Event-sourced Form 34A/B/C tally stream with a TLA+-verified tally spec, exactly-once tally consumer via Kafka transactions, ONNX handwriting ICR at server side, and HSM-countersigned result publications.

## Contents

1. What M10 does
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

## 1. What M10 does

M10 is the server-side authoritative tally engine. It ingests signed Form 34A transmissions from kits (M08), provides the RO view for M09, accepts Form 34B/C from ROs, applies server-side ONNX ICR for cross-validation against kit-side ICR, enforces tally invariants (verified by TLA+ and proptest), countersigns accepted results with the HSM, and publishes to the transparency log (P05) and the public portal (M11).

## 2. Data plane

Migrations `V0001`-`V0003` under `services/rts-svc/migrations/`:

| Aggregate | Purpose | Data class |
|---|---|---|
| **Form34AEvent** | event stream per station: station_code, vote_tallies, icr_result, po_sig, kit_serial, ts_hlc, seq | A (event-sourced) |
| **Form34BProjection** | constituency tally compiled from Form 34A events; ro_edit_log; dispute_cases; ro_sig; agent_sigs | A |
| **Form34CProjection** | county tally compiled from Form 34B projections; county_ro_sig | A |
| **TallySnapshot** | snapshot_id, scope, totals, merkle_root, hsm_sig, published_at | A |
| **IcrCrossValidation** | record_id, form_id, kit_icr, server_icr, delta_flags, review_required | B |

Event-sourced per `services/rts-svc/crates/rts-domain/src/aggregates/` (form34a.rs, form34b.rs, form34c.rs). Projection tables via `V0002__projection_tables.sql`. Audit shadows via `V0003__audit_shadows.sql`.

## 3. Ingest sources

| Source | Auth | Notes |
|---|---|---|
| RTS kits (M08) via gRPC | SVID mTLS | Signed Form 34A (data path + CBOR+zstd SMS fallback) |
| USSD gateway (M08 SMS fallback) | Gateway secret | CBOR+zstd payload decoded by `rts-device-api/src/codec.rs` |
| RO app (M09) via REST | JWT + SpiceDB `RETURNING_OFFICER` | Form 34B / 34C submission with edit log + agent sigs |
| HSM (P06) | PKCS#11 via `integrations/hsm-pkcs11/` | Countersign accepted Form 34B/C before publication |

## 4. Pipeline

Crates: `services/rts-svc/crates/rts-api/`, `rts-tally/`, `rts-projections/`, `rts-publisher/`, `declaration/`.

```
kit POST /rts/v1/device/form34a (SVID mTLS, signed payload)
      -> verify Ed25519 per-record signature + kit-cert binding
      -> Idempotency-Key dedup (PG-durable)
      -> server-side ONNX ICR cross-validation (rts-tally/src/aggregator.rs):
             compare kit ICR tallies vs server ICR on same image
             delta > threshold -> IcrCrossValidation.review_required = true
             flagged in RO view (M09)
      -> append Form34AEvent to event stream (V0001)
      -> exactly-once tally consumer (rts-tally/src/consumer.rs):
             Kafka transaction wrapping: consume event + update projection
             + produce outbox event atomically
             -> TLA+ tally invariants enforced:
                  sum(station tallies) <= registered_voters_at_station
                  no subtraction; no double-count; monotone-increasing seq
      -> update Form34BProjection (rts-projections/src/ro_view.rs)
      -> emit eel.rts.form34a_received.v1

RO POST /rts/v1/ro/form34b (JWT, signed form)
      -> verify Ed25519 RO sig + agent countersig count
      -> dispute cases all resolved? -> proceed
      -> HSM countersign Form34B (P06)
      -> Merkle-append to P05 transparency log
      -> update Form34BProjection -> Form34CProjection
      -> rts-publisher/src/scheduler.rs: publish TallySnapshot to portal
      -> emit eel.rts.form34b_declared.v1
             eel.rts.form34c_declared.v1 (on county declaration)
             eel.rts.result_published.v1 (to public portal)
```

## 5. API surface

Contract: `packages/openapi-contracts/rts/openapi.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /rts/v1/device/form34a` (gRPC) | Kit (SVID mTLS) | Ingest signed Form 34A |
| `GET /rts/v1/ro/stations` | RO (JWT + SpiceDB) | Constituency station tally summary |
| `POST /rts/v1/ro/form34b` | RO (JWT + SpiceDB) | Submit Form 34B with edit log + agent sigs |
| `POST /rts/v1/ro/form34c` | County RO (JWT + SpiceDB) | Submit Form 34C |
| `GET /rts/v1/results/{scope}` | Public (rate-limited) | Published tally snapshot |
| `GET /rts/v1/results/{scope}/proof` | Court registrar (JWT) | Merkle inclusion proof for tally snapshot |
| `GET /rts/v1/projections/iab` | IAB (JWT + SpiceDB) | Read-only IAB tally view (`rts-projections/src/iab_view.rs`) |
| `GET /rts/v1/health` | Internal | Health check |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| RO desktop app (M09) | Tauri consuming RO view API | Form 34B compilation + submission |
| IAB read-only dashboard (M15) | ecc-svc consuming IAB view | Independent Audit Board tally witness |
| Public portal live counts (M11) | portal-svc consuming `eel.rts.result_published.v1` | Public results display |
| CRO view projection | `rts-projections/src/cro_view.rs` | Chief Returning Officer county summary |

## 7. Integrity + anti-tamper

- **Event-sourced Form 34A stream** -- append-only; no in-place mutation; every event signed with kit Ed25519 and kit_serial.
- **Exactly-once tally consumer** (`rts-tally/src/consumer.rs`) -- Kafka transactions wrap consume + projection update + outbox produce; no double-tally under any failure mode.
- **TLA+ Tally spec** -- `services/rts-svc/crates/declaration/src/proptest_rules.rs` encodes tally invariants; TLC model check verifies monotonicity and non-negativity.
- **ONNX ICR cross-validation** -- server re-runs ICR against the same scanned image; divergence from kit ICR flagged for RO review; not silently accepted.
- **HSM countersignature** -- every Form 34B/C and TallySnapshot countersigned by HSM (P06) before publication; STH pauses if HSM unavailable; queue drains on restore.
- **Merkle-anchoring** -- TallySnapshot Merkle root published to P05 transparency log; inclusion proof available via API.
- **Audit shadows** (`V0003__audit_shadows.sql`) -- every projection mutation logged with originator and ts_hlc.
- **Replay protection** -- `eel_idempotency_key` per Form 34A submission; M10 returns cached 200 on duplicate.
- **Tally drift monitor** (`rts-tally/src/drift.rs`) -- detects unexpected tallying-day shifts in station-level counts; alerts ECC.

## 8. Kenya statute alignment

- **Elections Act s.39 (tally declaration)** -- Form 34B / 34C are legal declaration instruments; M10 is the authoritative server-side store; HSM-signed results are the certifiable electronic record.
- **Elections Act s.82 (court access to results)** -- Merkle inclusion proof (`GET /rts/v1/results/{scope}/proof`) enables independent cryptographic verification.
- **Elections (General) Regulations Reg. 87** -- complete edit log from M09 stored alongside Form 34B in M10.
- **Elections (Technology) Regulations 2017** -- P05 transparency log anchoring + KEBS certification evidence from HSM countersigning.

## 9. Honest caveats + external prerequisites

- **ONNX ICR model accuracy** on Kenyan handwritten Form 34A must be validated on a representative field corpus; server-side model same risk as kit-side (M08).
- **HSM availability** during declaration -- if HSM is unavailable at declaration time, STH pauses and Form 34B submission queues; this introduces latency in live results publication.
- **Kafka transactions** for exactly-once require Kafka broker version >= 2.5; exact-version pinning is an infrastructure prerequisite.
- **TLA+ spec scope** covers tally invariants at the aggregate level; network partition scenarios are exercised by chaos tests, not the TLA+ model.
- SMS fallback CBOR+zstd decode is in `rts-device-api/src/codec.rs`; server-side decode tested by `tests/sms_fallback_decode.rs` fuzz corpus.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (tally consumer, ICR cross-validation, HSM sign, tally invariants) | nextest | M10-UNIT- |
| Property (monotone tally; no double-count; event-source replay consistency) | proptest (`declaration/src/proptest_rules.rs`) | M10-PROP- |
| Formal (TLA+ tally spec with TLC) | TLC model checker | M10-FORMAL- |
| Contract (M08 form34a ingest; M09 form34b; portal result; IAB view) | Pact | M10-PACT- |
| Integration (HSM SoftHSM; Kafka testcontainer; tlog double) | nextest + testcontainers (`rts-integration-tests/`) | M10-INT- |
| Load (46k station form34a concurrent; declaration throughput) | k6 | M10-LOAD- |
| Chaos (HSM kill; Kafka ISR shrink; PG primary failover during declaration) | Chaos Mesh | M10-CHAOS- |

Runbooks (to create): `docs/runbooks/rts-svc-degradation.md`, `docs/runbooks/rts-tally-recovery.md`.

Full spec: `docs/plans/modules/rts-backend.md`. Crates: `services/rts-svc/crates/rts-api/`, `rts-tally/`, `rts-projections/`, `rts-publisher/`, `declaration/`. Integration tests: `services/rts-svc/crates/rts-integration-tests/`.
