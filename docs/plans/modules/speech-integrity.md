# M17 speech-integrity

| field | value |
|---|---|
| M-number | M17 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.5 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M17-speech-integrity.md |

## Purpose

Real-time monitoring of public electoral discourse for incitement, disinformation, and coordinated inauthentic behaviour: ingest content from citizen grievances (M23), public social platforms, and analyst uploads; classify using multilingual models including NLLB for Kenyan languages; surface flags with evidence packages to ECC (M16) and IAB (M15) for human adjudication.

## Surfaces and languages

| surface | technology | path under services/ai-platform-svc/ |
|---|---|---|
| Analyst admin dashboard | Elixir (Phoenix LiveView) | ex/speech-live/ |
| Inference API | Python (FastAPI) | py/speech-inference/ |
| Model registry + training harness | Python | py/speech-training/ |
| Batch ingest worker | Rust | crates/speech-ingest/ |
| Evidence package builder | Rust | crates/speech-evidence/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ContentFlag | flag_id, source, content_hash, content_lang, categories, severity, confidence, model_version, analyst_verdict, status | C | T2 |
| SafetyIncident | incident_id, flag_ids, category, affected_platforms, escalation_level, status, resolved_at | C | T2 |
| EvidencePackage | pkg_id, incident_id, content_refs, transcripts, chain_of_custody, sig, assembled_at | C | T1-audit |
| ModelCard | model_id, version, architecture, training_data_hash, bias_eval_ref, approved_at, sig | D | T2 |
| AuditEvent | event_id, actor, action, ts_hlc, sig | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/ai-platform/speech.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /speech/v1/classify | internal gRPC | SVID mTLS | yes | Ingest content for classification; async |
| GET /speech/v1/flags | 3 accredited | JWT+SpiceDB | n/a | Flag feed for analysts; filterable |
| PATCH /speech/v1/flags/{id} | 3 accredited | JWT+SpiceDB | yes | Analyst adjudication verdict |
| POST /speech/v1/incidents | 3 accredited | JWT+SpiceDB | yes | Escalate flags to incident |
| POST /speech/v1/evidence/{incident_id} | 3 accredited | JWT+SpiceDB | yes | Assemble evidence package |
| GET /speech/v1/evidence/{id} | 3 accredited | JWT+SpiceDB | n/a | Evidence package for ECC / IAB |
| GET /speech/v1/models | 3 accredited | JWT+SpiceDB | n/a | Model registry; cards |
| POST /speech/v1/models | 3 accredited | JWT+SpiceDB | yes | Register new model version |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.speech.content_flagged.v1 | emits | eel.speech.flag.v1 | operational | yes | at-least-once idempotent |
| eel.speech.incident_escalated.v1 | emits | eel.speech.incident.v1 | operational | yes | at-least-once idempotent |
| eel.speech.evidence_assembled.v1 | emits | eel.speech.evidence.v1 | operational | yes (HSM) | exactly-once |
| eel.grievance.case_created.v1 | consumes | eel.grievance.case.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| ECC M16 | internal | integrations/ecc/ | yes | incident push; circuit breaker |
| IAB dashboard M15 | internal | integrations/iab/ | yes | flag feed; stale labelled |
| Grievance channels M23 | internal | integrations/grievance/ | yes | content pull from grievance text |
| Verifier app M13 | internal | integrations/verifier/ | yes | content relay |
| NLLB translation (LiteLLM) | external | integrations/nllb/ | yes (stub) | fail-open; raw text if translate fails |
| Social platforms | external | integrations/social/ | yes (stub) | quota; circuit breaker |
| NCIC/DCI | external | integrations/ncic/ | yes (stub) | evidence package handoff; async |
| AI monitoring M28 | internal | integrations/ai-monitor/ | yes | model version audit |

## ML and AI

Primary models (managed by M28 AI monitoring stack):
- Incitement classifier: fine-tuned on East African context; multilingual (EN, SW, and 9 regional languages via NLLB).
- Disinformation detector: claim-extraction + fact-check index lookup.
- Coordinated inauthentic behaviour: graph-based network analysis of posting patterns.
- Bias evaluation: quarterly eval on held-out Kenyan-language test sets.
- Inference: Python FastAPI (py/speech-inference/), GPU-optional, batched async queue.
- All model cards signed and registered before use (FR-M17-009).
- Training data provenance: hashed and stored in model card.

## Dependencies

- Modules: M13 (grievance content source), M15 (IAB flag consumer), M16 (ECC incident consumer), M28 (AI monitoring)
- Platform: P05 (audit), NLLB via LiteLLM (192.0.2.11:4000)
- Packages: eel-crypto-rs, eel-outbox-rs

## Feature checklist

1. FR-M17-001: Ingest content from Twitter/X firehose via streaming API adapter with per-account deduplication.
2. FR-M17-002: Ingest content from Facebook Graph API with page/group scope controls and quota management.
3. FR-M17-003: Ingest content from YouTube Data API (video metadata, captions, comments) with channel allowlist.
4. FR-M17-004: Accept analyst-uploaded broadcast clips (MP4, MP3, WAV) via authenticated multipart upload endpoint.
5. FR-M17-005: Detect content language using fast local model before routing to NLLB translation.
6. FR-M17-006: Translate non-English, non-Kiswahili content to English via NLLB (LiteLLM proxy at 192.0.2.11:4000) with timeout circuit breaker; fail open to raw text.
7. FR-M17-007: Classify content for hate speech per NCIC Act categories (ethnicity, religion, gender, disability); output category list + per-category confidence score.
8. FR-M17-008: Classify content for electoral incitement (calls to violence, voter intimidation, disenfranchisement); confidence score + severity: low/medium/high/critical.
9. FR-M17-009: Detect disinformation via claim extraction and cross-reference against indexed fact-check corpus; return claim text + matched fact-check URL + contradiction score.
10. FR-M17-010: Detect coordinated inauthentic behaviour (CIB) via graph-based analysis of posting patterns, timing, and account similarity; flag clusters above threshold.
11. FR-M17-011: Detect deepfake/synthetic media signals in video (frame consistency, facial landmark deviation, compression artefacts) and audio (pitch artefacts, splice markers).
12. FR-M17-012: Attach provenance metadata to every content item at ingestion: source platform, ingestion timestamp, ingest worker version, original URL hash.
13. FR-M17-013: Compute and store content hash (SHA-256) at ingestion; use as idempotency key to prevent double-processing of re-shared content.
14. FR-M17-014: Publish eel.speech.content_flagged.v1 event to Kafka topic eel.speech.flag.v1 for every content item that crosses any severity threshold.
15. FR-M17-015: Enforce RBAC via SpiceDB: content analyst sees only their assigned queue; senior analyst sees all flags; legal sees approved referrals; comms sees published summaries.
16. FR-M17-016: Analyst dashboard: filterable flag feed by severity, category, language, platform, date range; sortable by confidence and SLA deadline.
17. FR-M17-017: Analyst adjudication actions: accept, dismiss (with mandatory reason code), escalate to senior, refer to legal; action recorded with actor JWT subject and HLC timestamp.
18. FR-M17-018: Senior analyst review actions: confirm, override with rationale, return to queue; override triggers model feedback loop event.
19. FR-M17-019: Legal review workflow: approve or reject referral; approve triggers evidence package assembly; reject returns to senior analyst with notes.
20. FR-M17-020: Incident creation: group related flags by topic cluster or platform campaign; assign incident_id; set escalation level (platform, NCA, DCI, court); track SLA.
21. FR-M17-021: Evidence package assembly: collect content items, transcripts, classification outputs, analyst verdicts, chain-of-custody log; sign with HSM key; store in T1-audit.
22. FR-M17-022: Evidence package export to NCIC/DCI: async handoff via secure channel; generate signed chain-of-custody form; receive and store handoff receipt.
23. FR-M17-023: Every ML inference logs model_id, model_version, input_hash, output categories, confidence scores, inference latency, GPU node ID.
24. FR-M17-024: Human-in-the-loop: no content item advances to referral without at least one analyst verdict and one senior analyst confirmation.
25. FR-M17-025: Analyst override of ML verdict logged as training feedback event (eel.speech.feedback.v1) for future retraining cycle.
26. FR-M17-026: Bias monitoring: compute per-language, per-community false-positive rate (FPR) on rolling 7-day window; store in time-series store.
27. FR-M17-027: Bias alert: fire PagerDuty alert and Grafana annotation when any language FPR exceeds 5% threshold.
28. FR-M17-028: Model governance: model card (ModelCard entity) required and signed before any model version is promoted to production; registered in M28.
29. FR-M17-029: Model version locked at inference time; inference log references exact model_id; rollback restores prior locked version with no re-processing required.
30. FR-M17-030: Adversarial robustness: red-team evasion test suite (obfuscated slurs, Unicode homoglyphs, code-switching) runs in CI; block promotion if pass rate < 90%.
31. FR-M17-031: Data minimisation: raw content text purged within 72 hours of ingestion unless content_hash appears in a live EvidencePackage; purge job auditable.
32. FR-M17-032: PII stripping: named entities (phone numbers, national IDs, email addresses) redacted from stored transcripts before persistence; original hash retained for dedup.
33. FR-M17-033: Content stored as hash reference after processing; full text retained only in signed evidence packages under T1-audit controls.
34. FR-M17-034: Transparency report: weekly and monthly batch job aggregates detection counts by category and severity; no PII; published to public endpoint.
35. FR-M17-035: Transparency report excludes any content item under active legal review or in an open evidence package.
36. FR-M17-036: Analyst dashboard multilingual UI: English and Kiswahili; locale switching per user preference; stored in user profile.
37. FR-M17-037: WCAG 2.2 AA compliance on analyst dashboard: colour contrast >= 4.5:1; keyboard navigable; screen-reader labels on all flag cards and action buttons.
38. FR-M17-038: SLA tracking: critical flags reviewed within 1 hour; high within 4 hours; medium within 24 hours; dashboard shows SLA countdown and breach alert.
39. FR-M17-039: Classification p95 latency <= 30 seconds per content item under normal load; reported in Grafana SLO dashboard.
40. FR-M17-040: Evidence package assembly completes within 5 minutes; timeout triggers alert and partial package is preserved for manual completion.
41. FR-M17-041: GPU queue backlog alert fires when queue depth exceeds 500 items; auto-scales inference workers up to configured maximum.
42. FR-M17-042: Social platform API quota tracking: log remaining quota per platform per hour; pause ingest and alert when quota < 10%.
43. FR-M17-043: Social platform API revocation: detect 401/403 from platform adapter; switch to manual analyst upload path; notify ops within 5 minutes.
44. FR-M17-044: Canary content isolation: known test content (marked with canary header) routed to shadow queue; never surfaced to production analyst dashboard or transparency report.
45. FR-M17-045: Broadcast clip transcription: audio-to-text via Whisper-compatible endpoint; transcript attached to content item before classification.
46. FR-M17-046: Propagation graph: track reshare/retweet chains for flagged content; store graph edges; visualise in analyst dashboard as propagation timeline.
47. FR-M17-047: Content deduplication across platforms: same text or media hash from multiple platforms treated as one content item; platform sources listed.
48. FR-M17-048: Analyst queue assignment: round-robin assignment with load balancing; analyst can self-assign from unassigned pool; senior can reassign.
49. FR-M17-049: NCIC referral dual sign-off: referral requires both a senior analyst approval and legal approval before evidence package is transmitted.
50. FR-M17-050: Referral status tracking: pending, transmitted, acknowledged, closed; NCA/DCI acknowledgement receipt stored against referral record.
51. FR-M17-051: Audit trail: every detection, analyst action, model override, escalation, and referral signed with actor identity and HLC timestamp; stored in T1-audit via P05.
52. FR-M17-052: Audit log immutability: T1-audit events append-only; deletion requires signed out-of-band approval from two platform admins; deletion event itself logged.
53. FR-M17-053: Fact-check corpus: indexed from accredited Kenyan and regional fact-checkers; updated nightly; versioned by update date.
54. FR-M17-054: Claim extraction model: identifies checkable factual claims in content; outputs claim text, subject entity, claim type; linked to fact-check lookup.
55. FR-M17-055: Network analysis for CIB: account graph built from interaction data; community detection via Louvain; suspicious cluster threshold tunable by senior analyst.
56. FR-M17-056: Model retraining trigger: automated trigger when bias alert fires or when analyst override rate on a category exceeds 15% in a 24-hour window.
57. FR-M17-057: Training data provenance: every training dataset hashed; hash stored in model card; reproducible training pipeline documented in model card.
58. FR-M17-058: Reproducible inference: given model_version and input_hash, inference can be replayed and output verified; required for evidence admissibility.
59. FR-M17-059: Jailbreak detection: prompt-injection patterns in user-supplied text content detected and flagged before classification; logged separately.
60. FR-M17-060: Rate limiting on ML inference API: per-caller token bucket; burst allowed up to 2x sustained rate; 429 with Retry-After on breach.
61. FR-M17-061: Internal gRPC inference endpoint authenticated via SPIFFE/SVID mTLS; no unauthenticated callers permitted.
62. FR-M17-062: All REST API endpoints require JWT; SpiceDB authz check on every request; 403 on policy miss; audit logged.
63. FR-M17-063: Evidence package integrity: SHA-256 hash of package contents stored in AuditEvent; verifiable without access to T1-audit storage directly.
64. FR-M17-064: HSM signing for evidence packages: key usage restricted to evidence sign operations; key ID recorded in package; rotation supported without reissuing old packages.
65. FR-M17-065: Soft-delete for content flags: analyst dismiss marks status dismissed; content hash and classification output retained for bias monitoring; raw text still purged per FR-M17-031.
66. FR-M17-066: Batch ingest worker (Rust crate speech-ingest) checkpoints stream offset to durable store; resumes from last committed offset on restart; no re-ingest of already-processed items.
67. FR-M17-067: Dead-letter queue for ingest failures: items that fail classification three times routed to DLQ; ops alerted; manual review path available.
68. FR-M17-068: Grafana dashboard: flag rate by severity, analyst SLA adherence, per-language FPR, NLLB translation latency, GPU queue depth, model version active.
69. FR-M17-069: SLO burn-rate alerts: 99.5% availability burn-rate alert; classification p95 > 30 s alert; FPR > 5% alert; all wired to PagerDuty.
70. FR-M17-070: OpenAPI contract (packages/openapi-contracts/ai-platform/speech.yaml) versioned and linted in CI; breaking changes require semver minor bump and migration note.
71. FR-M17-071: gRPC proto for internal classify endpoint versioned in packages/proto/speech/v1/; backward-compatible field additions only; breaking changes require new version package.
72. FR-M17-072: Outbox pattern (eel-outbox-rs) for all emitted events; at-least-once delivery; idempotency key on consumer side prevents double processing.
73. FR-M17-073: Health and readiness endpoints: /health returns liveness; /ready checks DB connectivity, Kafka reachability, NLLB proxy reachability; used by k8s probes.
74. FR-M17-074: Structured logging (JSON, log level configurable via env); every log line includes trace_id, span_id, service, module; shipped to Loki.
75. FR-M17-075: Distributed tracing: OpenTelemetry spans for ingest, translate, classify, adjudicate, assemble; trace exported to Tempo; linked in Grafana Explore.

## Acceptance criteria

AC-M17-01: A Kiswahili incitement sample from the held-out test set is correctly classified at >= 85% precision; measured by ML eval harness against labelled dataset.
AC-M17-02: NLLB translation timeout (injected via chaos stub) causes the pipeline to fall back to raw text classification within 500 ms; no flag is dropped; verified by integration test.
AC-M17-03: An assembled evidence package can be independently verified by recomputing SHA-256 over its content fields and matching the stored hash in the AuditEvent; verified by integration test.
AC-M17-04: When per-language FPR for any language exceeds the 5% threshold in the bias monitor job, a PagerDuty alert fires within 2 minutes; verified by regression test with synthetic FPR spike.
AC-M17-05: Every NCIC/DCI referral results in a signed handoff receipt stored against the referral record within 10 minutes of transmission; verified by integration test against NCIC stub.
AC-M17-06: Raw content text is absent from the primary datastore 73 hours after ingestion for content items not referenced by any EvidencePackage; verified by data retention audit test.
AC-M17-07: A content analyst cannot access flags outside their assigned queue; SpiceDB policy rejects the request with 403; verified by RBAC integration test.
AC-M17-08: Analyst override of an ML verdict appears as a training feedback event on the Kafka topic eel.speech.feedback.v1 within 5 seconds; verified by event consumer integration test.
AC-M17-09: Classification p95 latency stays <= 30 seconds under a simulated election-day load of 500 concurrent items; verified by k6 load test.
AC-M17-10: A referral cannot be transmitted to NCIC without both a senior analyst approval and a legal approval recorded in the audit trail; API returns 422 if either is missing; verified by integration test.
AC-M17-11: Canary content marked with the canary header does not appear on the production analyst dashboard or in any transparency report; verified by end-to-end test.
AC-M17-12: Adversarial evasion test suite (obfuscated slurs, Unicode homoglyphs) achieves >= 90% detection rate against the fine-tuned incitement classifier; CI blocks promotion if below threshold.
AC-M17-13: Every model version deployed to production has a signed ModelCard registered in M28 before the first inference call; deployment pipeline enforces this as a gate; verified by CI integration test.
AC-M17-14: The analyst dashboard renders correctly in both English and Kiswahili locales and passes automated WCAG 2.2 AA colour-contrast checks; verified by Playwright accessibility test.

## Applicable edge cases

EC-DAT-01: Empty or whitespace-only content body arrives from social platform API; ingest worker rejects with 400 before classification; no flag emitted.
EC-DAT-02: Content field contains only an embedded image URL with no text; OCR pipeline activated before NLP classification; if OCR yields no text, item logged and discarded.
EC-SEC-05: JWT presented with expired signature; API returns 401; no authz check performed; event logged with actor unknown.
EC-SEC-16: Raw content contains national ID numbers or phone numbers; PII-strip regex redacts before persistence; original content hash retained for dedup; strip operation audited.
EC-IDN-02: Two social platform accounts share identical display names but different platform IDs; CIB graph keys on platform-scoped account ID, not display name; no false cluster merge.
EC-IDN-10: Content item reposted across three platforms with minor text mutations; content normalisation and fuzzy hash (simhash) detects near-duplicate; single canonical flag created.
EC-SCL-04: Database write throughput saturates during election-day spike; ingest worker applies backpressure by pausing Kafka consumer; no data loss; lag monitored and alerted.
EC-ML-01: Model drift detected post-deployment (rising FPR on Kikuyu-language content); bias alert fires; retraining triggered automatically; drifted model rolled back within 30 minutes if FPR exceeds 10%.
EC-ML-02: Adversarial input with Unicode homoglyphs (Latin 'a' replaced by Cyrillic) evades naive classifier; adversarial robustness eval in CI catches regression; promotion blocked.
EC-ML-03: Canary test content tagged with internal header reaches production ingest; isolation logic routes to shadow queue; shadow queue events never reach analyst dashboard or transparency report.
EC-ML-04: NLLB model returns a translation that is longer than the classifier's max token limit; content is chunked; each chunk classified independently; results merged by max severity.
EC-ML-05: Fact-check corpus is stale (nightly update failed); claim lookup returns no match; flag annotated with corpus_version and corpus_age; analyst alerted to stale corpus.
EC-ML-06: CIB cluster threshold is mis-tuned, causing a legitimate civic campaign to be flagged; senior analyst overrides; override event triggers threshold sensitivity review task in ops backlog.
EC-ML-07: GPU node fails mid-inference; job requeued automatically by inference worker; idempotency key on content_hash prevents duplicate flags; failed node removed from pool.
EC-SEC-01: SSRF attempt via social platform webhook callback URL containing internal IP range; allowlist validation rejects URL before HTTP call; incident logged.
EC-SEC-02: Content upload endpoint receives a ZIP bomb (deeply nested archive); file type validation and size cap reject before decompression; 413 returned.
EC-SEC-03: Analyst session token stolen; attacker attempts flag modification; token bound to IP range (configurable); SpiceDB logs anomalous actor; session revoked; analyst notified.
EC-SEC-06: SQL injection attempt in flag filter query parameter; parameterised queries prevent execution; WAF blocks; request logged with actor.
EC-SEC-09: Evidence package signing key requested by unauthorised service; HSM key usage policy blocks non-evidence-builder callers; attempted access logged.
EC-SEC-14: Analyst account brute-forced; Keycloak lockout after 5 failed attempts; lockout event forwarded to SIEM; account requires senior admin unlock.
EC-SEC-17: Internal gRPC classify endpoint called without valid SVID certificate; mTLS handshake fails; connection rejected; certificate error logged.
EC-SCL-01: Social platform firehose delivers 10x normal volume for 15 minutes during a breaking news event; Kafka consumer group auto-scales; ingest lag stays under 60 seconds.
EC-SCL-04: (second manifestation) Analyst dashboard query for flag feed times out under high concurrent analyst load; read replica promoted for dashboard queries; primary reserved for writes.
EC-SCL-05: Election-day GPU queue backlog exceeds 500 items; auto-scaler provisions additional inference worker pods; p95 latency relaxes to 60 seconds under documented SLA exception.
EC-SCL-07: Transparency report batch job competes with live ingest for DB read capacity; report job runs against read replica; no production latency impact.
EC-IDN-09: Analyst attempts to access an evidence package for an incident assigned to a different legal team; SpiceDB incident-scoped policy returns 403; access attempt logged for audit.
EC-SEC-05: (second manifestation in module context) Expired model card signing certificate; model card verification fails during deployment gate; deployment blocked; ops alerted to renew certificate.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | classifier pipeline, confidence threshold, analyst FSM, evidence builder, purge scheduler | pytest + nextest | M17-UNIT- |
| property | every flag reaches terminal state; evidence hash stable | proptest | M17-PROP- |
| contract | M23 grievance text (consumer); M16 incident push (provider); M15 flag feed (provider) | Pact | M17-PACT- |
| integration | NLLB stub; social platform stub; NCIC stub; SoftHSM evidence sign | pytest + testcontainers | M17-INT- |
| e2e | grievance -> classify -> flag -> analyst adjudicate -> incident -> evidence -> NCIC | ExUnit + Playwright | M17-E2E- |
| ML | precision/recall per language on held-out test set; adversarial robustness | pytest + MLflow | M17-ML- |
| fuzz | content payload; NLLB translation response; evidence package fields | cargo-fuzz | M17-FUZZ- |
| load | election-day spike; GPU queue under load; analyst dashboard latency | k6 | M17-LOAD- |
| security | PII-strip regression; SSRF on social webhook; OWASP ZAP | ZAP + Semgrep | M17-SEC- |

## Runbooks and dashboards

- docs/runbooks/speech-integrity-degradation.md (create): NLLB unavailable; GPU queue overload; social API revoked; NCIC handoff failure.
- docs/runbooks/speech-model-governance.md (create): model card approval; bias eval; rollback procedure.
- platform/observability/ai-platform-svc/grafana-speech.json: flag rate by severity, analyst SLA adherence, per-language FPR, NLLB translation latency.
- platform/observability/ai-platform-svc/slo-alerts.yaml: 99.5% burn rate; classification p95 > 30 s; FPR > 5%.

## Open questions

- Social platform API access: platform approval timelines are unpredictable; if access is not granted before T-6 months, the social ingest surface is replaced by manual analyst upload.
- Kenyan-language bias: the training corpus for 9 regional languages is thin; a bias audit against a held-out Kenyan dataset is required before M reaches production approval; results inform whether any language is gated out.
- NCIC MoU: the data-sharing agreement with NCIC/DCI for evidence handoffs has not been signed; legal must confirm scope and liability before evidence package export is enabled.
