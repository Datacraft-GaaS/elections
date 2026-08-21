# M02 bvr-backend

| field | value |
|---|---|
| M-number | M02 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-3.5 (docs/plans/04-wave-3-critical-path.md) |
| Size | XL |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M02-bvr-backend.md |

## Purpose

Central authoritative store of the Kenya voter register. Ingests enrolment records from BVR device clients, orchestrates ABIS dedup and adjudication, publishes the register at each electoral scope, and provides back-office tools for Returning Officers.

## Surfaces and languages

| surface | technology | path under services/bvr-svc/ |
|---|---|---|
| Ingest API + gRPC | Rust (axum + tonic + utoipa) | crates/bvr-api/ |
| Domain logic | Rust | crates/bvr-domain/ |
| Projections / read models | Rust | crates/bvr-projections/ |
| Background workers (ABIS queue, export, IPRS) | Rust | crates/bvr-worker/ |
| RO back-office admin | Django on packages/eel-web-py | django/bvr-admin/ |
| ODPC subject-access workflow | Django | django/bvr-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Voter | voter_id (UUID7), biographic_fields (22), status, version, election_scope | A | T1-voter |
| BiometricTemplate | template_id, voter_id, modality, field-encrypted blob, key_id | A | T1-voter (field-encrypted) |
| IDDocumentScan | scan_id, voter_id, mrz_parsed, pdf_ref | A | MinIO |
| AuditEvent | event_id, actor, action, ts_hlc, signed_by | C | T1-audit |
| MerkleRoot | root_hash, seq, signed_by_hsm, escrow_confirmed | C | T1-audit |
| AdjudicationCase | case_id, voter_ids, modality, status, investigator_id | B | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/bvr/backend.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /bvr/v1/enrolments | internal gRPC | SVID mTLS | yes | Kit to backend ingest; signature verified |
| PATCH /bvr/v1/voters/{id}/status | 3 accredited | JWT+SpiceDB | yes | RO status change; biometric-auth gated |
| POST /bvr/v1/transfers | 3 accredited | JWT+SpiceDB | yes | Voter transfer; dual-auth |
| GET /bvr/v1/registers/{scope} | 4 bulk | JWT+SpiceDB | n/a | Scoped register generation; dual-control |
| GET /bvr/v1/voters/{id} | 3 accredited | JWT+SpiceDB | n/a | Single voter lookup; redacted fields enforced |
| POST /bvr/v1/sar | 3 accredited | JWT+SpiceDB | yes | ODPC subject access request |
| POST /bvr/v1/handoff/{token} | 2 live-per-user | JWT | yes | VPRS handoff token redemption |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.enrolment_accepted.v1 | emits | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.enrolment_rejected.v1 | emits | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.voter_status_changed.v1 | emits | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.register_frozen.v1 | emits | eel.bvr.voter.v1 | event-source | yes | exactly-once |
| eel.bvr.adjudication_opened.v1 | emits | eel.bvr.adjudication.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.adjudication_resolved.v1 | emits | eel.bvr.adjudication.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.merkle_root_published.v1 | emits | eel.bvr.audit.v1 | event-source | yes | exactly-once |
| eel.abis.match_result.v1 | consumes | eel.abis.match.v1 | operational | yes | at-least-once idempotent |
| eel.bve.enrolment_captured.v1 | consumes | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| ABIS (M03) | internal | integrations/abis/ | yes | circuit breaker; dedup queue accumulates; alert if backlog >1 h |
| IPRS | I01 | integrations/iprs/ | yes | fail-closed on new enrolments after 24 h down; reads from cache |
| HSM (P06) | I07 | integrations/hsm-pkcs11/ | yes (SoftHSM) | STH pauses; ingest continues; countersign queue drains on restore |
| Transparency log (P05) | internal | integrations/tlog/ | yes | Merkle root queued; published on restore |
| BI Power BI | I04 | integrations/bi-mart/ | yes | async only; degraded to stale snapshot |
| MDM (O01) | internal | integrations/mdm/ | yes | electoral-area propagation queued |

## ML and AI

none

## Dependencies

- Modules: M01 (device client, ingest source), M03 (ABIS dedup), M04 (VPRS handoff), M07 (BVV register reads)
- Platform: P04 (IAM/SVID), P05 (audit/Merkle), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

1. FR-M02-001: ingest enrolment records from kits via mTLS + kit-certificate authentication.
2. FR-M02-002: verify per-record signature and digital-certificate binding to kit.
3. FR-M02-003: store voter record and field-encrypted biometric templates with separate key per field.
4. FR-M02-004: enqueue ABIS 1:N dedup check on every ingest.
5. FR-M02-005: route ABIS match/no-match/below-quality outcomes to adjudication workflow.
6. FR-M02-006: investigator UI side-by-side minutiae comparison with biometric-auth approval.
7. FR-M02-007: IPRS integration on ingest; cache last-known status; fail-closed after 24 h down.
8. FR-M02-008: RO back-office status lifecycle: ACTIVE/DECEASED/DELETED/TRANSFERRED/SUSPENDED with biometric auth and reason code.
9. FR-M02-009: voter transfer: dual-auth (RO + originating voter fingerprint); polling-station reassigned.
10. FR-M02-010: gazetted cut-off enforcement: register freezes; snapshot published to escrow.
11. FR-M02-011: register generation at any scope (station/ward/constituency/county/national) in PDF + CSV.
12. FR-M02-012: bulk export: PDF + CSV; dual-control approval + audit entry + DLP rate-limit.
13. FR-M02-013: adjudication investigations queue with side-by-side comparison and resolution workflow.
14. FR-M02-014: electoral-area mapping propagation to kits via MDM channel.
15. FR-M02-015: configurable max voters per polling station (default 700).
16. FR-M02-016: voter streaming/sorting per station for register generation.
17. FR-M02-017: QR/barcode generation per voter record for BVI-day lookup.
18. FR-M02-018: per-kit monitoring: SNMP + dashboard alerts on processes/logs/disk/memory.
19. FR-M02-019: async BI mart publication to Power BI.
20. FR-M02-020: ODPC SAR workflow: retrieve, rectify, erase, port per Kenya DPA 2019.
21. FR-M02-021: retention scheduler enforcing 5-year retention on voter data.
22. FR-M02-022: every state change signed by originator + countersigned by HSM + Merkle-logged + escrow-mirrored.
23. Ingest throughput 1 000 records/sec sustained; 30M record generation <= 30 min.
24. Versioned voter rows: each mutation appends a new version; no in-place delete.
25. Bulk export DLP: >1 000 rows blocked without dual-control; rate-limited; audit entry mandatory.
26. Register freeze is atomic across all in-flight writes for the election scope (TLA+ invariant).
27. ABIS backlog alert fires if dedup queue exceeds 30 min of capacity.
28. ODPC SAR fulfilled with full erasure proof within 30 days.
29. Escrow mirror lag alert fires if mirror is > 5 min behind primary.
30. All read paths from RO back-office enforce scoped access via SpiceDB.
31. Adjudication SLA: first response <= 4 h; resolution <= 24 h; timer visible in UI.
32. Admin export produces reproducible PDF matching the gazette register format.

## Acceptance criteria

AC-M02-01: ingest 1 000 records/sec sustained 10 min without loss (load test).
AC-M02-02: ABIS investigation loop closes end-to-end within 24 h SLA (UAT).
AC-M02-03: IPRS failure fails-closed on new enrolments; open on reads (chaos).
AC-M02-04: 30M record register generation <= 30 min (bench).
AC-M02-05: bulk export >1 000 rows blocked without dual approval (policy test).
AC-M02-06: every state change signed + Merkle-logged + escrow-mirrored (audit trace).
AC-M02-07: SAR fulfilled within 30 days (workflow test).
AC-M02-08: gazetted cut-off freezes writes and snapshots to escrow (time-based test).
AC-M02-09: 99.99% availability sustained over 30-day SLO window (SLO measurement).
AC-M02-10: register freeze is atomic; no partial writes under concurrent load (TLA+ + chaos).

## Applicable edge cases

**connectivity**
EC-CON-04: primary/mirror partition; Patroni fencing; no split-brain; RPO <= 5 min.
EC-CON-11: Kafka ISR shrink; producer blocks; outbox accumulates; no loss.

**time**
EC-TIM-03: backend clock jumps backward; HLC never regresses; skew alert.
EC-TIM-06: event time < causation time; accepted; audit skew anomaly.
EC-TIM-09: register freeze race T-60/T-30; OPA hard-close at tx time; all-or-nothing.

**identity**
EC-IDN-03: SVID rotates during batch; old SVID valid to expiry.
EC-IDN-09: dual-control approver revoked mid-flow; re-validate both at commit.
EC-IDN-10: SpiceDB down; fail closed writes; class-C reads only.

**data**
EC-DAT-01: duplicate ingest same key+body; cached response; no re-execution.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-07: partial batch failure; per-row report; valid rows committed.
EC-DAT-08: 22-30M row ops; streaming; checkpoint-resumable; bounded memory.
EC-DAT-09: Unicode names; NFC; ICU collation.
EC-DAT-12: Debezium slot lag; alert 60 s; outbox retained; disk guard.
EC-DAT-13: soft-deleted voter in search; tombstone honoured.
EC-DAT-15: IPRS returns two persons; AdjudicationCase; no auto-merge.

**security**
EC-SEC-01: signature mismatch; never applied; tamper topic; page <= 30 s.
EC-SEC-03: HSM unavailable; STH pauses; ingest continues; countersigns queue.
EC-SEC-04: KEK rotation mid-cycle; online re-wrap; old readable until done.
EC-SEC-05: injection on every input; parameterised only; sqlx compile-time.
EC-SEC-10: voter enumeration; constant-time shape; caps; step-up.
EC-SEC-12: cache poisoning; key includes sig version; client verifies.
EC-SEC-13: split-view on log; witness divergence; consistency fails.
EC-SEC-16: secrets in logs; telemetry refuses Tier A/B.

**scale**
EC-SCL-03: hot partition; stream-id spread; lag alert; HPA.
EC-SCL-04: consumer lag >60 s; page; backpressure; no OOM.
EC-SCL-05: sequencer stall; writes unaffected; cadence alert.
EC-SCL-08: Valkey eviction; PG-durable store; Valkey cache only.

**legal**
EC-LEG-09: T-60 auto-referral race; single referral; idempotent scheduler.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | ingest signature verifier, register-freeze FSM, IPRS circuit-breaker, retention scheduler, DLP block | nextest | M02-UNIT- |
| property | queue never loses a record under arbitrary partition; freeze is atomic; HLC monotonic | proptest | M02-PROP- |
| contract | M01 ingest API; M03 ABIS callback; M07 register-read; VPRS handoff | Pact | M02-PACT- |
| integration | IPRS double; ABIS double; HSM SoftHSM; Merkle log testcontainer | nextest + testcontainers | M02-INT- |
| e2e | full enrolment-to-register-generation flow; freeze + snapshot | nextest | M02-E2E- |
| fuzz | ingest payload; MRZ; Unicode fields; bulk export boundary | cargo-fuzz | M02-FUZZ- |
| formal | register-freeze atomicity TLA+; Merkle append-only kani | TLC + kani | M02-FORMAL- |
| load | 1 000 records/sec 10 min; 30M generation; bulk export | k6 + pg bench | M02-LOAD- |
| chaos | primary kill; IPRS blackhole; HSM kill; Kafka ISR shrink | Chaos Mesh | M02-CHAOS- |
| security | Semgrep; ASVS L3; ZAP; STRIDE threat-model mitigations | Semgrep + ZAP + OWASP | M02-SEC- |

## Runbooks and dashboards

- docs/runbooks/bvr-svc-degradation.md (create): offline IPRS; ABIS backlog; escrow lag; bulk-export DLP fire.
- docs/runbooks/bvr-svc-register-freeze.md (create): cut-off procedure; escrow confirmation; rollback if freeze fails.
- platform/observability/bvr-svc/grafana-ingest.json: ingest rate, ABIS queue depth, adjudication SLA, escrow lag.
- platform/observability/bvr-svc/slo-alerts.yaml: 99.99% burn rate; ingest throughput; IPRS circuit state.

## Open questions

- IPRS variability: IPRS SLA is not contractually defined; circuit-breaker thresholds need agreement with NRB before G3.
- 30M scale: confidential-compute workers for biometric matching (M03) need infrastructure allocation confirmed.
- Escrow custodian identity: the sw/ spec references "escrow mirror" without naming a custodian; needs IEBC decision before G2.
