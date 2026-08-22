# Continuous voter registration (M35)

Module M35 (`cvr-continuous-voter-register`) owned by `bvr-svc`. Keeps the voter register a living record between elections by ingesting deaths, ID corrections, dormancy signals, and family reports; proposing removals; approving them under commissioner + HSM; publishing signed quarterly gazette notices anchored in the Merkle log.

## Contents

1. What CVR does (vs M01 initial enrolment)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What CVR does (vs M01 initial enrolment)

- **M01** = first-time enrolment via BVR kits in the field (initial capture of biometrics + demographics).
- **M35** = maintain the register between elections: ingest deaths, ID corrections, dormancy signals, family reports; propose removals; approve them under commissioner + HSM; publish signed quarterly gazette notices anchored in the Merkle log.

The two modules share the `Voter` aggregate but write disjoint event types.

## 2. Data plane

Three aggregates (per `services/bvr-svc/crates/cvr/src/models.rs` and the M35 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **VoterRegisterEvent** | Every proposed / approved / rejected / reinstated register mutation with source (`CRS` / `NRB` / `KRA` / `FAMILY_SHORTCODE` / `OFFICER_MANUAL`), confidence, batch, decision basis. | A |
| **RegisterSnapshot** | Point-in-time integrity anchor: `total_registered` + `root_hash` (SHA-256 over sorted voter ID list) + `merkle_root` + Commissioner HSM signature + `gazette_notice_id`. | A |
| **BatchIngestionReport** | Per-source batch report: `received_at` + `record_count` + `matched` / `unmatched` / `flagged` / `actioned` + error CSV URL. | B |

Migrations that back these aggregates land under `services/bvr-svc/migrations/`.

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **CRS** (Civil Registration Service, death notifications) | Daily SFTP | `integrations/crs/adapter` | SVID mTLS + PGP-signed batches | Retry with alert; circuit breaker via `packages/eel-resilience-rs` |
| **NRB** (National Registration Bureau, ID corrections) | Weekly SFTP | `integrations/nrb/adapter` | SVID mTLS | Retry, then flag for officer triage |
| **KRA** (dormancy signals) | Quarterly SFTP | `integrations/kra-itax/adapter` (Tier-B, WP-4.12) | SVID mTLS | Retry, then age-out with alert |
| **`*400#` family short-code** | Real-time USSD | `integrations/ussd-gateway/adapter` | Gateway secret | Rate-limited 1000 / hr; every report is provisional, requires CRS confirmation within 90 days |
| **Officer manual** | On demand | Direct API + Django | JWT + SpiceDB `COMMISSION_OFFICER` role | N/A |

Every adapter ships a `double/` under the same integration directory so the pipeline works end-to-end before the MoU-driven live endpoints are provisioned.

## 4. Pipeline

Files on disk under `services/bvr-svc/crates/cvr/src`: `lib.rs`, `models.rs`, `ingest.rs`, `routes/`, `state.rs`, `main.rs`, `error.rs`.

```
ingest source (CRS / NRB / KRA / USSD / manual)
      -> batch validate + canary-row round-trip check
      -> per-row match against BVR (M02)
      -> classify VoterRegisterEvent.event_type in
            { DEATH_PROPOSED, DEATH_CONFIRMED, ID_CORRECTION,
              DORMANCY_FLAG, FAMILY_NOTIFICATION,
              REMOVAL_APPROVED, REMOVAL_REJECTED, REINSTATEMENT }
      -> ABIS soft-hold via integrations/abis (M03) if biometric
         correlation is ambiguous (never a hard delete)
      -> officer triage queue (Phoenix LiveView at
         services/bvr-svc/ex/cvr_web)
      -> commissioner approve / reject with HSM signature
      -> emit CloudEvents via packages/eel-outbox-rs:
            eel.bvr.removal_proposed.v1
            eel.bvr.removal_approved.v1
            eel.bvr.removal_rejected.v1
            eel.bvr.id_correction_applied.v1
      -> quarterly RegisterSnapshot with Commissioner HSM signature
         + Merkle-anchored gazette notice via P05:
            eel.bvr.gazette_published.v1
            eel.bvr.register_snapshot.v1
```

Every step goes through the transactional outbox (`packages/eel-outbox-rs`) so no event is lost if PG commits but Kafka is unreachable. Debezium picks up outbox rows and publishes to Kafka topics `eel.bvr.cvr.v1` and `eel.bvr.snapshot.v1`.

## 5. API surface

Contract: `packages/openapi-contracts/bvr/cvr.yaml`. Every write endpoint enforces the `Idempotency-Key` header (backed by `eel_idempotency_keys` in V023).

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/cvr/batches/crs` | `CRS_SYSTEM` (SVID) | Ingest CRS death batch, 5/day cap |
| `POST /v1/cvr/batches/nrb` | `NRB_SYSTEM` (SVID) | Ingest NRB ID-correction batch, 5/day cap |
| `POST /v1/cvr/batches/kra` | `KRA_SYSTEM` (SVID) | Ingest KRA dormancy batch, 5/quarter cap |
| `GET /v1/cvr/events` | Commission officer (JWT + SpiceDB) | Triage queue |
| `PATCH /v1/cvr/events/{id}` | Commission officer | Approve / reject / hold |
| `GET /v1/cvr/snapshots` | Commission admin | List snapshots |
| `POST /v1/cvr/snapshots` | Commissioner + HSM | Create + sign new snapshot (2/day cap) |
| `GET /v1/cvr/snapshots/{id}/proof` | Court registrar | Merkle inclusion proof |
| `POST /v1/cvr/family-notifications` | USSD gateway secret | `*400#` intake, 1000/hr |
| `GET /v1/cvr/status/{national_id_hash}` | Public, 30/hr | Citizen self-check; no internal flags |

Every write is HSM-signed via the response-signing middleware in `packages/eel-api-server-rs`. Every read + write is audited into `eel_bvr_audit_shadow`.

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Officer triage console (LiveView) | `services/bvr-svc/ex/cvr_web` | Officer sees proposed removals, drills into evidence, approves or holds |
| Commissioner gazette panel + batch dashboard (Django) | `services/bvr-svc/django/gazette_admin` | Commissioner signs quarterly gazette; ops staff monitor batch health |
| Public gazette + self-check (Astro) | `services/bvr-svc/astro/cvr-public-site` | Public views the gazette; citizens self-check with hashed national ID |
| Read API (Rust axum) | `services/bvr-svc/crates/bvr-api` + `crates/cvr` | REST + gRPC surface for internal + external consumers |

Languages: EN + SW for staff and commissioner surfaces; citizen short-code and self-check in EN + SW + Kikuyu + Luo + Kamba. WCAG 2.2 AA across all human surfaces.

## 7. Integrity + anti-tamper

- **Merkle-anchored snapshots** — every `RegisterSnapshot` is HSM-signed by a Commissioner (per-cycle KEK from `eel_kek_generations` in V022) and its `root_hash` becomes a leaf in the P05 transparency log. Court registrars can pull the inclusion proof via `GET /v1/cvr/snapshots/{id}/proof`.
- **ABIS soft-hold** — proposed removals trigger `POST /internal/soft-hold` to ABIS (M03) so the biometric template is retained until adjudication settles. No hard delete without dual confirmation.
- **Family-report grace window** — `FAMILY_NOTIFICATION` events sit in `PENDING` for 90 days waiting for CRS confirmation. If CRS confirms within the window then escalate to `DEATH_CONFIRMED` then officer approval then removal. If not, auto-expire to `HELD` with an SMS alert to the reporting party for evidence (birth-cert or ODPC-approved).
- **Quarterly gazette** — `eel.bvr.gazette_published.v1` fires on Q1 / Q2 / Q3 / Q4; the gazette notice is submitted via `integrations/gov-printer/gazette-report-poster` and its receipt id is stored in the snapshot row.
- **Circuit breakers** on every external adapter via `packages/eel-resilience-rs` prevent CRS / NRB / KRA outages from cascading.
- **Silent-corruption defence** — canary rows are seeded into every batch; if they don't round-trip the batch is rejected with a `BatchIngestionReport` failure and the operator is paged.
- **Event chain** — every `VoterRegisterEvent` includes `prev_event_sha256` so an auditor can walk the per-voter history and verify no mutation has been silently rewritten.

## 8. Kenya statute alignment

- **Kenya Data Protection Act §26 (minimisation)** — snapshots persist hashed national IDs; raw IDs never leave the T1-voter store.
- **Kenya Data Protection Act §40 (right of erasure)** — satisfied via the removal event chain + gazette publication; the erasure event itself is signed and anchored so the deletion is provable.
- **Kenya Data Protection Act §41 (automated processing)** — every automated proposal (CRS batch matched to a voter) requires officer approval before it becomes a removal; no fully-automated decisions.
- **Elections Act §6A (public inspection)** — satisfied via `astro/cvr-public-site` + `GET /v1/cvr/snapshots/{id}/proof` + gazette notice.
- **Elections (Technology) Regulations 2017** — signed snapshots + Merkle anchoring + KEBS-certifier attestation before each quarterly publish.

## 9. Honest caveats + external prerequisites

- MoUs with CRS + NRB + KRA are external prerequisites. The adapters + doubles are ready under `integrations/{crs,nrb,kra}/`. Actual bulk SFTP endpoints must be provisioned by IEBC.
- Family short-code `*400#` requires a Safaricom / Africa's Talking commercial arrangement (I08 Tier-A integration, adapter shipped, live short-code allocation pending).
- Quarterly gazette submission depends on Gov Printer accepting the XML envelope schema. The adapter is written to a documented template; the exact schema is locked at first live batch.
- Kit-based CVR corrections (physical NCPD centres) are not built. Only server-side ingest + officer triage. The spec left kit-based correction for a future wave if IEBC decides to authorise it.
- ODPC breach notification flow for a botched CVR mutation is wired via `integrations/odpc-breach` but the reporting portal endpoint is set at first live incident.

## 10. Test coverage + operational runbooks

Every CVR path has coverage under `services/bvr-svc/crates/cvr/tests/` (property-based via `proptest` on the classifier, integration via `testcontainers` for PG + Kafka + Debezium round-trip, chaos via `chaos/scenarios/eel-cvr-*.yaml`).

Runbooks under `docs/runbooks/`:

- `bvr-svc.md` — deploy, rollback, DR, degradation.
- `pgbackrest-pitr.md` — PITR-restore drill (CVR data is in T1-audit, RPO 15 min, RTO 60 min).
- `witness-divergence-response.md` — if the transparency-log witness cosigns diverge on a CVR snapshot.
- `election-day-mode-toggle.md` — CVR ingest is paused during T-30 through T+7 (election freeze window).

Related documents: full spec at `docs/plans/modules/cvr-continuous-voter-register.md`; service capability catalogue at `services/bvr-svc/docs/CAPABILITIES.md`; edge-case matrix rows tagged EC-CVR-* in `docs/plans/08-edge-case-matrix.md`.
