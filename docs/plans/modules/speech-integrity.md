# M17 speech-integrity

| field | value |
|---|---|
| M-number | M17 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.5 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M17-speech-integrity.md |

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
- Platform: P05 (audit), NLLB via LiteLLM (62.169.25.77:4000)
- Packages: eel-crypto-rs, eel-outbox-rs

## Feature checklist

1. FR-M17-001: multilingual classification: incitement, hate speech, disinformation, coordinated behaviour.
2. FR-M17-002: language detection + NLLB translation for 9 Kenyan regional languages.
3. FR-M17-003: confidence score per flag; severity: low/medium/high/critical.
4. FR-M17-004: analyst adjudication workflow: accept, dismiss, escalate with notes.
5. FR-M17-005: incident escalation: group related flags; assign to analyst team; SLA tracking.
6. FR-M17-006: evidence package assembly: content + metadata + chain of custody, HSM-signed.
7. FR-M17-007: evidence package handoff to NCIC/DCI for prosecution referrals.
8. FR-M17-008: bias monitoring: per-language false-positive rate tracked; alert on drift.
9. FR-M17-009: model governance: model card required before deployment; signed; registered in M28.
10. FR-M17-010: GDPR/DPA data minimisation: content stored as hash after processing; raw text purged within 72 h unless in evidence package.
11. FR-M17-011: social platform ingestion: crawl or webhook from accredited platforms (API access gated by platform approval).
12. FR-M17-012: canary messaging detection: known test content not surfaced to public dashboard.
13. FR-M17-013: NCIC referral workflow: evidence package export, chain-of-custody form, digital handoff receipt.
14. Classification p95 <= 30 s per content item.
15. Evidence package assembly <= 5 min.
16. Analyst SLA: critical flags reviewed within 1 h; high within 4 h; medium within 24 h.
17. False-positive rate per language tracked; alert if > 5%.
18. All model inference logged with model_version and confidence.

## Acceptance criteria

AC-M17-01: Kiswahili incitement sample correctly classified at >= 85% precision (ML eval).
AC-M17-02: NLLB translation fail-open: raw text processed when translate times out (chaos test).
AC-M17-03: evidence package chain of custody verifiable independently (integration test).
AC-M17-04: bias alert fires when per-language FPR exceeds threshold (regression test).
AC-M17-05: NCIC handoff receipt issued for every referral (integration test).
AC-M17-06: content purged within 72 h unless in evidence package (data retention audit).
AC-M17-07: 99.5% availability (SLO measurement).

## Applicable edge cases

**data**
EC-DAT-09: Unicode content; grapheme-safe; NLLB handles multi-script.
EC-DAT-16: legacy content format (PDF screenshot); OCR pipeline before classify.

**security**
EC-SEC-06: SSRF via social platform webhook URL; allowlist.
EC-SEC-16: secrets in content; PII-strip before storage.

**ML**
EC-ML-01: model drift post-deployment; bias eval alert; retraining trigger.
EC-ML-02: adversarial input (obfuscated slurs); robustness eval before deploy.
EC-ML-03: canary messaging in test environment; isolation; not surfaced to prod.

**legal**
EC-LEG-05: NCIC referral requires dual analyst sign-off; chain of custody auditable.
EC-LEG-06: DPA content retention; raw text purge within 72 h.

**operations**
EC-OPS-06: social platform API revoked; fallback to manual upload path.

**scale**
EC-SCL-05: election day content spike; GPU queue backlog; p95 SLA relaxed to 60 s.

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
