# M03 abis

| field | value |
|---|---|
| M-number | M03 |
| Home service | abis-svc (ADR-0020) |
| Owning work package | WP-3.6 (docs/plans/04-wave-3-critical-path.md) |
| Size | XL |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M03-abis.md |

## Purpose

Deduplicate the Kenya voter register at 30M+ scale using multi-modal biometric matching (fingerprint, iris, facial) inside confidential-compute enclaves. Feeds adjudication queues for ambiguous matches and provides the register integrity guarantee that no citizen is enrolled twice.

## Surfaces and languages

| surface | technology | path under services/abis-svc/ |
|---|---|---|
| Match API + gRPC | Rust (axum + tonic + utoipa) | crates/abis-api/ |
| Enclave matching workers | Rust inside TDX/SEV-SNP | crates/abis-domain/ |
| Adjudication workbench | Django on packages/eel-web-py | django/abis-admin/ |
| Batch dedup job | Rust worker | crates/abis-worker/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| TemplateIndex | index_id, voter_id, modality, encrypted_feature_vector, enclave_attestation_id | A | T1-voter (field-encrypted, enclave-only) |
| MatchRequest | request_id, voter_id, modalities, status, enqueued_at | C | T2 |
| MatchResult | result_id, request_id, outcome, confidence_score, modality_scores | C | T1-audit |
| AdjudicationCase | case_id, match_result_id, voter_ids, status, investigator_id, decision, reason_code | B | T2 |
| EnclaveAttestation | enclave_id, worker_id, tee_quote, verified_at | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/abis/match.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /abis/v1/match/1n | internal gRPC | SVID mTLS | yes | 1:N match request from bvr-svc |
| POST /abis/v1/match/11 | internal gRPC | SVID mTLS | yes | 1:1 verification request |
| GET /abis/v1/match/{request_id} | internal gRPC | SVID mTLS | n/a | Poll match result |
| GET /abis/v1/adjudication | 3 accredited | JWT+SpiceDB | n/a | Investigator case queue |
| PATCH /abis/v1/adjudication/{case_id} | 3 accredited | JWT+SpiceDB | yes | Record adjudication decision |
| POST /abis/v1/batch/dedup | 3 accredited | JWT+SpiceDB | yes | Trigger full-register dedup job |
| GET /abis/v1/reports | 3 accredited | JWT+SpiceDB | n/a | FAR/FRR and throughput reports |
| GET /abis/v1/certifier/testset | 3 accredited | JWT+SpiceDB | n/a | Independent certifier interface |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.abis.match_result.v1 | emits | eel.abis.match.v1 | operational | yes (Ed25519) | at-least-once idempotent |
| eel.abis.adjudication_opened.v1 | emits | eel.abis.adjudication.v1 | operational | yes | at-least-once idempotent |
| eel.abis.adjudication_resolved.v1 | emits | eel.abis.adjudication.v1 | operational | yes | at-least-once idempotent |
| eel.abis.dedup_batch_completed.v1 | emits | eel.abis.batch.v1 | analytics | yes | at-least-once idempotent |
| eel.bvr.enrolment_accepted.v1 | consumes | eel.bvr.voter.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| BVR back-end (M02) | internal | integrations/bvr-backend/ | yes | match result queued; bvr-svc retries |
| Confidential compute (TDX/SEV-SNP) | internal | crates/abis-enclave/ | yes (software simulation) | worker quarantined; redistributed to healthy enclaves |
| Legacy IEBC ABIS | I05 | integrations/legacy-abis/ | yes (mock) | federation adapter fails closed; in-house matcher used |
| HSM (P06) | I07 | integrations/hsm-pkcs11/ | yes (SoftHSM) | signing queued; alert fired |
| Transparency log (P05) | internal | integrations/tlog/ | yes | anchoring queued on restore |

## ML and AI

- Multi-modal biometric matcher: fingerprint (ISO 19794-2 minutiae + NBIS BOZORTH3 variant compiled to Rust), iris (ISO 19794-6 Hamming distance), facial (ISO 19794-5 + FERET-aligned CNN embedding). All models run inside TDX/SEV-SNP enclave; no unencrypted templates leave enclave memory.
- Multi-modal fusion policy: configurable score-level fusion per IEBC-approved thresholds; thresholds tuned to FAR <= 0.001% FNMR <= 0.1%.
- Batch quality filter: NFIQ2 < 20 returns "insufficient" without matching.

## Dependencies

- Modules: M02 (BVR back-end, match requests)
- Platform: P04 (IAM/SVID), P05 (audit/Merkle), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, eel-biometric-rs, eel-web-py

## Feature checklist

1. FR-M03-001: 1:N fingerprint (10 fingers), iris (both eyes), facial against >= 30M records.
2. FR-M03-002: 1:1 verification against a specific voter ID.
3. FR-M03-003: multi-modal fusion with configurable score-level policy.
4. FR-M03-004: match/no-match/adjudicate outcome per configurable thresholds.
5. FR-M03-005: adjudication queue with minutiae overlay, side-by-side comparison, decision logging.
6. FR-M03-006: investigator workbench: search by biometric, ID, name; case SLA timer.
7. FR-M03-007: below-threshold quality returns "insufficient"; above-threshold ambiguity returns "adjudicate".
8. FR-M03-008: FBI IAFIS IQS + EBTS IAFIS-DOC-01078-9.0; ISO 19794-2/5/6; ANSI/NIST-ITL.
9. FR-M03-009: all matching inside TDX/SEV-SNP enclave; templates never leave enclave memory.
10. FR-M03-010: on each new enrolment, 1:N against full register; rejected records retain no template.
11. FR-M03-011: optional legacy ABIS federation adapter with fallback to in-house matcher.
12. FR-M03-012: scheduled full-register dedup batch (on-demand or quarterly).
13. FR-M03-013: reports: dedup counts, FAR/FRR, throughput, adjudication SLA compliance.
14. FR-M03-014: test-set interface for independent third-party certifier.
15. FR-M03-015: every match request + outcome signed + audit-logged.
16. 1:N fingerprint p99 <= 8 s; iris p99 <= 5 s; facial p99 <= 6 s at 30M records.
17. 100+ concurrent 1:N queries sustained.
18. Adjudication first response <= 4 h; resolution <= 24 h.
19. Enclave attestation verified on every enclave; failed worker quarantined.
20. FAR <= 0.001% FNMR <= 0.1% on IEBC test set (per appendix SLA: FMR <= 0.001%).
21. Batch full-register dedup job is resumable from last checkpoint on interruption.
22. Match results are immutable after issuance; disputes create new AdjudicationCase.
23. Per-group fairness metrics tracked per-modality across demographic strata.

## Acceptance criteria

AC-M03-01: 1:N against 30M synthetic records fingerprint p99 <= 8 s (bench).
AC-M03-02: FAR <= 0.001% + FRR <= 0.1% on IEBC test set confirmed by independent certifier.
AC-M03-03: confidential compute enclave attestation verified on all workers (live attestation report).
AC-M03-04: adjudication workflow closes all cases within 24 h SLA (UAT).
AC-M03-05: standards compliance package accepted by independent certifier.
AC-M03-06: 100 concurrent 1:N queries sustained at p99 <= 8 s (load test).
AC-M03-07: no unencrypted template observable outside enclave memory (attestation + audit review).
AC-M03-08: batch dedup is resumable from checkpoint after kill-resume (chaos test).

## Applicable edge cases

**connectivity**
EC-CON-04: primary/mirror partition; matching redistributed to healthy workers; no result loss.
EC-CON-11: Kafka ISR shrink; match-result outbox accumulates; alert.

**data**
EC-DAT-01: duplicate match request same key+body; cached response.
EC-DAT-07: partial batch dedup failure; per-record report; clean rows committed.
EC-DAT-08: 30M row full-register dedup; streaming; checkpoint-resumable.

**security**
EC-SEC-03: HSM unavailable; signing queued; ingest continues.
EC-SEC-05: injection on API inputs; parameterised only.
EC-SEC-11: timing side-channel on biometric compare; constant-time.
EC-SEC-16: secrets in logs; telemetry refuses Tier A/B.

**scale**
EC-SCL-03: hot partition on match queue; stream-id spread.
EC-SCL-04: consumer lag >60 s; backpressure; no OOM.

**ML**
EC-ML-01: model drift; drift alert; review ratio rises; card versioned.
EC-ML-02: adversarial biometric input; confidence gate; no crash.
EC-ML-04: bias across demographic groups; per-group FAR/FRR in tolerance; in model card.
EC-ML-06: wrong/unsigned model served; cosign-verified; enclave refuses.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | score fusion logic, threshold logic, quality gate, SLA timer, enclave attestation parser | nextest | M03-UNIT- |
| property | match never produces false accept above configured FAR threshold on synthetic corpus | proptest | M03-PROP- |
| contract | bvr-svc match-request producer; match-result consumer bvr-svc | Pact | M03-PACT- |
| integration | enclave software simulation; legacy ABIS mock; HSM SoftHSM | nextest + testcontainers | M03-INT- |
| e2e | end-to-end enrolment -> 1:N match -> adjudication resolution | nextest | M03-E2E- |
| fuzz | template payload; API inputs; batch manifest | cargo-fuzz | M03-FUZZ- |
| load | 100 concurrent 1:N at 30M records; p99 <= 8 s | k6 + loadtest/ | M03-LOAD- |
| chaos | enclave kill; HSM kill; Kafka partition; primary kill | Chaos Mesh | M03-CHAOS- |
| security | ASVS L3; STRIDE mitigations; enclave side-channel analysis; Semgrep | OWASP + Semgrep | M03-SEC- |

## Runbooks and dashboards

- docs/runbooks/abis-svc-degradation.md (create): enclave quarantine; matching redistribution; adjudication backlog escalation.
- docs/runbooks/abis-svc-certifier.md (create): independent certifier test-set procedure.
- platform/observability/abis-svc/grafana-match.json: 1:N latency p99, concurrent queries, FAR/FRR trend, adjudication queue depth.
- platform/observability/abis-svc/slo-alerts.yaml: 99.99% burn rate; p99 latency breach; FAR drift alert.

## Open questions

- Vendor-vs-build: clarification C6 (retain existing IEBC ABIS vs full custom build) is unresolved; federation adapter is built regardless but the matching compute allocation differs materially.
- TDX vs SEV-SNP: hardware availability in Kenyan DC needs confirmation; software simulation is the dev target but HIL bench timing must be validated before G3.
- sw/ spec references "Confluent" for Kafka; ADR stack uses self-hosted Kafka; text should be corrected in the spec to remove the vendor name.
