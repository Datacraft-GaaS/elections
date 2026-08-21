# M10 rts-backend

| field | value |
|---|---|
| M-number | M10 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.1 (docs/plans/04-wave-3-critical-path.md) |
| Size | XL |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M10-rts-backend.md |

## Purpose

Central Results Transmission System back-end: ingest signed form batches from kits and RO stations, run ICR on scanned tallies, compute and publish constituency / regional / national snapshots under HSM-backed signatures, anchor every mutation to the transparency log, and expose the public read surface consumed by the portal and verifier apps.

## Surfaces and languages

| surface | technology | path under services/rts-svc/ |
|---|---|---|
| Ingest + domain API | Rust (axum + tonic + utoipa) | crates/rts-api/ |
| Domain logic + event store | Rust | crates/rts-domain/ |
| ICR inference service | Python (FastAPI + Torch) | py/rts-icr/ |
| Snapshot projections | Rust | crates/rts-projections/ |
| Batch aggregation workers | Rust | crates/rts-batch/ |
| Back-office admin | Django on packages/eel-web-py | django/rts-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Form34A | form_id, station_code, election_id, position_code, raw_scan_ref, extracted_counts, kit_sig, ro_sig, status | C | T1-audit |
| Form34B | form_id, constituency_code, election_id, position_code, station_subtotals, totals, ro_sig, witness_sig | C | T1-audit |
| Form34C | form_id, region_code, election_id, constituency_subtotals, totals, ro_sig | C | T1-audit |
| PublicationSnapshot | snap_id, scope, scope_code, position_code, totals_json, merkle_root, published_at, sig | D | T1-consensus |
| MerkleRoot | root_id, tree_ref, leaf_count, published_at, sig | D | T1-consensus |
| ICRJob | job_id, form_id, model_version, confidence_map, corrected_by, completed_at | C | T2 |
| AuditEvent | event_id, actor, action, ts_hlc, sig, seq | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/rts/backend.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /rts/v1/forms | internal gRPC | SVID mTLS | yes (Idempotency-Key per form) | Kit/RO ingest; verifies kit + RO signature |
| POST /rts/v1/forms/34b | internal gRPC | SVID mTLS | yes | 34B publish from RO desktop |
| POST /rts/v1/forms/34c | internal gRPC | SVID mTLS | yes | 34C publish from RO desktop |
| GET /rts/v1/snapshots | 1 signed-static | public | n/a | Latest national snapshot; cacheable |
| GET /rts/v1/snapshots/{scope}/{code} | 1 signed-static | public | n/a | Constituency/regional snapshot; cacheable |
| GET /rts/v1/forms/{form_id}/proof | 2 live-per-user | JWT | n/a | Inclusion proof for form in Merkle tree |
| POST /rts/v1/icr/{form_id} | internal gRPC | SVID mTLS | yes | Trigger ICR job for a form |
| GET /rts/v1/icr/{job_id}/result | internal gRPC | SVID mTLS | n/a | Poll or stream ICR result |
| GET /rts/v1/audit/{form_id} | 3 accredited | JWT+SpiceDB | n/a | Full audit trail for a form |
| GET /rts/v1/tokens/{token_sha}/proof | 2 live-per-user | JWT | n/a | Participation token inclusion proof |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.rts.form_ingested.v1 | emits | eel.rts.form.v1 | event-source | yes (HSM) | exactly-once |
| eel.rts.icr_completed.v1 | emits | eel.rts.form.v1 | event-source | yes | exactly-once |
| eel.rts.form34b_accepted.v1 | emits | eel.rts.form.v1 | event-source | yes (HSM) | exactly-once |
| eel.rts.form34c_accepted.v1 | emits | eel.rts.form.v1 | event-source | yes (HSM) | exactly-once |
| eel.rts.snapshot_published.v1 | emits | eel.rts.snapshot.v1 | event-source | yes (HSM) | exactly-once |
| eel.rts.merkle_root_anchored.v1 | emits | eel.rts.merkle.v1 | event-source | yes | exactly-once |
| eel.rts.form_captured.v1 | consumes | eel.rts.form.v1 | operational | yes | exactly-once |
| eel.rts.transmission_succeeded.v1 | consumes | eel.rts.form.v1 | operational | yes | exactly-once |
| eel.rts.form34b_published.v1 | consumes | eel.rts.form.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| RTS device (M08) | internal | integrations/rts-device/ | yes | backpressure; outbox accumulates |
| RO desktop (M09) | internal | integrations/rts-ro/ | yes | outbox accumulates |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on publish; snapshot queued |
| Transparency log (P05) | internal | integrations/tlog/ | yes | anchoring queued; retry |
| Public portal (M11) | internal | integrations/public-portal/ | yes | push on snapshot; circuit breaker |
| ICR engine (py/rts-icr) | internal | integrations/icr/ | yes (stub model) | graceful fallback to manual |

## ML and AI

ICR (intelligent character recognition) pipeline:
- Model: fine-tuned TrOCR on Kenyan handwriting samples in Kiswahili and English digits.
- Inference: Python FastAPI worker (py/rts-icr/), GPU-optional, batched.
- Output: per-cell count with confidence score; cells below threshold flagged for manual adjudication by RO (M09).
- Training feedback: RO corrections streamed as training signals to model registry (M28 AI monitoring).
- Data class: ICR job artifacts are class C; raw form images are class C (never class A).

## Dependencies

- Modules: M08 (kit form source), M09 (RO adjudication), M11 (public output), M14 (reproducibility bundle), M21 (RLA draws from this store)
- Platform: P05 (audit/Merkle), P06 (HSM signing)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs

## Feature checklist

1. FR-M10-001: ingest signed Form 34A from kits; verify per-form kit signature before storing.
2. FR-M10-002: run ICR on raw scan; return confidence map; flag low-confidence cells.
3. FR-M10-003: accept RO-corrected counts from M09; update ICRJob record.
4. FR-M10-004: accept and verify dual-officer signed Form 34B/34C from M09.
5. FR-M10-005: compute constituency totals; assemble national snapshot.
6. FR-M10-006: publish snapshots signed by HSM at national/regional/constituency scope.
7. FR-M10-007: anchor every published snapshot to transparency log with Merkle root.
8. FR-M10-008: expose public read API (tier 1 signed-static) for portal and verifier apps.
9. FR-M10-009: inclusion proof API: prove form or participation token is in Merkle tree.
10. FR-M10-010: admin API: full audit trail per form; dispute history.
11. FR-M10-011: push snapshot events to M11 public portal on publication.
12. FR-M10-012: participation token batch ingest from M08; cross-reference against BVI for consistency.
13. FR-M10-013: multi-position: simultaneous pipelines for all elective positions.
14. FR-M10-014: TLA+ formal model: no count can decrease; snapshot monotonically advances.
15. FR-M10-015: RTO <= 30 min; RPO <= 5 min; active-active primary/replica.
16. Kit form ingest p95 <= 500 ms; p99 <= 1 s at 60k concurrent kits.
17. Snapshot publish p95 <= 5 s from 34B acceptance trigger.
18. ICR job p95 <= 30 s per form; p99 <= 60 s.
19. Public snapshot read p95 <= 200 ms; p99 <= 500 ms at 5M req/hr.
20. 100% of transmitted forms have inclusion proof in final Merkle tree.
21. No snapshot can be published with outstanding unresolved disputes.
22. Kafka partition count provisioned for 100k events/s.
23. ICR model version pinned per election cycle; upgrade requires model-registry approval.
24. Zero form drops on primary fail (event-source store on replica before ack).

## Acceptance criteria

AC-M10-01: kit form ingest p95 <= 500 ms at 60k concurrent kits (load test).
AC-M10-02: snapshot published within 5 s of 34B acceptance (integration test).
AC-M10-03: inclusion proof verifiable by M11 and M13 (contract test).
AC-M10-04: TLA+ invariants: count never decreases; snapshot monotone (formal verification).
AC-M10-05: primary kill; replica active within RPO <= 5 min (chaos test).
AC-M10-06: ICR corrections captured as training signals without exposing PII (security review).
AC-M10-07: participation token cross-reference consistent with BVI data (integration test).
AC-M10-08: 99.99% availability over tallying window (SLO measurement).

## Applicable edge cases

**connectivity**
EC-CON-04: primary/replica partition; no split-brain; RPO <= 5 min.
EC-CON-11: Kafka ISR shrink; producer blocks; outbox accumulates.
EC-CON-07: high-latency satellite kit; transport timeouts tuned; MTU-safe.

**time**
EC-TIM-03: backend clock backward; HLC never regresses.
EC-TIM-07: extended tallying window; HSM-signed extension event.

**identity**
EC-IDN-01: kit cert revoked; reject form from that kit immediately.
EC-IDN-03: SVID rotates during ingest; old SVID valid to expiry.
EC-IDN-09: dual-officer 34B: re-validate both at commit.
EC-IDN-10: SpiceDB down; fail closed accredited writes; tier-1 reads only.

**data**
EC-DAT-01: duplicate form same key+body; cached.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-04: replayed signed event; reject on counter.
EC-DAT-05: extracted counts > registered voters; block; DLQ with reason.
EC-DAT-14: oversized scan > 1 MB; reject at boundary.

**security**
EC-SEC-01: signature mismatch; never applied; tamper alert.
EC-SEC-03: HSM unavailable; snapshot queued; ingest continues.
EC-SEC-12: cache poisoning of public snapshot; key includes sig version + Merkle root.
EC-SEC-13: split-view on log; witness divergence.

**scale**
EC-SCL-03: hot partition; stream-id spread; lag alert; HPA.
EC-SCL-04: consumer lag > 60 s; backpressure; no OOM.

**ML**
EC-ML-01: ICR model drift mid-election; version pinned; correction rate alert fires.
EC-ML-02: adversarial form scan (white-box attack); confidence < threshold; mandatory manual.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | form ingestion, signature verification, ICR trigger, snapshot assembly, Merkle computation | nextest | M10-UNIT- |
| property | count never decreases; inclusion proof valid for all ingested forms; HLC monotonic | proptest | M10-PROP- |
| contract | M08 kit producer; M09 RO publisher; M11 snapshot consumer; M13 proof consumer | Pact | M10-PACT- |
| integration | SoftHSM signing; ICR stub; tlog double; BVI participation token | nextest + testcontainers | M10-INT- |
| e2e | kit form -> ingest -> ICR -> 34B dual-sign -> snapshot -> portal visible | patrol + Playwright | M10-E2E- |
| fuzz | form payload; ICR correction; snapshot JSON | cargo-fuzz | M10-FUZZ- |
| formal | TLA+ spec: count monotone; no publish with unresolved dispute | TLA+ / kani | M10-FORMAL- |
| load | 60k concurrent kits; 5M req/hr read; ICR backlog | k6 + Gatling | M10-LOAD- |
| chaos | primary kill; Kafka ISR shrink; HSM kill; ICR worker crash | Chaos Mesh | M10-CHAOS- |
| security | ASVS L3; STRIDE on snapshot publish; Semgrep; signature replay | Semgrep + OWASP | M10-SEC- |
| ML | ICR accuracy on held-out Kenyan handwriting set; adversarial form | pytest + custom | M10-ML- |

## Runbooks and dashboards

- docs/runbooks/rts-svc-backend-degradation.md (create): primary fail; HSM fail; ICR worker crash; Kafka lag remediation.
- platform/observability/rts-svc/grafana-rts-backend.json: ingest rate, ICR queue depth, snapshot lag, Merkle anchor rate, public read latency.
- platform/observability/rts-svc/slo-alerts.yaml: 99.99% burn rate; ingest p95 > 500 ms; snapshot publish > 5 s; ICR queue > 30 min.

## Open questions

- TLA+ scope: the formal model covers count monotonicity and no-publish-with-dispute; completeness of the model relative to the Elections Act s.39 needs legal review.
- ICR handwriting samples: the training corpus needs IEBC sign-off on provenance and consent before use; current plan is to use synthetic + historical forms.
- Kafka capacity: 100k events/s at peak tallying; partition count and retention size need capacity planning before G3.
