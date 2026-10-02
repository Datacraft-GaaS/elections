# M10 rts-backend

| field | value |
|---|---|
| M-number | M10 |
| Home service | rts-svc (ADR-0020) |
| Owning work package | WP-3.1 (docs/plans/04-wave-3-critical-path.md) |
| Size | XL |
| Availability class | election-critical 99.99% |
| Event-sourced | yes (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M10-rts-backend.md |

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

Form ingest and verification:
1. FR-M10-001: ingest Form 34A submissions from kits via mTLS with kit-certificate and per-form Ed25519 signature verification; reject on any mismatch. (spec FR-M10-001)
2. FR-M10-002: store original signed scan image immutably in object store; hash-indexed in database; deletion prohibited. (spec FR-M10-002)
3. FR-M10-003: sharded ingest per constituency to prevent hot spots; consistent-hash routing by constituency code. (spec FR-M10-003)
4. FR-M10-004: idempotent form receipt: duplicate submission same Idempotency-Key+body returns cached response; no re-processing. (spec, idempotency)
5. FR-M10-005: SMS/USSD fallback receiver: reconstruct Form 34A from compressed SMS submission; verify signature; alert on mismatch. (spec FR-M10-014)
6. FR-M10-006: per-form submission status API: kit and RO can check status (received, queued, extracted, adjudicated, published). (spec FR-M10-013)
7. FR-M10-007: legacy kit pre-CloudEvents payload shim: envelope upgraded; original kit signature preserved.
8. FR-M10-008: Form 34A immutability enforced post-ingest: compensating-event model only; no in-place update.

ICR and extraction:
9. FR-M10-009: ICR extraction: OCR handwritten Form 34A into structured per-candidate-per-position counts; confidence score per field. (spec FR-M10-004)
10. FR-M10-010: ICR runs in confidential compute enclave; enclave attestation report verified before processing batch. (spec FR-M10-005)
11. FR-M10-011: route low-confidence ICR extractions to M09 human review queue; configurable confidence threshold. (spec FR-M10-006)
12. FR-M10-012: random sampling of high-confidence extractions for human quality control; sampling rate configurable. (spec FR-M10-007)
13. FR-M10-013: ICR model version pinned per election cycle; upgrade requires model-registry dual-sign approval. (spec FR-M10, model management)
14. FR-M10-014: ICR model cards published: training data provenance, accuracy metrics, bias report per demographic.
15. FR-M10-015: ICR training signal capture: RO corrections streamed to model-registry; PII stripped before export.
16. FR-M10-016: ICR enclave attestation cached; re-attest on worker restart or model-version change.
17. FR-M10-017: per-form ICR throughput: sustain 100 forms/s at peak; ICR job p95 <= 30 s; p99 <= 60 s. (spec NFR-M10-004)

Aggregation and validation:
18. FR-M10-018: accept RO-corrected counts from M09; verify RO SVID and payload signature before applying. (spec FR-M10-003)
19. FR-M10-019: accept and verify dual-officer signed Form 34B/34C from M09; policy check: right scope and right personnel. (spec FR-M10-009)
20. FR-M10-020: aggregation service assembles Form 34B (constituency totals) and Form 34C (regional/national totals) from confirmed 34As. (spec FR-M10-008)
21. FR-M10-021: validation rules enforced at aggregation: votes <= registered; rejected + valid = total; non-negative counts.
22. FR-M10-022: independent parallel-tally service replays signed events; any drift from primary raises P0 alert. (spec FR-M10-015)
23. FR-M10-023: multi-position pipelines: independent processing for President, Governor, Senator, MP, CWR, MCA, and referenda. (spec FR-M10-011)
24. FR-M10-024: per-position pipeline failure isolation: a crash in one position does not block others.
25. FR-M10-025: dispute handling: disputed form marked in published status; signed image and audit trail available to petitioners. (spec FR-M10-017)
26. FR-M10-026: TLA+-verified state machine: count never decreases; snapshot monotonically advances; no double-count; no lost write. (spec FR-M10-014)

Publication:
27. FR-M10-027: publication service: on M-of-N quorum HSM authorisation, sign snapshot and push to CDN. (spec FR-M10-010)
28. FR-M10-028: Merkle tree anchored on every publication cycle; root appended to transparency log. (spec FR-M10-013)
29. FR-M10-029: escrow mirror: signed snapshot mirrored asynchronously; lag alert if mirror falls behind > configured threshold. (spec FR-M10-010)
30. FR-M10-030: public portal feed: signed snapshots pushed to CDN at defined cadence; portal never reads production database. (spec FR-M10-016)
31. FR-M10-031: publication cadence configurable per election phase; minimum 5 min between snapshots.
32. FR-M10-032: publication blocked if outstanding unresolved disputes exist; ECC alerted.
33. FR-M10-033: snapshot generation <= 5 min per cycle; p95 <= 2 min. (spec NFR-M10-005)

Public read APIs:
34. FR-M10-034: public snapshot read API (tier 1 signed-static): latest national snapshot; CDN-cacheable. (spec FR-M10-016)
35. FR-M10-035: constituency/regional snapshot read API: scoped by code and position; CDN-cacheable.
36. FR-M10-036: inclusion proof API: prove any form or participation token is in the Merkle tree. (spec FR-M10-013)
37. FR-M10-037: public snapshot p95 <= 200 ms; p99 <= 500 ms at 5M req/hr. (spec NFR)
38. FR-M10-038: full audit trail API per form (accredited only): submission events, ICR job, adjudication, publication.

Participation tokens:
39. FR-M10-039: participation token batch ingest from M08; verify batch signature and station provenance. (spec FR-M10-012)
40. FR-M10-040: cross-reference token count against BVI authenticated-voter count for consistency; alert on mismatch.
41. FR-M10-041: token inclusion proof in Merkle tree; verifiable by M13 and M14.

Scale and resilience:
42. FR-M10-042: 60,000 concurrent RTS sessions sustained; p99 auth <= 100 ms. (spec NFR-M10-001, NFR-M10-002)
43. FR-M10-043: active-active HA across primary and mirror DC; consistent-hash load balancer; write to both; failover RTO <= 30 s. (spec NFR-M10-008)
44. FR-M10-044: RTO <= 30 min; RPO <= 5 min; event-source store on replica before ack to kit. (spec NFR-M10-003)
45. FR-M10-045: back-pressure: on ingest overload, kits receive 429 with Retry-After; kits queue and retry.
46. FR-M10-046: Kafka provisioned for 100k events/s; partition count set for peak + 50% headroom. (spec, Kafka SLA)
47. FR-M10-047: zero form drops on primary fail: event-source store on replica before ack.
48. FR-M10-048: Debezium slot lag monitor: alert at 60 s; disk guard prevents WAL bloat.

Security:
49. FR-M10-049: rate limiting, DDoS shielding, and WAF at ingress. (spec FR-M10-012)
50. FR-M10-050: immutable event log with cryptographic tamper-evidence; append-only store. (spec FR-M10-013)
51. FR-M10-051: transparency log split-view detection: witness divergence raises alert; P0 page.
52. FR-M10-052: secrets in logs refused; Semgrep Tier A/B log ban enforced in CI.
53. FR-M10-053: parameterised queries only on all ingest paths; Semgrep sqlx compile-time check.
54. FR-M10-054: HSM unavailable: snapshot queued; ingest continues; degraded-mode alert.
55. FR-M10-055: KEK rotation mid-cycle: online re-wrap; old key readable until done; audit entry.
56. FR-M10-056: cache poisoning prevention on public snapshot: key includes sig version and Merkle root hash.
57. FR-M10-057: SSRF prevention on any URL fields: allowlist; no private ranges; rebinding guard.
58. FR-M10-058: replay protection on all ingest paths: monotonic sequence counter per station; duplicate rejected.

ICR security:
59. FR-M10-059: ICR model integrity: cosign-verified model artefact before load; unsigned model refused.
60. FR-M10-060: adversarial input gate: confidence below threshold routes to mandatory manual review.
61. FR-M10-061: PII stripped from ICR training signals before export; PIIStrippingEvent emitted.

Monitoring and operations:
62. FR-M10-062: Grafana dashboards: ingest rate, ICR throughput, snapshot latency, Merkle anchor lag, escrow lag, dispute queue depth.
63. FR-M10-063: SLO burn-rate alerts: 99.99% availability; p99 ingest; snapshot latency.
64. FR-M10-064: parallel-tally drift alert fires within 60 s of first divergence; P0 page to war room.
65. FR-M10-065: admin API: re-trigger ICR job; re-publish snapshot; manual dispute resolution. (spec FR-M10-010)
66. FR-M10-066: PITR restore drill: RPO/RTO verified quarterly; restore logged to transparency log.
67. FR-M10-067: schema v1/v2 coexistence: both accepted; dual-emit; Karapace compat check enforced in CI.
68. FR-M10-068: out-of-order form delivery: buffer until gap filled or DLQ; projection independent of wall time.
69. FR-M10-069: hot-partition handling: stream-id spread by station_code+position_code; lag alert; HPA.
70. FR-M10-070: cold replay / tiered storage: <= 10 MB/s throttle; snapshot-then-tail; bounded memory.

## Acceptance criteria

AC-M10-01: kit form ingest p95 <= 500 ms; p99 <= 1 s at 60k concurrent kits (load test).
AC-M10-02: snapshot published within 5 s of 34B acceptance (integration test).
AC-M10-03: inclusion proof verifiable by M11 and M13 (contract test).
AC-M10-04: TLA+ invariants: count never decreases; snapshot monotone (formal verification with TLC).
AC-M10-05: primary kill; replica active within RPO <= 5 min; RTO <= 30 min (chaos test).
AC-M10-06: ICR corrections captured as training signals; PII stripped before export (security review).
AC-M10-07: participation token cross-reference consistent with BVI data (integration test).
AC-M10-08: 99.99% availability over tallying window (SLO measurement).
AC-M10-09: parallel-tally drift raises P0 alert within 60 s of divergence (fault-injection test).
AC-M10-10: ICR enclave attestation report verified on each batch; unsigned model refused (attestation test).
AC-M10-11: 100% of transmitted forms have inclusion proof in final Merkle tree (audit trace).
AC-M10-12: no snapshot published with outstanding unresolved disputes; ECC alerted (integration test).
AC-M10-13: SMS/USSD fallback Form 34A reconstructed and signature-verified end-to-end (field trial).
AC-M10-14: Kafka provisioned for 100k events/s; no consumer lag > 60 s under peak load (load test).
AC-M10-15: public snapshot read p95 <= 200 ms; p99 <= 500 ms at 5M req/hr (load test).

## Applicable edge cases

connectivity:
EC-CON-04: primary/mirror site partition; Patroni fencing; no split-brain; MirrorMaker lag alert; RPO <= 5 min.
EC-CON-07: satellite kit with 600 ms+ RTT; transport timeouts tuned; MTU-safe payloads.
EC-CON-11: Kafka ISR shrink; producer blocks; outbox accumulates; no message loss on ISR recovery.

time:
EC-TIM-01: kit clock skew > 5 min; form rejected; kit alerted to resync.
EC-TIM-02: NTP/NTS unreachable; HLC + monotonic fallback; time_confidence=low.
EC-TIM-03: back-end clock jumps backward; HLC never regresses; TimeChaos test.
EC-TIM-04: DST/TZ: all timestamps stored in UTC; EAT display only in read layer.
EC-TIM-05: leap second in event timestamp; parser tolerates 23:59:60.
EC-TIM-06: event time < causation time; accept; audit skew anomaly recorded.
EC-TIM-07: extended tallying window; HSM-signed extension event consumed; fence reopened.
EC-TIM-08: petition window T+7 midnight EAT; s.75(1) inclusive; boundary table verified +-1 s.
EC-TIM-09: register freeze race T-60; OPA hard-close at tx time; all-or-nothing aggregation.

identity:
EC-IDN-01: kit cert revoked mid-session; reject form from that kit immediately; alert war room.
EC-IDN-02: JWT expires mid-request on accredited API; in-flight completes; next 401 + silent refresh.
EC-IDN-03: SVID rotates during ingest batch; old SVID valid to expiry; producer_key_id per event.
EC-IDN-09: dual-officer 34B: re-validate both signers at commit; approver revoked mid-flow causes re-delegation.
EC-IDN-10: SpiceDB down; fail closed on accredited writes; tier-1 signed-static reads continue.
EC-IDN-11: realm JWKS rotated; stale kid; refetch on kid miss; no 401 storm.

data:
EC-DAT-01: duplicate form same key+body; cached response; no re-processing.
EC-DAT-02: same key different body; 422 idempotency-key-reuse.
EC-DAT-03: out-of-order Form 34A delivery; buffer until gap filled or DLQ.
EC-DAT-04: replayed signed event; reject on monotonic counter.
EC-DAT-05: extracted counts > registered voters; block; DLQ with reason; partition continues.
EC-DAT-06: schema v1/v2 coexist in stream; both accepted; dual-emit; Karapace compat check.
EC-DAT-07: partial batch failure in aggregation; per-row report; valid rows committed.
EC-DAT-08: 46k forms x 6 positions; streaming; checkpoint-resumable; bounded memory.
EC-DAT-09: Unicode candidate names; NFC normalisation; grapheme-safe in snapshot JSON.
EC-DAT-10: zero votes for a candidate; valid; renders "0"; no div-by-zero in aggregation.
EC-DAT-11: totals > registered voters in 34B; block; dispute case created.
EC-DAT-12: Debezium slot lag; alert 60 s; outbox retained; disk guard on WAL bloat.
EC-DAT-13: soft-deleted form in audit query; tombstone honoured; purge removes.
EC-DAT-14: oversized scan > 1 MB; reject at ingest boundary; blob -> MinIO + hash.
EC-DAT-15: IPRS or BVI returns two persons for same token; AdjudicationCase; no auto-merge.
EC-DAT-16: legacy kit pre-CloudEvents payload; shim upgrades envelope; kit signature preserved.

security:
EC-SEC-01: signature mismatch on ingest; form never applied; tamper topic; P0 alert.
EC-SEC-03: HSM unavailable; snapshot queued; ingest continues; no gap in event log.
EC-SEC-04: KEK rotation mid-cycle; online re-wrap; old key readable until done; audit entry.
EC-SEC-05: injection on all ingest and admin inputs; parameterised only; Semgrep sqlx clean.
EC-SEC-12: cache poisoning of public snapshot; cache key includes sig version and Merkle root.
EC-SEC-13: split-view on transparency log; witness divergence alert; consistency check fails.
EC-SEC-16: secrets in logs refused; telemetry refuses Tier A/B fields; CI regex enforced.

scale:
EC-SCL-03: hot partition by station_code; stream-id spread; lag alert; HPA triggered.
EC-SCL-04: consumer lag > 60 s; page; backpressure; no OOM on aggregation workers.
EC-SCL-05: sequencer stall; writes unaffected; cadence alert; drain >= peak rate.
EC-SCL-06: tiered-storage lag; cold replay throttled <= 10 MB/s; snapshot-then-tail.

ML:
EC-ML-01: ICR model drift mid-election; version pinned; correction-rate alert fires.
EC-ML-02: adversarial form scan; confidence < threshold; mandatory manual review; no crash.
EC-ML-06: unsigned or un-cosigned model pushed; refused at load; alert.
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
- ICR handwriting samples: the training corpus needs the Commission sign-off on provenance and consent before use; current plan is to use synthetic + historical forms.
- Kafka capacity: 100k events/s at peak tallying; partition count and retention size need capacity planning before G3.
