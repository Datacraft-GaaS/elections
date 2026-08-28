# M28 — AI Monitoring Stack

**Module ID:** M28
**Name:** On-Device and Central AI Monitoring Stack (Sovereignty-Constrained)
**Runs on:** KIEMS kit (on-device, INT8 quantised) + Central Kenyan cloud (model serving + monitoring)
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4; underlying OSS model bases under their respective licences (Apache 2.0, CC-BY-NC-4.0 as specified per model)

## 1. Purpose

Deploy a governed ensemble of AI models around — but never inside — the result-determining pipeline. On-device models handle real-time quality control and operational support at the kit; central models handle form processing, anomaly detection, translation, communication, and disinformation monitoring. A model registry with model cards, eval harness, version pinning, Kenyan-jurisdiction hosting, and IAB notification for every production change enforces the governance frame.

Bright red lines (hard technical and contractual prohibitions):
1. No AI model in any vote-tally or result-aggregation pipeline.
2. No autonomous voter rejection — every adverse AI output requires a named human decision.
3. No training on live voter biometric data.
4. No LLM in the result-determining pipeline at any layer.
5. No biometric data transiting a non-Kenyan cloud endpoint.

Strategy reference: §X.

## 2. User Personas

- **Presiding Officer** — benefits from on-device quality feedback without operational overhead
- **Voter** — benefits from improved accessibility (voice guidance, language support), faster throughput
- **IEBC Form Processing Officer** — receives AI-assisted Form 34A OCR queue; all results human-confirmed
- **ECC Analyst** — receives anomaly alerts from central AI; all alerts advisory
- **SOC Analyst** — benefits from AI-assisted SIEM; all alerts reviewed before action
- **IEBC Communications Officer** — approves AI-assisted translations and summaries before publication
- **Datacraft AI Lead** — maintains model registry, model cards, eval harness, deployment pipeline
- **IAB AI Observer** — receives notification of every production model change; has read access to all decision logs
- **ODPC Data Protection Officer (IEBC)** — verifies DPA 2019 compliance for all AI processing activities
- **Researcher / Civil Society** — downloads model cards and AI decision logs from the public registry

## 3. Functional Requirements

### Kit-Layer (On-Device) Models

- **FR-M28-001** A1 — Face liveness detection: depth-map + texture CNN, INT8 quantised, <= 15 MB. Detects photo or video-playback impersonation at voter capture and authentication. Runs in <= 500 ms on kit reference hardware. Presiding Officer can override any liveness rejection with dual-agent witness + reason code. No liveness model decision is autonomous.
- **FR-M28-002** A2 — Fingerprint quality scoring: ISO 29794-4 compliant quality model, edge inference, <= 5 MB. Computes quality score before template extraction; prompts for retake if below threshold. Increases first-try authentication success rate. No quality score is used to reject a voter; low quality triggers a retake prompt only.
- **FR-M28-003** A3 — Iris quality scoring: ISO 29794-6 compliant model, edge inference, <= 5 MB. Same design as A2. No autonomous rejection.
- **FR-M28-004** A4 — ID document OCR cross-check: CRNN/TrOCR fine-tuned on Kenyan ID types (2nd-gen ID, Maisha card, Passport), INT8, <= 8 MB. Compares OCR-extracted biographic fields against manually entered fields. Flags mismatches for Presiding Officer confirmation. Presiding Officer can accept or reject each field independently. No OCR output creates an automatic rejection.
- **FR-M28-005** A5 — Voice-guided prompts in 15 Kenyan languages: TTS model with Kenyan-accented voices, INT8, <= 20 MB. Used for audio guidance at every step of the BVR and BVI workflows. Scripts generated and human-approved in each language before deployment.
- **FR-M28-006** A6 — On-device anomaly detection: rule-based threshold + lightweight classifier (Isolation Forest variant, INT8, <= 3 MB). Detects station-level anomalies per M26 Layer 1 specification. Alert sent to ECC; Presiding Officer not blocked from any action.
- **FR-M28-007** A7 — Pre-mark ballot validation: vision model per M27 Element 2 specification. On-device only. Image deleted within 1 second of inference.
- **FR-M28-008** A8 — Face age estimation (BVR only): lightweight CNN estimating voter age range (over/under 18) as a secondary check during enrolment. Age estimate is advisory; Presiding Officer confirms age against ID document. No voter rejected on age-model output alone.

### Central-Layer Models

- **FR-M28-009** B1 — Form 34A image processing pipeline: document AI (orientation detection, de-skew, OCR, quality scoring) for every Form 34A photograph transmitted from a kit. Below-quality-threshold forms flagged and held in human review queue; forms are never auto-rejected or auto-published without human sign-off. Kenyan-cloud-hosted.
- **FR-M28-010** B2 — Handwriting recognition on tally lines: Kenyan-election-form-tuned HTR model. Extracts candidate vote counts from handwritten Form 34A tally rows. Human review required before any extracted count is recorded; HTR output is a suggestion, not a result.
- **FR-M28-011** B3 — Anomaly detection on kit telemetry: LSTM / Isolation Forest on time-series telemetry per M26 Layer 2 specification. Central counterpart to A6. All alerts advisory.
- **FR-M28-012** B4 — Duplicate-voter suggestion for ABIS reviewer: ABIS 1:N matching engine with confidence score bands. Confidence below the auto-clear threshold produces an adjudicator referral; adjudicator makes all dedup decisions. Not autonomous.
- **FR-M28-013** B5 — Turnout implausibility flag: statistical model on rolling Form 34A updates. Flags stations with turnout > 3 sigma from constituency mean; feeds M26 Layer 2.
- **FR-M28-014** B6 — Transmission-gap pattern alert: threshold + trend model on transmission-event time series. Feeds M26 Layer 1.
- **FR-M28-015** B7 — ML-assisted SIEM: Isolation Forest / unsupervised network anomaly detection. SOC analyst reviews all alerts; no autonomous network action.

### Citizen-Layer (Public-Facing)

- **FR-M28-016** C1 — Multilingual chatbot: LLM with IEBC election knowledge base. Deploys on WhatsApp, Telegram, and SMS. PII-stripped input (names, national ID numbers, biometric references, and location data below county level stripped before any API call). Escalates to a human agent on complex, sensitive, or ambiguous queries. Hosted on Kenyan-tenant LiteLLM instance (62.169.25.77:4000 or designated Kenyan cloud tenant).
- **FR-M28-017** C2 — Real-time announcement translation: NMT model (NLLB-family or equivalent) for 15 Kenyan languages. IEBC Communications Officer approves every translated text before publication. Back-translation QA on all sensitive content.
- **FR-M28-018** C3 — Text-to-speech for radio and IVR: TTS with Kenyan-accented voices. Human reviews all scripts before production.
- **FR-M28-019** C4 — Speech-to-text for grievance intake: ASR model fine-tuned for Kenyan languages. Transcribes voice-note grievances from M23 kit kiosk and WhatsApp voice messages. Human agent reviews transcript; transcript used as intake aid only.

### Disinformation Monitoring

- **FR-M28-020** D1 — CIB detection: graph-based coordinated-inauthentic-behaviour classifier + velocity model on Kenyan social media. Human analyst reviews HIGH-confidence flags. Published daily summary to IAB and IEBC Comms.
- **FR-M28-021** D2 — Deepfake detection: FaceForensics-family CNN on viral video flagged by analysts or social-media monitoring triggers. Human analyst confirms before any action. Honest limitation: state-level deepfakes may evade current detectors; this is a best-available deployment, not a guarantee.
- **FR-M28-022** D3 — Fake Form 34A detection: hash lookup + visual similarity model. Compares circulating Form 34A images against published hashes. Any citizen can query via Public Verifier; results are deterministic for hash matches, advisory for visual-similarity matches.
- **FR-M28-023** D4 — Bot-network detection on hashtag-hijacking: network analysis + bot-score model. Human analyst reviews patterns; distinction between coordinated-inauthentic and legitimate coordinated campaigns made by human.

### Internal Operations

- **FR-M28-024** E1 — AI-assisted code review: static analysis + LLM code review on every KIEMS software change in CI pipeline. Developer + security reviewer approve all changes; LLM output is a prompt, not a gate.
- **FR-M28-025** E2 — Adaptive poll-worker training assessments: IRT model on assessment response patterns. Trainer reviews flagged poor performers; remediation is human-designed.

### Governance

- **FR-M28-026** Model registry: every model in production has a registry entry containing: model ID, version, training data description (no PII, no biometric data), eval harness results, known limitations, deployment date, IAB notification date, rollback procedure, model card URL.
- **FR-M28-027** Model card: published on the public portal for every on-device and central model. Format: purpose, architecture, training data source, eval metrics, known failure modes, human-in-loop controls, version history. Published in EN; summary in SW.
- **FR-M28-028** Eval harness: every model has a CI-integrated eval suite. Minimum eval dimensions: accuracy on task, bias across demographic groups where applicable, adversarial robustness on a held-out test set. Eval results published with model card.
- **FR-M28-029** IAB notification: every production model change (new model, version update, threshold change) triggers a 72-hour advance notice to IAB with the model card diff and eval delta. IAB may request a 7-day delay for review; Datacraft implements or justifies not implementing.
- **FR-M28-030** AI decision log: every central AI model decision that affects a human interaction (OCR flagged, adjudicator referral, chatbot escalation, translation approval required) is logged with: model version, confidence score, input hash, outcome, human decision. IAB has read access continuously.
- **FR-M28-031** Sovereignty: all on-device models packaged into KIEMS firmware; no internet calls during inference. All central models hosted on Kenyan-jurisdiction infrastructure (Konza Technopolis / Icolo / Safaricom Cloud / Liquid Telecom Nairobi). No biometric data transits a non-Kenyan cloud endpoint.
- **FR-M28-032** CCB approval: every production AI model change requires Change Control Board sign-off per PPADA 2015 requirements. CCB log is part of the evidentiary bundle.
- **FR-M28-033** PII stripping middleware: for all LLM/NMT calls, input is passed through a Kenyan-hosted PII stripper (names, national ID numbers, biometric references, and sub-county location data replaced with anonymised tokens) before any API call. Stripping logs retained for 7 years.

## 4. Non-Functional Requirements

- **NFR-M28-001** On-device liveness detection (A1): <= 500 ms inference time on reference KIEMS hardware.
- **NFR-M28-002** On-device quality scoring (A2, A3): <= 200 ms.
- **NFR-M28-003** On-device ID OCR (A4): <= 2 seconds.
- **NFR-M28-004** Central Form 34A processing pipeline (B1, B2): <= 5 minutes per form.
- **NFR-M28-005** Chatbot response latency (C1): <= 3 seconds for 95th percentile.
- **NFR-M28-006** All on-device models: INT8 quantised, offline-capable, no network dependency during inference.
- **NFR-M28-007** Model registry availability: >= 99.9% for public model card reads.
- **NFR-M28-008** WCAG 2.2 AA: all voter-facing AI outputs (chatbot responses, voice prompts, pre-mark validation screens).
- **NFR-M28-009** Languages: EN + SW for all public model cards; 15 Kenyan languages for TTS/ASR/chatbot.
- **NFR-M28-010** DPA 2019 compliance per AI processing activity: DPIA registered with ODPC for biometric processing (A1-A3), OCR (A4), chatbot (C1), CIB/deepfake detection (D1-D2). Lawful basis documented per activity.
- **NFR-M28-011** Model update cycle: on-device models updated via signed OTA only (P-OTA); central models deployed via CCB-approved pipeline; both have rollback < 30 minutes.
- **NFR-M28-012** No model deployed to production without passing the eval harness with published minimum thresholds.

## 5. User Stories

1. As a voter presenting a printed photograph for impersonation, liveness detection alerts the Presiding Officer, who requests alternative verification; no voter is rejected without a human decision.
2. As a voter with worn fingerprints (common in manual labourers), quality scoring prompts for iris capture as an alternative; the voter authenticates successfully.
3. As an IEBC Form Processing Officer, I see a Form 34A flagged by B1 OCR because it is rotated 90 degrees; I confirm the rotation correction and approve publication.
4. As a Kenyan-language speaker in Dholuo, I interact with the chatbot on WhatsApp entirely in Dholuo and receive correct information about polling-station hours without switching to English.
5. As an IAB AI Observer, I receive a 72-hour advance notice that Datacraft is updating the liveness detection model; I review the model card diff and approve deployment within the notice period.
6. As a SOC Analyst, I see a B7 SIEM alert about an anomalous traffic pattern from a constituency's KIEMS cluster; I investigate and determine it is a misconfigured router; the AI did nothing autonomously.
7. As a member of the public, I download the model card for the Form 34A OCR model and see its accuracy on a held-out test set and its known failure modes.
8. As the IEBC Communications Officer, I approve a Gikuyu translation of a press release before it is published; the back-translation summary shows no material error.
9. As a researcher, I query the AI decision log via the IAB read-only API and confirm that zero chatbot responses were published without human review.
10. As the Datacraft CISO, I confirm in the IAB quarterly report that no biometric data touched a non-Kenyan cloud endpoint in the preceding quarter.

## 6. Data Model Additions

```python
class AIModel(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str                         # uuid7str()
    code: str                       # e.g. "A1", "B1", "C1"
    name: str
    version: str                    # semantic version
    layer: Literal["KIT","CENTRAL","CITIZEN","DISINFORMATION","INTERNAL"]
    architecture: str
    training_data_description: str  # no PII; source citation
    eval_accuracy: float | None
    eval_bias_flag: bool
    eval_adversarial_robustness: str | None
    known_limitations: str
    human_in_loop: str
    sovereign_hosting: str          # e.g. "on-device", "Kenyan cloud (Safaricom)"
    model_card_url: str
    deployed_at: datetime
    iaab_notified_at: datetime
    ccb_approval_id: str
    rollback_procedure: str
    is_active: bool

class AIDecisionLog(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    model_id: str
    model_version: str
    interaction_type: str           # e.g. "OCR_FLAG","CHATBOT_ESCALATION","TRANSLATION_APPROVAL"
    input_hash: str                 # SHA-256 of PII-stripped input
    confidence_score: float | None
    model_output: str               # structured, no PII
    human_decision: str | None      # recorded when a human acts on the output
    human_actor_id: str | None
    human_decision_at: datetime | None
    decision_at: datetime

class ModelCardPublication(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    model_id: str
    version: str
    published_at: datetime
    card_hash: str                  # SHA-256 of published model card markdown
    merkle_node_id: str | None
    languages: list[str]            # EN mandatory; SW for summary

class PIIStrippingEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    model_id: str
    original_input_length: int
    stripped_input_hash: str        # SHA-256 of stripped text; original not stored
    pii_tokens_replaced: int
    stripped_at: datetime
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| GET | /v1/ai/models | Public | 500/min | List all registered AI models with model card links |
| GET | /v1/ai/models/{id} | Public | 500/min | Single model registry entry |
| GET | /v1/ai/models/{id}/card | Public | 500/min | Download model card (markdown) |
| POST | /v1/ai/models | DATACRAFT_AI_LEAD JWT + CCB_APPROVAL | 5/day | Register new model or version |
| PATCH | /v1/ai/models/{id}/activate | DATACRAFT_AI_LEAD JWT + IEBC_CHAIR HSM | 5/day | Activate model version in production |
| GET | /v1/ai/decision-log | IAB_OBSERVER JWT | 100/hr | Query AI decision log (no PII) |
| POST | /v1/ai/chatbot/message | Authenticated user or public | 500/min | Send message to election chatbot (PII-stripped) |
| POST | /v1/ai/translation/request | IEBC_COMMS JWT | 100/hr | Request NMT translation |
| POST | /v1/ai/translation/{id}/approve | IEBC_COMMS JWT | 100/hr | Human-approve a translation before publication |
| GET | /v1/ai/disinformation/flags | SOC_ANALYST JWT | 100/hr | List disinformation detection flags |
| PATCH | /v1/ai/disinformation/flags/{id} | SOC_ANALYST JWT | 100/hr | Record human analyst verdict |
| POST | /v1/ai/form34a/submit-image | RTS_BACKEND_CERT | 500/hr | Submit Form 34A image for OCR processing |
| GET | /v1/ai/form34a/{id}/ocr-result | IEBC_OFFICER JWT | 200/hr | Fetch OCR result for human review |
| POST | /v1/ai/form34a/{id}/approve | IEBC_OFFICER JWT | 200/hr | Human-approve Form 34A after OCR review |

## 8. Integration Points

- **M01 BVR Device / M05 BVI Device** — on-device models A1-A8 packaged in kit firmware
- **M03 ABIS** — B4 duplicate-voter suggestion feeds ABIS adjudicator queue
- **M10 RTS Back-End** — B1/B2 Form 34A processing; B3/B5/B6 telemetry analytics
- **M23 Grievance Channels** — C1 chatbot intake (M23 escalation path); C4 ASR for voice grievances
- **M26 Anomaly Detection** — B3/B5/B6/B7 central AI models feed the anomaly detection layers
- **M27 Intent Preservation** — A7 pre-mark validation model
- **P05 Merkle Chain** — model card publications and AI decision logs anchored
- **P06 HSM** — model activation requires IEBC Chair HSM
- **M11 Public Portal** — model registry, model cards, public AI decision log summary
- **LiteLLM on Kenyan cloud tenant** — C1 chatbot LLM routing layer (PII-stripped)
- **NLLB / NMT service** (Kenyan cloud) — C2 translation
- **Social media monitoring API** (aggregated; no individual user data) — D1/D4 inputs

## 9. UI/UX Key Screens

1. **Model Registry (Public)** — searchable table of all deployed models; version, layer, deployment date, model card link, eval accuracy; badge for IAB-notified status.
2. **Model Card Page** — structured markdown view: purpose, architecture summary, training data source, eval metrics, known limitations, human-in-loop controls, version history. Download as PDF. Language toggle (EN / SW).
3. **Form 34A OCR Review Console (IEBC Officer)** — side-by-side: form image and OCR-extracted tallies; confidence score per field; one-click accept/correct per field; final approve button.
4. **AI Decision Log Viewer (IAB)** — filterable log: model, interaction type, date range, confidence band; no PII fields visible; export to CSV.
5. **Chatbot Interface (WhatsApp/Web/SMS)** — standard chatbot UI; language selector; escalation path clearly labelled; powered-by disclosure ("This chatbot is powered by AI and may make mistakes — contact IEBC for official information").
6. **Disinformation Flag Dashboard (SOC Analyst)** — list of flagged items sorted by confidence; source attribution; flag type (CIB/deepfake/fake-form/bot-network); analyst verdict controls; timeline of flags by hour.
7. **Translation Approval Queue (IEBC Comms Officer)** — pending translations list; source text + NMT output side-by-side; back-translation summary; approve/reject/edit controls.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| No AI in result-determining pipeline | Strategy §X.1; Art. 138 Constitution (counting authority) |
| No autonomous voter rejection | Strategy §X.3; Elections Act s.44A (human Presiding Officer authority) |
| Biometric sovereignty | Strategy §X.4; DPA 2019 s.48 (data transfer restrictions) |
| AI model transparency | Strategy §X.1; IAB mandate; Access to Information Act 2016 s.4 |
| DPA 2019 DPIA for biometric AI | DPA 2019 s.31 (data protection impact assessments) |
| No training on live voter biometric data | Strategy §X.3; DPA 2019 s.26 (purpose limitation) |
| CCB approval for production changes | PPADA 2015; Election Technology Regulations 2017 |
| Human-in-loop for all adverse AI decisions | Strategy §X.1 principle; Elections Act s.44A |
| Chatbot PII stripping | DPA 2019 s.26; contractual AI provider terms |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| On-device liveness false rejection rate | < 1% | Eval harness + election-day log | IAB |
| Form 34A OCR pipeline accuracy | >= 98% on tally fields | Post-election OCR vs manual check | IEBC Chair |
| AI decision log coverage | 100% of human-affecting AI decisions | Log completeness audit | IAB AI Observer |
| Model cards published before production deployment | 100% | Registry timestamp vs deploy timestamp | Datacraft AI Lead |
| IAB 72-hour advance notice compliance | 100% | Notification log | IAB |
| Biometric data touching non-Kenyan endpoint | 0 incidents | CISO network audit | Datacraft CISO |
| Autonomous voter rejections | 0 | Decision log audit | IAB |
| Autonomous ballot rejections | 0 | Decision log audit | IAB |
| Chatbot responses without human-approval loop | 0 for official communications | Decision log | IEBC Comms Officer |
| PII tokens reaching LLM/NMT API | 0 | PII stripping log | Datacraft DPO |

## 12. Test Strategy

- **Unit:** PII stripping middleware (confirm no name, national ID, or biometric token passes through); model registry CRUD; AI decision log append.
- **Integration:** A1 liveness detection end-to-end on kit; B1 Form 34A OCR pipeline with synthetic forms; C1 chatbot escalation path; C2 translation approval workflow.
- **Eval harness (per model):** accuracy, bias, adversarial robustness on held-out test sets. All results published in model card before production deployment.
- **Security:** attempt to activate a model without CCB approval; attempt to route a chatbot query without PII stripping; confirm no biometric data in any network log.
- **Adversarial (per-model):** A1 liveness: printed photo, video replay, 3D mask; A4 OCR: adversarial patch on ID document; D2 deepfake: frontier GAN-generated video.
- **Sovereignty audit:** network packet capture during election-day simulation; confirm all biometric data stays within Kenyan-IP address space.
- **Accessibility:** chatbot WCAG 2.2 AA compliance; TTS voice quality review by native speakers per language.
- **Load:** 10,000 concurrent chatbot sessions; B1 OCR pipeline handling 55,393 forms within 4 hours.
- **Rollback:** model version rollback drill — confirm < 30 minutes from trigger to previous version active.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| On-device model A1-A6 training and eval complete | 2026-10-31 | Eval harness published; model cards drafted |
| Central model B1-B7 deployed to staging | 2026-12-01 | Kenyan cloud staging environment |
| Chatbot C1 live (test elections) | 2027-01-15 | Deployed on Kenyan-tenant LLM; 5 languages |
| Model cards published on public portal (T-90) | 2027-05-11 | All active models |
| IAB AI Observer briefing | 2027-05-15 | Model registry walkthrough |
| Kit firmware with on-device models | 2027-06-01 | A1-A8 bundled in firmware |
| All models production-ready (T-30) | 2027-07-11 | CCB sign-off; eval harness green |
| Post-election AI performance report | 2027-09-10 | T+31; eval harness vs election-day outcomes |

## 14. Dependencies

- M01/M05 BVR/BVI Devices — on-device model packaging and OTA delivery
- M03 ABIS — B4 adjudicator queue integration
- M10 RTS Back-End — B1/B2 Form 34A pipeline; B3/B5/B6 telemetry
- M23 Grievance Channels — C1 chatbot intake; C4 ASR for voice notes
- M26 Anomaly Detection — central model outputs as detection layer inputs
- M27 Intent Preservation — A7 pre-mark validation model
- P05 Merkle Chain — model card and decision log anchoring
- P06 HSM — model activation signing
- M11 Public Portal — model registry and public AI decision log
- LiteLLM Kenyan-tenant deployment (infrastructure dependency — 62.169.25.77:4000 or migrated Kenyan cloud instance)
- Kenyan cloud IaaS (Konza / Icolo / Safaricom Cloud / Liquid Telecom Nairobi) — central model hosting

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| State-level deepfake evades D2 detection | Low | High | D2 is best-available-open-model deployment; human analyst required before any action; public commitment in model card that detection is not a guarantee for state-level deepfakes; detection gap is published, not concealed |
| Chatbot hallucination provides incorrect election information to a voter | Medium | Medium | LLM knowledge base is structured from verified IEBC data only; LLM cannot access unverified external URLs; response includes "for official information contact IEBC on [number]"; escalation threshold is conservative |
| PII stripping middleware fails and voter identity reaches LLM endpoint | Low | High | PII stripping operates as a mandatory middleware layer with a circuit-breaker: if stripping service is unavailable, the downstream LLM call is rejected, not passed through with unstripped data; stripping log alerts on any bypass |
| Model card published with incorrect eval metrics | Low | Medium | Eval harness is CI-integrated; eval metrics automatically pulled from CI output into the model card template; manual override requires dual-approval |
| IAB objects to a model deployment within the 72-hour notice window | Medium | Low (operationally) | IAB objection triggers a 7-day hold and a written response from Datacraft; deployment proceeds only if IAB receives a written justification and does not seek a court order; this has never occurred in precedent elections with similar oversight frameworks |
