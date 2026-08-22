# Voter ID card (M53)

Module M53 (`voter-id-card`) owned by `bvr-svc`. Manages voter ID card batch production orders to the NRB-contracted manufacturer, tracks individual card collection records, maintains a blacklist with a 30-minute propagation SLA, and integrates card deactivation with the BVR voter register lifecycle.

## Contents

1. What Voter ID Card does (vs M02 BVR backend)
2. Data plane (schema)
3. External integrations
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Voter ID Card does (vs M02 BVR backend)

- **M02** = authoritative voter register: stores demographics, biometrics, and status; is the source of truth for who is registered.
- **M53** = physical credential layer on top of M02: batches card production orders, tracks per-voter card state, records collection at constituency offices, and blacklists stolen or invalidated cards.

M53 never modifies the register directly; it reads the verified voter record from M02 and emits `eel.bvr.card_issued.v1` / `eel.bvr.card_blacklisted.v1` events that M02 appends as annotations. A voter without a card remains on the register; a blacklisted card does not deregister the voter.

## 2. Data plane

Four aggregates (per `services/bvr-svc/crates/voter-id-card/src/models.rs` and the M53 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **VoterCard** | Per-voter card record: `voter_id`, `card_number` (manufacturer serial), `batch_id`, `state` (ORDERED / PRINTED / DISPATCHED / COLLECTED / BLACKLISTED / DESTROYED), `issued_at`, `collected_at`, `blacklisted_reason`. | A |
| **CardBatch** | Production order to manufacturer: `batch_id`, `constituency_code`, `count`, `order_date`, `print_date`, `dispatch_date`, `manifest_hash` (SHA-256 of card-number list), commissioner HSM signature. | A |
| **CollectionRecord** | Constituency-office collection event: `voter_id`, `card_number`, `collected_by_officer`, `id_verified_method` (BIOMETRIC / OTP / MANUAL), `location_code`, `collected_at`. | B |
| **Blacklist** | Stolen or invalidated cards: `card_number`, `reason` (REPORTED_STOLEN / LOST / VOTER_DEATH / OFFICER_REVOCATION), `blacklisted_at`, `propagated_at`. Propagation SLA: 30 minutes to all BVI nodes. | A |

Migrations under `services/bvr-svc/migrations/`.

## 3. External integrations

| Integration | Direction | Auth | SLA | Failure handling |
|---|---|---|---|---|
| **NRB manufacturer** (batch orders) | Outbound SFTP + PGP | SVID mTLS + PGP key exchange | 48 h acknowledgement | Retry 3x, alert if unacknowledged |
| **NRB manufacturer** (delivery manifest) | Inbound SFTP | SVID mTLS | On dispatch | Halt distribution if manifest hash mismatches |
| **BVI nodes** (blacklist propagation) | Push via Kafka topic `eel.bvr.card_blacklisted.v1` | Kafka SASL/SCRAM + TLS | 30 min end-to-end | Circuit breaker via `packages/eel-resilience-rs`; BVI falls back to central blacklist check |
| **M02 BVR** (voter verification) | Internal gRPC | SVID mTLS | Synchronous | Hard fail: no card issued without verified voter |

## 4. Pipeline

Files on disk under `services/bvr-svc/crates/voter-id-card/src`: `lib.rs`, `models.rs`, `batch.rs`, `collection.rs`, `blacklist.rs`, `routes/`.

```
commissioner triggers batch order (constituency + count)
      -> validate: constituency exists in M18 boundary registry
      -> validate: voter IDs in batch have REGISTERED status in M02
      -> create CardBatch with draft state; HSM signature by commissioner
      -> emit eel.bvr.card_batch_ordered.v1 to manufacturer SFTP
      -> manufacturer prints and dispatches; inbound manifest arrives
      -> validate manifest_hash against batch; update batch DISPATCHED
      -> constituency officer records collection
            -> biometric / OTP / manual ID verification
            -> create CollectionRecord; set VoterCard.state = COLLECTED
            -> emit eel.bvr.card_issued.v1
      -> if stolen/lost report arrives:
            -> create Blacklist entry; emit eel.bvr.card_blacklisted.v1
            -> Kafka propagates to all BVI nodes within 30 min SLA
            -> monitor eel.bvr.blacklist_propagated.v1 per-node ack
```

Every step transits the outbox (`packages/eel-outbox-rs`). Debezium picks up outbox rows and publishes to Kafka.

## 5. API surface

Contract: `packages/openapi-contracts/bvr/voter-id-card.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/cards/batches` | Commissioner + HSM | Create production batch order |
| `GET /v1/cards/batches/{id}` | Commission admin | Batch status + manifest verification |
| `POST /v1/cards/batches/{id}/manifest` | NRB system (SVID) | Ingest delivery manifest |
| `POST /v1/cards/collections` | Constituency officer (JWT) | Record voter card collection |
| `GET /v1/cards/voters/{voter_id}` | Commission officer | Card state for voter |
| `POST /v1/cards/blacklist` | Commission officer (JWT + SpiceDB `CARD_BLACKLIST` role) | Blacklist a card |
| `GET /v1/cards/blacklist/{card_number}` | BVI node (SVID) | Real-time blacklist check |
| `GET /v1/cards/blacklist/export` | BVI sync job (SVID) | Bulk blacklist for offline BVI nodes |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Commissioner batch panel (Django) | `services/bvr-svc/django/card_admin` | Order batches, review manifests, sign with HSM |
| Constituency collection desk (LiveView) | `services/bvr-svc/ex/card_collection_web` | Officer records collection; biometric prompt |
| Blacklist management console (Django) | `services/bvr-svc/django/card_admin` | Report stolen cards; view propagation status |

Languages: EN + SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Manifest hash verification** (SHA-256) on every inbound delivery: if `manifest_hash` in `CardBatch` mismatches the computed hash of the delivered card-number list, the batch is quarantined and the commissioner is paged.
- **Biometric gate at collection**: collection defaults to biometric verification via ABIS (M03); OTP and manual are fallback paths requiring officer supervisor countersignature.
- **30-minute blacklist SLA**: Kafka consumer-group lag is monitored via Prometheus alert `eel_card_blacklist_propagation_lag_seconds > 1800`; PagerDuty fires if breached.
- **HSM-signed batch orders**: manufacturer cannot claim an order was never placed; the signed `CardBatch` is the canonical proof of order.
- **Audit shadow**: every card state transition logged to `eel_bvr_audit_shadow` with officer identity, timestamp, and IP.
- **No orphan blacklists**: the blacklist entry includes a back-reference to the `VoterCard` row; orphan blacklist entries fail a nightly integrity check.

## 8. Kenya statute alignment

- **Elections Act §30 (voter identification)**: card issuance is optional where the register is the primary identification instrument; M53 is feature-flagged and activated only when IEBC formally adopts photo cards.
- **Kenya Data Protection Act §25 (lawful processing)**: card production data shared with NRB manufacturer is governed by a data-processing agreement; no biometrics leave ABIS.
- **Kenya Data Protection Act §26 (data minimisation)**: manufacturer receives card number + photo reference only, not full voter demographics.
- **Elections (Technology) Regulations 2017 Regulation 8**: blacklist propagation to all BVI nodes must complete before polls open on election day.

## 9. Honest caveats + external prerequisites

- Feature flag: M53 is disabled by default. IEBC must issue a formal decision to activate card issuance before deployment proceeds beyond staging.
- NRB card manufacturer contract is an external procurement. The SFTP adapter and manifest schema are ready; the vendor's exact export format locks at first live batch.
- Biometric collection relies on ABIS (M03) uptime. If ABIS is unavailable, collection falls to manual with supervisor countersignature; this is a weaker assurance path.
- 30-minute blacklist SLA assumes Kafka topic replication factor >= 3 and BVI nodes have working Kafka consumers. Offline BVI kits (election-day disconnection) must receive a full blacklist dump via `GET /v1/cards/blacklist/export` at kit initialisation.
- Lost-card self-reporting by voters is not built; the current flow requires an officer to initiate blacklisting after receiving a citizen report through a separate channel.

## 10. Test coverage + operational runbooks

Coverage under `services/bvr-svc/crates/voter-id-card/tests/`: manifest hash verification (property-based via `proptest`), collection flows for each verification method, blacklist propagation round-trip (testcontainers PG + Kafka), 30-minute SLA regression (chaos via `chaos/scenarios/eel-card-blacklist-*.yaml`).

Runbooks under `docs/runbooks/`:

- `bvr-svc.md` (section: voter-id-card) -- deploy, rollback, blacklist propagation drain.
- `card-blacklist-sla-breach.md` -- steps when propagation lag exceeds 30 min before election day.
- `manifest-mismatch-quarantine.md` -- quarantine procedure when manufacturer delivery fails hash check.

Related documents: full spec at `docs/plans/modules/voter-id-card.md`; service capability catalogue at `services/bvr-svc/docs/CAPABILITIES.md`; edge-case matrix rows tagged EC-VIC-* in `docs/plans/08-edge-case-matrix.md`.

### Key metrics and SLOs

| Metric | Target | Alert threshold |
|---|---|---|
| Blacklist propagation latency | <= 30 min end-to-end | > 25 min: warning; > 30 min: PagerDuty |
| Manifest hash mismatch rate | 0% | Any mismatch: immediate quarantine |
| Collection biometric success rate | >= 95% | < 90%: alert to constituency manager |
| API p99 latency (collection endpoint) | <= 500 ms | > 1 s: alert |

### Dependency map

```
M02 BVR backend  --(voter verified?)--> M53 VoterCard creation
M03 ABIS         --(biometric check)-->  M53 CollectionRecord
M53 Blacklist    --(propagate via Kafka)--> All BVI nodes (M06)
M39 Ballot Chain --(serial awareness)--> (no direct link; card serial != ballot serial)
NRB manufacturer --(SFTP manifest)----> M53 CardBatch delivery
```

M53 is feature-flagged: the entire module activates only when `feature_flags.voter_id_card_enabled = true` in `services/bvr-svc/config/features.toml`. All API endpoints return HTTP 503 with `FEATURE_DISABLED` when the flag is off.
