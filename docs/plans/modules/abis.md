# M03 abis

| field | value |
|---|---|
| M-number | M03 |
| Home service | abis-svc (ADR-0020) |
| Owning work package | WP-3.6 (docs/plans/04-wave-3-critical-path.md) |
| Size | XL |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M03-abis.md |

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
| Legacy the Commission ABIS | I05 | integrations/legacy-abis/ | yes (mock) | federation adapter fails closed; in-house matcher used |
| HSM (P06) | I07 | integrations/hsm-pkcs11/ | yes (SoftHSM) | signing queued; alert fired |
| Transparency log (P05) | internal | integrations/tlog/ | yes | anchoring queued on restore |

## ML and AI

- Multi-modal biometric matcher: fingerprint (ISO 19794-2 minutiae + NBIS BOZORTH3 variant compiled to Rust), iris (ISO 19794-6 Hamming distance), facial (ISO 19794-5 + FERET-aligned CNN embedding). All models run inside TDX/SEV-SNP enclave; no unencrypted templates leave enclave memory.
- Multi-modal fusion policy: configurable score-level fusion per Commission-approved thresholds; thresholds tuned to FMR <= 0.001% FNMR <= 0.1%.
- Batch quality filter: NFIQ2 < 20 returns "insufficient" without matching.

## Dependencies

- Modules: M02 (BVR back-end, match requests)
- Platform: P04 (IAM/SVID), P05 (audit/Merkle), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-resilience-rs, eel-biometric-rs, eel-web-py

## Feature checklist

Matching core
FR-M03-001: 1:N fingerprint match (10 fingers, ISO 19794-2 minutiae) against >= 30M records; p95 <= 5 s, p99 <= 8 s.
FR-M03-002: 1:N iris match (both eyes, ISO 19794-6 Hamming distance) against >= 30M records.
FR-M03-003: 1:N facial match (ISO 19794-5 + FERET-aligned CNN embedding) against >= 30M records.
FR-M03-004: 1:1 verification against a specific voter_id with multi-modal fusion; p99 <= 3 s.
FR-M03-005: multi-modal score-level fusion with configurable policy per Commission-approved thresholds.
FR-M03-006: four match outcomes: match above threshold, no-match, adjudicate (ambiguous), insufficient-quality; all four routed distinctly.
FR-M03-007: configurable thresholds per modality and fusion policy; tuned to FMR <= 0.001%, FNMR <= 0.1% on Commission certified test set.
FR-M03-008: NFIQ2 quality gate: templates below NFIQ2 < 20 rejected with "insufficient" outcome; no match attempted.
FR-M03-009: compliance with ISO/IEC 19794-2/5/6, ANSI/NIST-ITL 1-2011 Part 1, FBI IAFIS IQS, and EBTS IAFIS-DOC-01078-9.0.
FR-M03-010: all matching executed inside TDX/SEV-SNP confidential-compute enclave; no unencrypted template observable outside enclave memory.

Enclave and key management
FR-M03-011: enclave TDX/SEV-SNP attestation quote verified before each worker begins processing; failed attestation triggers quarantine.
FR-M03-012: failed enclave attestation quarantines worker; match requests redistributed to healthy enclaves; alert paged within 60 s.
FR-M03-013: biometric templates field-encrypted at rest with per-field KEK via eel-crypto-rs; plaintext only inside enclave; key never persisted outside HSM.
FR-M03-014: ABIS model artifact verified against cosign signature before load; unsigned or verification-failed model refused at runtime and in CI.
FR-M03-015: every match request and outcome signed (Ed25519), audit-logged to T1-audit, and anchored to transparency log (P05).

Adjudication workbench
FR-M03-016: adjudication queue with minutiae overlay and side-by-side comparison; decision logging with reason code mandatory.
FR-M03-017: investigator workbench: case search by biometric hash, voter ID, or name; SLA timer visible (first response <= 4 h, resolution <= 24 h).
FR-M03-018: investigator biometric-auth approval required to record any adjudication decision.
FR-M03-019: dual-control on case resolution: two officers approve; both re-validated via SpiceDB at commit.
FR-M03-020: AdjudicationCase immutable after resolution; new dispute opens a linked successor case with cross-reference.
FR-M03-021: below-threshold quality: "insufficient" outcome returned without recording a match result or opening a case.
FR-M03-022: ambiguous match (above lower bound, below upper bound): AdjudicationCase automatically opened; enrolment blocked until resolved.

Batch operations
FR-M03-023: scheduled full-register dedup batch job (on-demand or quarterly); checkpoint-resumable from last committed record offset.
FR-M03-024: batch kill-resume: restarts from last checkpoint with no duplicate processing; gap audit logged to T1-audit.
FR-M03-025: partial batch failure: per-record report generated; clean records committed; failed records in DLQ with reason code.
FR-M03-026: batch completion emits eel.abis.dedup_batch_completed.v1 with dedup-count, adjudication-count, and insufficient-quality-count.

Federation and certifier interface
FR-M03-027: legacy Commission ABIS federation adapter: fail-closed on adapter failure; in-house matcher used as authoritative fallback.
FR-M03-028: independent third-party certifier interface: read-only access to test-set results, FAR/FRR reports, and enclave attestation evidence.

Reports and monitoring
FR-M03-029: reports endpoint: dedup counts, FAR/FRR trends, throughput, adjudication SLA compliance per configurable time window.
FR-M03-030: per-group fairness metrics tracked per modality across demographic strata; tolerance check results logged.
FR-M03-031: throughput capacity: >= 10 000 enrolment 1:N matches/hr; >= 100 concurrent 1:N queries sustained.
FR-M03-032: availability target 99.99% election-critical; SLO burn-rate alert on 30-day rolling window.
FR-M03-033: RTO <= 4 h RPO <= 15 min; PITR restore drill run and logged to transparency log before each election cycle.
FR-M03-034: abis-svc dashboard: 1:N latency p95/p99, concurrent query count, FAR/FRR trend, adjudication queue depth, enclave health per worker.

Security
FR-M03-035: all API inputs parameterised; sqlx compile-time query checks enforced; no dynamic query construction.
FR-M03-036: SpiceDB down: fail-closed on writes; class-C reads served from cache; 503 problem+json returned.
FR-M03-037: JWT expiry mid-request: in-flight request completes; next request 401 with silent refresh; JWKS refetched on kid miss.
FR-M03-038: realm key rotation with stale JWKS: refetched on kid miss from identity provider; no persistent 401 caching.
FR-M03-039: rate-limit evasion: keyed on subject + device + /64 prefix; priority shedding on overload.
FR-M03-040: secrets and PII never emitted to logs or traces; telemetry pipeline refuses eel_pii_tier_a and eel_pii_tier_b fields.
FR-M03-041: HSM unavailable: signing queued; alert paged; match-result ingest continues in degraded countersign mode.
FR-M03-042: KEK rotation mid-cycle: online re-wrap with new eel_key_id; old templates readable until re-wrap complete; full audit.
FR-M03-043: TLS 1.3 minimum; PQ-hybrid negotiated where peer supports; TLS 1.2 refused.
FR-M03-044: timing side-channel on biometric score comparison: constant-time computation enforced throughout eel-biometric-rs.
FR-M03-045: OWASP ASVS Level 3 controls implemented; no unresolved P1 pen-test findings before production deployment.

Scale and connectivity
FR-M03-046: hot Kafka partition on match-request queue: stream-id spread; lag alert; HPA triggered on consumer lag.
FR-M03-047: consumer lag > 60 s: backpressure applied; Alertmanager page fired; no OOM on match workers.
FR-M03-048: Valkey eviction does not lose idempotency keys: PG-durable idempotency store; Valkey is cache only.
FR-M03-049: Kafka ISR shrink: match-result outbox accumulates in PostgreSQL; no loss; page alert fired.
FR-M03-050: primary/mirror partition: matching redistributed to healthy workers; Patroni fencing; no split-brain.
FR-M03-051: Debezium slot lag alert at 60 s; WAL bloat guard; outbox retained; disk guard prevents overflow.
FR-M03-052: sequencer stall: writes unaffected; cadence alert fired; drain rate >= peak throughput on recovery.

Data and event handling
FR-M03-053: duplicate 1:N match request same eel_idempotency_key and body: cached response returned; no re-execution.
FR-M03-054: same eel_idempotency_key different body: 422 idempotency-key-reuse returned.
FR-M03-055: out-of-order eel.bvr.enrolment_accepted.v1 events: buffered until gap filled or routed to DLQ after timeout.
FR-M03-056: replayed signed event: rejected on eel_sequence counter mismatch; eel_tamper event emitted.
FR-M03-057: poison message (invalid or malformed template): DLQ with reason code; partition processing continues unaffected.
FR-M03-058: schema v1 and v2 coexist in stream: both accepted; dual-emit; Karapace compatibility check passes.
FR-M03-059: oversized event (> 1 MB template): rejected at outbox; payload offloaded to MinIO blob with hash reference.
FR-M03-060: soft-deleted voter templates: tombstone honoured in all match paths; purge removes from TemplateIndex.

Time and identity
FR-M03-061: HLC monotonic: backend clock jump backward detected; HLC never regresses; skew alert fired.
FR-M03-062: SPIFFE SVID rotation during batch: old SVID valid to expiry; eel_producer_key_id per event stamped.
FR-M03-063: dual-control adjudication approver revoked mid-flow: both principals re-validated at commit; transaction aborted if revoked.
FR-M03-064: realm key rotated with stale JWKS: JWKS refetched on kid miss from identity provider.

Compliance and fairness
FR-M03-065: match results retained for statutory period (5 years minimum) per Kenya DPA 2019; retention scheduler enforced.
FR-M03-066: path traversal and zip-slip sanitised in all file-upload and model-artifact paths; fuzz corpus in CI.
FR-M03-067: XXE and deserialisation: DTD disabled; strict serde models; no pickle; billion-laughs test in CI.
FR-M03-068: adversarial biometric input: confidence gate applied; no crash or panic; anomaly logged to T1-audit.
FR-M03-069: model drift detected: drift alert fired; manual review ratio rises; model card version incremented and logged.
FR-M03-070: per-group FAR/FRR tracked across demographic strata; tolerance check fails CI if exceeded for any stratum.
FR-M03-071: PII in ML inference logs: PIIStrippingEvent fired before any external API call; Semgrep regression corpus enforced.
FR-M03-072: SSRF via URL fields in admin endpoints: allowlist enforced; private IP ranges blocked; DNS rebinding guard active.
FR-M03-073: cache poisoning of match-result snapshots: cache key includes eel_sig_version; client verifies before serving.
FR-M03-074: split-view on transparency log: witness divergence detected; consistency check fails closed; alert paged.

## Acceptance criteria

AC-M03-01: 1:N fingerprint against 30M synthetic records p95 <= 5 s, p99 <= 8 s at 100 concurrent queries sustained (load test).
AC-M03-02: FMR <= 0.001% and FNMR <= 0.1% on Commission test set confirmed by independent biometric certifier (formal acceptance test).
AC-M03-03: confidential-compute enclave attestation verified on all workers; failed worker quarantined within 60 s of attestation failure (live attestation report).
AC-M03-04: adjudication workflow closes all UAT cases within 24 h SLA; no overdue cases in end-of-UAT report (UAT).
AC-M03-05: standards compliance package (ISO 19794-2/5/6, ANSI/NIST-ITL, FBI IAFIS IQS) accepted by independent certifier (formal acceptance).
AC-M03-06: 100 concurrent 1:N queries sustained at 30M records with p99 <= 8 s for all three modalities (load test).
AC-M03-07: no unencrypted template observable outside enclave memory; confirmed by attestation report and T1-audit review (attestation + audit).
AC-M03-08: batch dedup resumable from checkpoint after kill-resume; no duplicate processing; gap audit clean in T1-audit (chaos test).
AC-M03-09: 99.99% availability sustained over 30-day SLO window; burn-rate alert fires at 5x error budget (SLO measurement).
AC-M03-10: RTO <= 4 h RPO <= 15 min demonstrated by PITR restore drill; restore event logged to transparency log (DR drill).
AC-M03-11: ASVS L3 scan passes with no unresolved P1 findings; Semgrep eel_pii_tier_a and eel_pii_tier_b clean (security audit).
AC-M03-12: per-group FAR/FRR within tolerance across all demographic strata in fairness harness (fairness test).
AC-M03-13: cosign-verified model artifacts enforced; unsigned model push rejected by CI and refused at enclave load-time (security test).
AC-M03-14: SpiceDB down fails closed on writes; class-C reads served; 503 problem+json returned (chaos test).

## Applicable edge cases

connectivity
EC-CON-04: primary/mirror site partition; matching redistributed to healthy enclave workers; Patroni fencing prevents split-brain; RPO <= 5 min.
EC-CON-11: Kafka ISR shrinks; match-result outbox accumulates in PostgreSQL; no loss; page alert fired.

time
EC-TIM-03: backend clock jumps backward; HLC detects regression and never decrements; skew alert fired within 60 s.
EC-TIM-05: leap second (23:59:60) in match-request timestamp; parser tolerates non-standard second; request accepted.
EC-TIM-06: event time < causation time on reordered stream; record accepted; skew anomaly logged; projection order unaffected.

identity
EC-IDN-02: JWT expires mid-request; in-flight request completes; next request returns 401 with silent refresh.
EC-IDN-03: SPIFFE SVID rotates during batch; old SVID valid to its expiry; eel_producer_key_id stamped per event.
EC-IDN-09: dual-control adjudication approver revoked mid-flow; both principals re-validated at commit; aborted if revoked.
EC-IDN-10: SpiceDB down; fail-closed on writes; class-C reads from cache; 503 problem+json returned.
EC-IDN-11: realm key rotated with stale JWKS cache; JWKS refetched on kid miss; no persistent 401 caching.

data
EC-DAT-01: duplicate 1:N match request same eel_idempotency_key and body; cached response returned; no re-execution.
EC-DAT-02: same eel_idempotency_key different body; 422 idempotency-key-reuse returned; no match started.
EC-DAT-03: out-of-order enrolment events in Kafka stream; buffered until gap filled or routed to DLQ after timeout.
EC-DAT-04: replayed signed event; rejected on eel_sequence counter mismatch; eel_tamper event emitted.
EC-DAT-05: poison message with malformed template blob; DLQ with reason code; partition processing continues.
EC-DAT-06: schema v1 and v2 templates coexist; both accepted; dual-emit; Karapace compat check passes.
EC-DAT-07: partial batch dedup failure; per-record report; clean records committed; failed in DLQ.
EC-DAT-08: 30M row full-register dedup; streaming with bounded memory; checkpoint-resumable after interrupt.
EC-DAT-09: Unicode names in adjudication case search; NFC normalisation; ICU collation; grapheme-safe.
EC-DAT-10: zero dedup results for a batch scope; valid empty report generated; no div-by-zero.
EC-DAT-11: match result count exceeds registered voter count; block with reason code; AdjudicationCase created.
EC-DAT-12: Debezium slot lag or WAL bloat; alert at 60 s; outbox retained; disk guard active.
EC-DAT-13: soft-deleted voter template in match path; tombstone honoured; purge removes from TemplateIndex.
EC-DAT-14: oversized event (> 1 MB template); outbox rejects inline payload; offloaded to MinIO blob + hash.
EC-DAT-15: IPRS-sourced duplicate voter IDs trigger AdjudicationCase; no auto-merge; match blocked until resolved.
EC-DAT-16: legacy Commission ABIS pre-CloudEvents payload; shim upgrades envelope; original signature preserved.

security
EC-SEC-01: signature mismatch on match request; request never executed; eel_tamper event emitted; page within 30 s.
EC-SEC-02: kit chassis tamper event received; quarantine stream activated; custody event generated.
EC-SEC-03: HSM unavailable; signing queued; ingest of match results continues; alert paged; no Merkle gap.
EC-SEC-04: KEK rotation mid-cycle; online re-wrap; old templates readable until done; audit of re-wrap progress.
EC-SEC-05: injection on API inputs; parameterised only; sqlx compile-time; ZAP and Semgrep clean.
EC-SEC-06: SSRF via URL fields in admin; allowlist enforced; private ranges blocked; DNS rebinding guard.
EC-SEC-07: path traversal or zip-slip in model artifact upload; sanitised before write; fuzz corpus in CI.
EC-SEC-08: XXE or deserialisation in template or config parsing; DTD disabled; strict serde models; billion-laughs in CI.
EC-SEC-09: rate-limit evasion on match API; keyed on subject + device + /64; priority shedding on overload.
EC-SEC-10: voter enumeration via match API; constant-time response shape; caps enforced; step-up auth required.
EC-SEC-11: timing side-channel on biometric score compare; constant-time computation enforced in eel-biometric-rs.
EC-SEC-12: cache poisoning of match-result snapshot; cache key includes eel_sig_version; client verifies before serving.
EC-SEC-13: split-view on transparency log; witness divergence detected; consistency check fails closed.
EC-SEC-14: stolen kit in honey mode; back-end records forensic eel.bve.distress_detected.v1 event.
EC-SEC-15: session hijack attempt on adjudication workbench; CSRF + session binding + re-auth on step-up enforced.
EC-SEC-16: secrets or PII in logs or traces; telemetry refuses eel_pii_tier_a and eel_pii_tier_b fields; CI regex enforced.
EC-SEC-17: PQ-hybrid negotiation failure; classical TLS 1.3 only; TLS 1.2 refused on all enclave and API endpoints.

operations
EC-OPS-08: disk full or bucket quota hit; bounded queues applied; 507 returned; alert fired; no data corruption.

scale
EC-SCL-01: 5M concurrent verifiers hitting match status endpoint; >= 95% edge-cached; origin p99 <= 500 ms.
EC-SCL-02: thundering herd on match-result STH endpoint; SDK jitter; ETag 304 served; rate-limit active.
EC-SCL-03: hot Kafka partition on match-request topic; stream-id spread; lag alert; HPA triggered.
EC-SCL-04: consumer lag > 60 s on match queue; backpressure applied; Alertmanager page fired; no OOM.
EC-SCL-05: sequencer stall; writes unaffected; cadence alert; drain rate >= peak throughput.
EC-SCL-06: tiering lag on cold replay; <= 10 MB/s throttle; snapshot-then-tail replay for cold consumer.
EC-SCL-07: 10k enrolments/hr sustained; SLAs met at 1x load; <= 10% p99 regression at 5x load.
EC-SCL-08: Valkey eviction loses idempotency key; PG-durable store authoritative; Valkey is cache only.

ML
EC-ML-01: model drift detected; drift alert fired; manual adjudication review ratio rises; model card version incremented.
EC-ML-02: adversarial biometric input submitted; confidence gate applied; enclave does not crash; anomaly logged.
EC-ML-04: bias across demographic groups detected; per-group FAR/FRR out of tolerance fails CI fairness harness.
EC-ML-06: wrong or unsigned model artifact loaded; cosign verification fails; enclave refuses to start worker.

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

- Vendor-vs-build: clarification C6 (retain existing Commission ABIS vs full custom build) is unresolved; federation adapter is built regardless but the matching compute allocation differs materially.
- TDX vs SEV-SNP: hardware availability in Kenyan DC needs confirmation; software simulation is the dev target but HIL bench timing must be validated before G3.
- Self-hosted Kafka confirmed in ADR stack; any reference to a managed Kafka vendor in the sw/ spec should be corrected to self-hosted before G2.
