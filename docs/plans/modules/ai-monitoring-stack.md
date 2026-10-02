# M28 ai-monitoring-stack

| field | value |
|---|---|
| M-number | M28 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.4 (docs/plans/05-wave-4-operations.md) |
| Size | XL |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M28-ai-monitoring-stack.md |

## Purpose

Deploy a governed ensemble of AI models around but never inside the result-determining pipeline, covering on-device quality control (models A1-A8), central form processing and anomaly detection (B1-B7), citizen-facing multilingual services (C1-C4), disinformation monitoring (D1-D4), and internal operations (E1-E2); a model registry with published model cards, CI-integrated eval harnesses, Kenyan-jurisdiction hosting, and IAB 72-hour advance notification for every production change enforces five hard red lines: no AI in any vote-tally pipeline, no autonomous voter rejection, no training on live voter biometric data, no LLM in the result-determining pipeline, and no biometric data transiting a non-Kenyan cloud endpoint.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Public model registry | Astro | services/ai-platform-svc/astro/model-registry-public-site/ |
| Form 34A OCR Review Console (the Commission Officer) | Phoenix LiveView | services/ai-platform-svc/ex/ocr_web/ |
| AI Decision Log Viewer (IAB) | Django | services/ai-platform-svc/django/iab_ai_log/ |
| Disinformation Flag Dashboard (SOC Analyst) | Phoenix LiveView | services/ai-platform-svc/ex/soc_web/ |
| Translation Approval Queue (the Commission Comms) | Django | services/ai-platform-svc/django/comms_approval/ |
| Chatbot (WhatsApp / Web / SMS) | Python (FastAPI + LiteLLM) | services/ai-platform-svc/py/chatbot_worker/ |
| API | Rust (axum) | services/ai-platform-svc/crates/ai-platform-api/ |

Languages: EN + SW for all public model cards; 15 Kenyan languages for TTS, ASR, and chatbot (EN, SW, Gikuyu, Dholuo, Luhya, Kamba, Kalenjin, Meru, Borana, Taita, Mijikenda, Nandi, Somali, Turkana, Oromo).

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| AIModel | id, code (A1-E2), name, version, layer (KIT/CENTRAL/CITIZEN/DISINFORMATION/INTERNAL), architecture, training_data_description, eval_accuracy, eval_bias_flag, eval_adversarial_robustness, known_limitations, human_in_loop, sovereign_hosting, model_card_url, deployed_at, iab_notified_at, ccb_approval_id, rollback_procedure, is_active | B | T1-consensus |
| AIDecisionLog | id, model_id, model_version, interaction_type, input_hash (SHA-256 of PII-stripped input), confidence_score, model_output (structured, no PII), human_decision, human_actor_id, human_decision_at, decision_at | A | T1-audit |
| ModelCardPublication | id, model_id, version, published_at, card_hash (SHA-256 of card markdown), merkle_node_id, languages | B | T1-consensus |
| PIIStrippingEvent | id, model_id, original_input_length, stripped_input_hash (SHA-256 of stripped text; original not stored), pii_tokens_replaced, stripped_at | C | T2 |

## APIs

OpenAPI file: `packages/openapi-contracts/ai/ai-monitoring.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/ai/models | 1 | public | n/a | list all registered AI models |
| GET /v1/ai/models/{id} | 1 | public | n/a | single model registry entry |
| GET /v1/ai/models/{id}/card | 1 | public | n/a | download model card (markdown) |
| POST /v1/ai/models | 2 | JWT+SpiceDB (DATACRAFT_AI_LEAD) + CCB_APPROVAL | yes | register new model or version; 5/day limit |
| PATCH /v1/ai/models/{id}/activate | 2 | JWT+SpiceDB (DATACRAFT_AI_LEAD) + COMMISSION_CHAIR HSM | yes | activate model version in production |
| GET /v1/ai/decision-log | 2 | JWT+SpiceDB (IAB_OBSERVER) | n/a | query AI decision log (no PII) |
| POST /v1/ai/chatbot/message | 2 | JWT or public | yes | send message to election chatbot; PII stripped before any LLM call |
| POST /v1/ai/translation/request | 2 | JWT+SpiceDB (COMMISSION_COMMS) | yes | request NMT translation |
| POST /v1/ai/translation/{id}/approve | 2 | JWT+SpiceDB (COMMISSION_COMMS) | yes | human-approve translation before publication |
| GET /v1/ai/disinformation/flags | 2 | JWT+SpiceDB (SOC_ANALYST) | n/a | list disinformation detection flags |
| PATCH /v1/ai/disinformation/flags/{id} | 2 | JWT+SpiceDB (SOC_ANALYST) | yes | record human analyst verdict |
| POST /v1/ai/form34a/submit-image | internal gRPC | SVID mTLS (RTS_BACKEND_CERT) | yes | submit Form 34A image for OCR processing |
| GET /v1/ai/form34a/{id}/ocr-result | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | n/a | fetch OCR result for human review |
| POST /v1/ai/form34a/{id}/approve | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | yes | human-approve Form 34A after OCR review |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.ai.model_activated.v1 | emits | eel.ai.model-registry.v1 | operational | yes | exactly-once |
| eel.ai.model_card_published.v1 | emits | eel.ai.model-registry.v1 | operational | yes | exactly-once |
| eel.ai.iab_notice_sent.v1 | emits | eel.ai.model-registry.v1 | operational | yes | exactly-once |
| eel.ai.form34a_ocr_complete.v1 | emits | eel.ai.form34a.v1 | operational | yes | at-least-once idempotent |
| eel.ai.translation_approved.v1 | emits | eel.ai.translation.v1 | operational | yes | exactly-once |
| eel.ai.disinformation_flag.v1 | emits | eel.ai.disinformation.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M01 BVR Device / M05 BVI Device | internal | integrations/kit-firmware-ota/ | yes | on-device model packaging; OTA delivery via P-OTA |
| M03 ABIS | internal | integrations/abis/ | yes | B4 duplicate-voter suggestion feeds ABIS adjudicator queue |
| M10 RTS Back-End | internal | integrations/rts/ | yes | B1/B2 Form 34A pipeline; B3/B5/B6 telemetry |
| M23 Grievance Channels | internal | integrations/grievance/ | yes | C1 chatbot intake; C4 ASR for voice grievances |
| M26 Anomaly Detection | internal | integrations/detection/ | yes | B3/B5/B6/B7 central model outputs as detection layer inputs |
| M27 Intent Preservation | internal | integrations/intent/ | yes | A7 pre-mark validation model |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | model card and decision log anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes | model activation requires the Commission Chair HSM |
| M11 Public Portal | internal | integrations/portal/ | yes | model registry, model cards, public AI decision log summary |
| LiteLLM (Kenyan cloud tenant) | internal | integrations/litellm/ | yes | C1 chatbot LLM routing; PII stripped before any call |
| NLLB / NMT service | internal (Kenyan cloud) | integrations/nmt/ | yes | C2 translation |
| Social media monitoring | external (aggregated) | integrations/social-monitor/ | yes (stub) | D1/D4 inputs; no individual user data |

## ML and AI

Kit-layer on-device models (A-series, INT8 quantised, offline-capable):
- A1 face liveness detection (depth-map + texture CNN, <= 15 MB, <= 500 ms): photo/video impersonation; presiding officer override with dual-agent witness
- A2 fingerprint quality scoring (ISO 29794-4, <= 5 MB): quality before template extraction; prompts retake; no autonomous rejection
- A3 iris quality scoring (ISO 29794-6, <= 5 MB): same design as A2; no autonomous rejection
- A4 ID document OCR cross-check (CRNN/TrOCR, Kenyan ID types, <= 8 MB, <= 2 s): biographic field mismatch flagged; presiding officer confirms; no autonomous rejection
- A5 voice-guided prompts TTS (15 languages, <= 20 MB): scripts human-approved per language before deployment
- A6 on-device anomaly detection (Isolation Forest variant, <= 3 MB): feeds M26 Layer 1; presiding officer not blocked
- A7 pre-mark ballot validation (vision model, <= 10 MB): on-device only; image deleted within 1 s; per M27 spec
- A8 face age estimation (BVR only, <= 5 MB): advisory only; presiding officer confirms against ID document

Central-layer models (B-series, Kenyan-cloud-hosted):
- B1 Form 34A image processing (orientation, de-skew, OCR, quality): flagged forms held for human review; never auto-published
- B2 handwriting recognition on tally lines (Kenyan-form-tuned HTR): suggestion only; human review required before any count recorded
- B3 telemetry anomaly detection (LSTM + Isolation Forest): feeds M26 Layer 2; all alerts advisory
- B4 duplicate-voter suggestion (1:N matching confidence bands): adjudicator referral below auto-clear threshold; not autonomous
- B5 turnout implausibility flag (statistical model): > 3 sigma from constituency mean; feeds M26 Layer 2
- B6 transmission-gap pattern alert (threshold + trend): feeds M26 Layer 1
- B7 ML-assisted SIEM (Isolation Forest / unsupervised network anomaly): SOC analyst reviews all alerts; no autonomous network action

Citizen-layer (C-series):
- C1 multilingual chatbot (LLM with the Commission knowledge base): WhatsApp, Telegram, SMS; PII-stripped before any API call; escalates to human on complex or ambiguous queries; hosted on Kenyan-tenant LiteLLM
- C2 real-time announcement translation (NLLB-family NMT): the Commission Comms Officer approves every translation before publication; back-translation QA on all sensitive content
- C3 TTS for radio and IVR: human reviews all scripts before production
- C4 ASR for grievance intake (Kenyan-language fine-tuned): transcribes voice-note grievances; human agent reviews transcript; intake aid only

Disinformation (D-series):
- D1 CIB detection (graph + velocity model): human analyst reviews HIGH-confidence flags; daily summary to IAB and the Commission Comms
- D2 deepfake detection (FaceForensics-family CNN): human analyst confirms before any action; limitation published in model card: state-level deepfakes may evade current detectors
- D3 fake Form 34A detection (hash lookup + visual similarity): hash matches are deterministic; visual-similarity matches are advisory; public query via Public Verifier
- D4 bot-network detection (network analysis + bot-score model): human analyst reviews; human distinguishes coordinated-inauthentic from legitimate coordinated campaigns

Internal (E-series):
- E1 AI-assisted code review (static analysis + LLM): CI pipeline; developer + security reviewer approve all changes; LLM output is a prompt, not a gate
- E2 adaptive poll-worker training assessments (IRT model): trainer reviews flagged poor performers; remediation is human-designed

## Dependencies

- Modules: M01, M03, M05, M08, M10, M11, M23, M26, M27
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-outbox-py, eel-resilience-rs, eel-resilience-py, eel-crypto-rs, kit-sdk-flutter

## Feature checklist

1. FR-M28-001: A1 face liveness detection; depth-map + texture CNN; INT8; <= 15 MB; <= 500 ms; Presiding Officer override with dual-agent witness + reason code; no liveness decision autonomous
2. FR-M28-002: A2 fingerprint quality scoring; ISO 29794-4; edge inference; <= 5 MB; prompts retake below threshold; no autonomous rejection; no quality score used to reject a voter
3. FR-M28-003: A3 iris quality scoring; ISO 29794-6; <= 5 MB; same design as A2; no autonomous rejection
4. FR-M28-004: A4 ID document OCR cross-check; CRNN/TrOCR; Kenyan ID types (2nd-gen ID, Maisha card, Passport); INT8; <= 8 MB; <= 2 s; Presiding Officer confirms each flagged field; no OCR output creates automatic rejection
5. FR-M28-005: A5 TTS voice prompts; 15 Kenyan languages; INT8; <= 20 MB; scripts human-approved per language before deployment
6. FR-M28-006: A6 on-device anomaly detection; Isolation Forest variant; INT8; <= 3 MB; alert sent to ECC; Presiding Officer not blocked from any action; feeds M26 Layer 1
7. FR-M28-007: A7 pre-mark ballot validation; vision model per M27 spec; on-device only; image deleted within 1 s of inference
8. FR-M28-008: A8 face age estimation; BVR only; advisory only; Presiding Officer confirms against ID document; no voter rejected on age-model output alone
9. FR-M28-009: B1 Form 34A image processing; orientation, de-skew, OCR, quality scoring; below-quality forms held for human review; never auto-published or auto-rejected without human sign-off
10. FR-M28-010: B2 handwriting recognition on tally lines; Kenyan-election-form-tuned HTR; suggestion only; human review required before any extracted count recorded
11. FR-M28-011: B3 telemetry anomaly detection; LSTM + Isolation Forest on time-series telemetry per M26 Layer 2; all alerts advisory
12. FR-M28-012: B4 duplicate-voter suggestion; ABIS 1:N confidence score bands; adjudicator referral below auto-clear threshold; adjudicator makes all dedup decisions; not autonomous
13. FR-M28-013: B5 turnout implausibility flag; > 3 sigma from constituency mean; feeds M26 Layer 2; flag includes constituency_mean, observed_turnout, z_score
14. FR-M28-014: B6 transmission-gap pattern alert; threshold + trend model; feeds M26 Layer 1
15. FR-M28-015: B7 ML-assisted SIEM; Isolation Forest / unsupervised network anomaly; SOC analyst reviews all alerts; no autonomous network action or firewall rule
16. FR-M28-016: C1 multilingual chatbot; WhatsApp, Telegram, SMS; PII-stripped before LLM call; escalates to human on complex, sensitive, or ambiguous queries; Kenyan-tenant LiteLLM
17. FR-M28-017: C2 NMT translation; NLLB-family; Comms Officer approves every translation before publication; back-translation QA on all sensitive content
18. FR-M28-018: C3 TTS for radio and IVR; Kenyan-accented voices; human reviews all scripts before production
19. FR-M28-019: C4 ASR for grievance voice notes; Kenyan-language fine-tuned; human agent reviews transcript; intake aid only
20. FR-M28-020: D1 CIB detection; graph + velocity model; human analyst reviews HIGH-confidence flags; daily summary to IAB and Comms Officer
21. FR-M28-021: D2 deepfake detection; FaceForensics-family CNN; human analyst confirms before any action; limitation published in model card: state-level deepfakes may evade
22. FR-M28-022: D3 fake Form 34A detection; hash lookup + visual similarity; public query via Public Verifier; hash matches deterministic, visual-similarity advisory and labelled as such
23. FR-M28-023: D4 bot-network detection; network analysis + bot-score; human analyst reviews; human distinguishes coordinated-inauthentic from legitimate coordinated campaigns
24. FR-M28-024: E1 AI-assisted code review; CI pipeline; developer + security reviewer approve all changes; LLM output is a prompt, not a gate
25. FR-M28-025: E2 adaptive poll-worker training assessments; IRT model; trainer reviews flagged poor performers; remediation human-designed
26. FR-M28-026: model registry; every production model has: model ID, version, training data description (no PII, no biometric data), eval harness results, known limitations, deployment date, IAB notification date, rollback procedure, model card URL
27. FR-M28-027: model card published on public portal; format: purpose, architecture, training data source, eval metrics, known failure modes, human-in-loop controls, version history; EN required; SW summary required
28. FR-M28-028: eval harness; CI-integrated per model; minimum dimensions: accuracy, bias across demographic groups, adversarial robustness on held-out test set; results published with model card before production deployment
29. FR-M28-029: IAB notification; every production model change triggers 72-hour advance notice with model card diff and eval delta; IAB may request 7-day delay; eel.ai.iab_notice_sent.v1 emitted before activation
30. FR-M28-030: AI decision log; every central AI model decision affecting a human interaction logged with: model version, confidence score, input hash, outcome, human decision; IAB has continuous read access
31. FR-M28-031: sovereignty; all on-device models packaged in KIEMS firmware; no internet calls during inference; central models on Kenyan-jurisdiction infrastructure; no biometric data transiting non-Kenyan cloud endpoint
32. FR-M28-032: CCB approval; every production AI model change requires CCB sign-off per PPADA 2015; CCB log in evidentiary bundle; CCB approval ID embedded in activation event
33. FR-M28-033: PII stripping middleware; mandatory for all LLM/NMT calls; names, national ID numbers, biometric references, sub-county location replaced with anonymised tokens; if stripping service unavailable, downstream LLM call rejected; stripping logs retained 7 years
34. FR-M28-034: model registry availability >= 99.9% for public model card reads; SLO alert in place
35. FR-M28-035: on-device model update cycle via signed OTA (P-OTA) only; kit refuses unsigned model payload; cosign-verified
36. FR-M28-036: central model update cycle via CCB-approved pipeline only; no direct file uploads to production; all deployments emit eel.ai.model_activated.v1 exactly-once
37. FR-M28-037: model rollback target < 30 min from trigger to previous version active; rollback procedure documented per model; drill run at T-30
38. FR-M28-038: no model deployed to production without passing eval harness with published minimum thresholds; CI gate enforces
39. FR-M28-039: DPIA registered with ODPC for: A1-A3 biometric processing, A4 ID OCR, C1 chatbot, D1-D2 CIB/deepfake detection; DPIA reference ID stored in model registry entry
40. FR-M28-040: chatbot response includes "powered by AI, may make mistakes, contact the election commission for official information" disclosure in all delivery channels
41. FR-M28-041: chatbot knowledge base structured from verified election-authority data only; LLM cannot access unverified external URLs; escalation threshold conservative
42. FR-M28-042: chatbot escalates to human on: pre-publication result queries, legal questions, sensitive complaints, low-confidence responses; escalation path clearly labelled
43. FR-M28-043: model activation requires HSM-backed signature (P06); chair HSM key; CCB approval ID embedded; eel.ai.model_activated.v1 emitted exactly-once
44. FR-M28-044: IAB 7-day hold mechanism: if IAB invokes delay right, model activation blocked until hold expires or written justification accepted; ECC Commander notified
45. FR-M28-045: AI decision log exported as CSV for IAB; columns: model, version, interaction_type, confidence_score, model_output, human_decision, decision_at; no PII columns
46. FR-M28-046: PII stripping circuit-breaker: if stripping service unavailable, downstream LLM call rejected; alert raised; stripping bypass count = 0 in audit
47. FR-M28-047: PIIStrippingEvent stored in T2; original input not retained; stripped_input_hash (SHA-256) only; log retained 7 years
48. FR-M28-048: AI decision log appended at-least-once idempotent; duplicate log entry ID returns cached entry; no re-execution
49. FR-M28-049: A4 OCR field mismatch logged as AIDecisionLog entry; human decision recorded by Presiding Officer; no autonomous field rejection
50. FR-M28-050: all biometric data paths (A1-A3 inference pipelines) network-isolated to Kenyan-IP address space; CISO packet capture at T-30 and T-day
51. FR-M28-051: sovereignty attestation report: CISO report to IAB confirming zero non-Kenyan biometric endpoint touches; report signed and anchored in P05
52. FR-M28-052: A6 on-device anomaly telemetry payload contains no voter PII; only station_code, alert_type, confidence_score, timestamp
53. FR-M28-053: B3 alert rate-limited to prevent alert flood; ranked by severity before delivery to ECC; dedup window applied
54. FR-M28-054: C2 translation: back-translated diff stored in TranslationApproval record; Comms Officer reviews source vs back-translation before publication
55. FR-M28-055: D3 result labelled "Hash Match (Verified)" vs "Visual Similarity (Advisory)" so citizen understands confidence level
56. FR-M28-056: model card versioned and anchored; each published card emits eel.ai.model_card_published.v1; SHA-256 hash of card markdown stored in ModelCardPublication; merkle_node_id recorded
57. FR-M28-057: eval harness CI-integrated; every PR touching a model asset triggers harness run; minimum thresholds enforced as CI gate; results uploaded to model card before deploy
58. FR-M28-058: consumer lag on eel.ai topics monitored; alert fires if lag > 60 s; backpressure applied; no OOM
59. FR-M28-059: secrets (API keys, HSM handles, SVID private keys) excluded from all logs, traces, AI decision log fields; CI regex gate + log-scanner fixture
60. FR-M28-060: SpiceDB enforces role boundaries for model activation, AI decision log access, OCR result access; fail closed on SpiceDB unavailability
61. FR-M28-061: duplicate model registration same key+body returns cached response; no re-execution
62. FR-M28-062: same model registration key, different body returns 422 idempotency-key-reuse
63. FR-M28-063: API rate limiting on all /v1/ai endpoints per rate table; 429 with Retry-After header; no enumeration surface
64. FR-M28-064: post-election AI performance report published at T+31; per-model: eval accuracy vs election-day outcomes, alert volumes, false-positive rates, human override rates
65. FR-M28-065: WCAG 2.2 AA on all voter-facing AI outputs (chatbot web, voice-prompt screens, pre-mark result display); axe-core audit in CI
66. FR-M28-066: B5 turnout flag threshold: > 3 sigma from constituency mean per rolling Form 34A window; flag generated as advisory; no automatic hold
67. FR-M28-067: B7 SIEM alerts advisory only; no autonomous firewall rule or block applied; human SOC analyst reviews all before action
68. FR-M28-068: A5 TTS scripts locked in firmware and human-approved per language; no runtime TTS call during inference; no network needed
69. FR-M28-069: B4 confidence below auto-clear threshold generates adjudicator referral; adjudicator makes all dedup decisions; referral logged in AIDecisionLog
70. FR-M28-070: eel.ai.model_card_published.v1 and eel.ai.iab_notice_sent.v1 events delivered exactly-once; Kafka producer idempotent

## Acceptance criteria

1. AC-M28-01: on-device liveness false rejection rate < 1%; measurement: eval harness + election-day log; IAB sign-off
2. AC-M28-02: Form 34A OCR pipeline accuracy >= 98% on tally fields; measurement: post-election OCR vs manual check audit
3. AC-M28-03: AI decision log covers 100% of human-affecting AI decisions; measurement: log completeness audit by IAB AI Observer
4. AC-M28-04: model cards published before production deployment for 100% of models; measurement: registry timestamp vs deploy timestamp; 0 models deployed without card
5. AC-M28-05: IAB 72-hour advance notice compliance 100%; measurement: notification log; all eel.ai.iab_notice_sent.v1 events precede activation by >= 72 h
6. AC-M28-06: biometric data touching non-Kenyan endpoint = 0 incidents; measurement: CISO packet capture audit; sign-off: Datacraft CISO
7. AC-M28-07: autonomous voter rejections = 0; measurement: decision log audit by IAB; all adverse A-model decisions have human_decision recorded
8. AC-M28-08: autonomous ballot rejections = 0; measurement: decision log audit by IAB; all B1/B2 OCR results have human approval event
9. AC-M28-09: chatbot official communications without human-approval loop = 0; measurement: translation approval decision log
10. AC-M28-10: PII tokens reaching LLM/NMT API = 0; measurement: PII stripping log audit; stripped_input_hash verified for all calls
11. AC-M28-11: model version rollback confirmed < 30 min in rollback drill at T-30; measurement: drill timestamp log
12. AC-M28-12: WCAG 2.2 AA audit pass on chatbot web interface; measurement: independent axe-core + manual audit report
13. AC-M28-13: all on-device models cosign-verified before kit accepts OTA payload; measurement: CI OTA pipeline log; unsigned payload rejected in test
14. AC-M28-14: D2 deepfake detection model card discloses state-level evasion limitation; measurement: model card review by IAB AI Observer before production deployment
15. AC-M28-15: consumer lag on all eel.ai topics stays below 60 s under simulated 55,393-form OCR load; measurement: k6 load test + Kafka lag monitor

## Applicable edge cases

**data**
- EC-DAT-01: duplicate model registration same key+body; cached response; no re-execution
- EC-DAT-02: same model registration key, different body; 422 idempotency-key-reuse
- EC-DAT-14: model card with oversized eval report blob > 1 MB; outbox rejects; blob stored in MinIO with hash reference

**security**
- EC-SEC-01: attempt to activate model without CCB approval signature; activation blocked; tamper topic emitted
- EC-SEC-03: HSM unavailable during model activation signing; activation paused; ingest continues; degraded-mode alert
- EC-SEC-04: PII stripping service unavailable mid-cycle; circuit-breaker rejects downstream LLM call; alert raised; 0 bypasses
- EC-SEC-05: injection on all AI API inputs (chatbot message, OCR submit); parameterised only; Semgrep + ZAP + fuzz
- EC-SEC-06: SSRF via chatbot URL field; allowlist applied; no private-range requests; rebinding guard
- EC-SEC-08: chatbot asked to provide pre-publication voter result; knowledge base from verified election-authority data; DTD off for XML parsers
- EC-SEC-12: cache poisoning of model registry snapshot; key includes sig version; client verifies; Varnish injection test
- EC-SEC-16: secrets absent from AI decision log, chatbot logs, PII stripping logs; CI regex gate + log-scanner fixture

**identity**
- EC-IDN-02: JWT expires mid-OCR approval request; in-flight completes; next 401 + silent refresh
- EC-IDN-10: SpiceDB down when SOC analyst accesses disinformation flags; fail closed; 503 problem+json
- EC-IDN-11: realm key rotated; stale JWKS for IAB observer JWT; refetch on kid miss; no access interruption

**scale**
- EC-SCL-03: 55,393 Form 34As through B1/B2 OCR pipeline within 4 hours; worker pool sized for peak
- EC-SCL-04: consumer lag on eel.ai.form34a.v1 > 60 s; page alert; backpressure applied; no OOM

**ML**
- EC-ML-01: on-device model drift between training and election day; eval harness run at T-30 on synthetic election-day data; recalibration if needed
- EC-ML-02: D2 deepfake adversarial input; confidence gate; no crash; limitation published in model card
- EC-ML-04: C2 translation bias across demographic groups; per-group metrics in tolerance; bias flag in model card
- EC-ML-05: PII in chatbot prompt before stripping; PIIStrippingEvent confirms replacement; original not stored; Semgrep enforced
- EC-ML-06: unsigned model payload pushed to kit OTA; cosign verification fails; kit refuses; model_refused event
- EC-ML-07: B3 anomaly alert flood during turnout surge; dedup + rate-limit + ranked delivery; ECC inbox not overwhelmed

**mobile**
- EC-MOB-01: A5 TTS voice prompt on low-battery kit; all 15 languages cached in firmware; no network needed

**operations**
- EC-OPS-03: central model cluster degraded on election day; fallback to advisory-only mode; "model unavailable" indicator; no autonomous clear
- EC-OPS-04: model activation requires dual-control; HSM unavailable; activation paused; ingest continues; alert raised

**connectivity**
- EC-CON-11: Kafka ISR shrink on eel.ai topics; producer blocks; outbox accumulates; page; no loss

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | PII stripping middleware (confirm no name/ID/biometric token passes through); model registry CRUD; AI decision log append; CCB approval gate | nextest + pytest | M28-UNIT- |
| property | decision log append idempotency; PII token count invariant | proptest | M28-PROP- |
| contract | A1-A8 kit firmware model interface, B1-B2 Form 34A OCR, C1 chatbot escalation, model registry API | Pact | M28-PACT- |
| integration | A1 liveness end-to-end on kit; B1 Form 34A OCR with synthetic forms; C1 chatbot escalation path; C2 translation approval workflow | testcontainers | M28-INT- |
| eval harness | per-model: accuracy, bias, adversarial robustness; results published in model card before production | pytest | M28-EVAL- |
| security | attempt to activate model without CCB; attempt to route chatbot without PII stripping; confirm no biometric in any network log | ZAP + cargo-fuzz | M28-SEC- |
| adversarial | A1: printed photo, video replay, 3D mask; A4: adversarial patch on ID document; D2: frontier GAN-generated video | external red team | M28-ADV- |
| sovereignty | network packet capture during election-day simulation; confirm all biometric data within Kenyan-IP address space | packet capture | M28-SOV- |
| a11y | chatbot WCAG 2.2 AA; TTS voice quality review by native speakers per language | axe-core + manual | M28-A11Y- |
| load | 10,000 concurrent chatbot sessions; B1 OCR handling 55,393 forms within 4 hours | k6 | M28-LOAD- |
| rollback | model version rollback drill; confirm < 30 min from trigger to previous version active | nextest | M28-ROLLBACK- |

Tags: `#[trace("FR-M28-031","AC-M28-06","EC-SEC-04")]`

## Runbooks and dashboards

- `docs/runbooks/ai-platform-svc-degradation.md` - extend with model cluster partial failure; advisory-only fallback
- `docs/runbooks/pii-stripping-failure-response.md` - new runbook; circuit-breaker alert; incident classification; ODPC notification if PII leaked
- `docs/runbooks/model-rollback-procedure.md` - new runbook; < 30 min target; CCB-approved rollback steps
- `docs/runbooks/iab-model-objection-response.md` - new runbook; 7-day hold; written justification process
- Grafana dashboard: `platform/observability/ai-platform-svc/ai-monitoring-dashboard.json`
- SLO alerts: `platform/observability/ai-platform-svc/ai-slo-alerts.yaml` (model registry 99.9%, OCR pipeline < 5 min/form, chatbot < 3 s p95)

## Open questions

- Kenyan-jurisdiction IaaS: Konza Technopolis / Icolo / Safaricom Cloud / Liquid Telecom Nairobi must be contracted and SLA agreed before central model deployment; timing risk.
- NLLB-family NMT service: Kenyan-hosted instance at 192.0.2.11 (ml/vault server) may need migration to Konza/Icolo for DPA compliance; confirm with Datacraft CISO.
- Social media monitoring API: aggregated feed from which provider? No individual user data; sourcing and MoU must be agreed before D1/D4 integration sprint.
- ODPC DPIA registration: five DPIAs required; the Commission DPO must lead; timeline risk if ODPC review takes > 90 days.
- sw/ spec references LiteLLM at 192.0.2.11:4000; confirm this remains the designated Kenyan-tenant endpoint or flag migration required; ADR requires Kenyan-jurisdiction hosting.
