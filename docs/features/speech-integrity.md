# Speech integrity (M17)

Module M17 (`speech-integrity`) owned by `ai-platform-svc` + `transparency-svc`. Content flag pipeline, coordinated inauthentic behaviour (CIB) clustering, deepfake detection, Kenyan-language bias reporting, social media platform integration, and NCIC (National Cohesion and Integration Commission) referral interface.

## Contents

1. What M17 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M17 does

M17 monitors the election information environment for harms: synthetic or manipulated audio/video (deepfakes), coordinated inauthentic behaviour networks (CIB), and election-related disinformation. It produces content flags with confidence scores, clusters coordinated accounts, generates a bias report across Kenyan languages, provides a platform API for social media companies to receive flags, and routes confirmed findings to NCIC for statutory action.

M17 produces advisory flags only; all enforcement actions are taken by human decision-makers or the relevant authority. No automated content removal or account suspension is performed by this system.

## 2. Data plane

Crates: `services/ai-platform-svc/crates/ai-gateway/src/routes/anomaly.rs`, `decision_log.rs`, `registry.rs`. Python workers: `services/ai-platform-svc/py/`. Migrations: `services/ai-platform-svc/migrations/`.

| Aggregate | Purpose | Data class |
|---|---|---|
| **ContentFlag** | flag_id (UUID7), content_url_hash, content_hash, flag_type (DEEPFAKE/CIB/DISINFORMATION/HATE_SPEECH), confidence_score, model_id, language_detected, ts | B |
| **CibCluster** | cluster_id, account_ids_hashed[], coordination_signal (TIMING/CONTENT/NETWORK), confidence, cluster_size, first_seen, last_seen | B |
| **DeepfakeFinding** | finding_id, content_hash, modality (AUDIO/VIDEO/IMAGE), detection_model, confidence, spectral_evidence_ref | B |
| **BiasReport** | report_id, language, model_id, false_positive_rate, false_negative_rate, demographic_strata[], period, published_at | B |
| **NcicReferral** | referral_id, flag_ids[], content_summary, severity, referred_by, referred_at, ncic_receipt_id, status | B |
| **DecisionLog** | log_id, flag_id, actor (human or model_id), decision, reason, ts_hlc | C |

No raw content stored; only content_hash and URL hash. All model decisions logged via `ai-gateway/src/routes/decision_log.rs`.

## 3. Ingest sources

| Source | Notes |
|---|---|
| Social media platform webhooks | Partner webhook; signed HMAC; content hash submitted (not raw content) |
| Citizen report intake (PWA / portal) | Citizens submit URL + hash; OTP-verified to prevent spam |
| NCIC referral channel | Structured referral submission; NCIC provides receipt ID |
| Public RSS / Atom monitoring | `integrations/` web crawlers hash-only; no content storage |
| Kenyan language corpora (bias calibration) | Offline; loaded into bias-report model at training time |

## 4. Pipeline

Crates: `services/ai-platform-svc/crates/ai-gateway/src/`, Python workers: `services/ai-platform-svc/py/`.

```
content hash + URL submitted (via webhook or citizen report)
      -> PII strip: ai-gateway/src/pii_strip.rs
             strip names, IDs, phone numbers before any model sees input
      -> model registry lookup (ai-gateway/src/routes/registry.rs):
             route to appropriate model(s) based on content type:
               image/video  -> deepfake detection pipeline
               text (SW/EN/KI/LU) -> hate speech / disinformation classifier
               account graph  -> CIB clustering
      -> deepfake detection (DeepfakeFinding):
             video: spatiotemporal coherence check (face-swap inconsistency)
               -- FaceForensics++ benchmark (Rossler et al. ICCV 2019)
             audio: mel-spectrogram artifact detection
               -- RawNet2 (Jung et al. Interspeech 2020)
             image: frequency-domain DCT artifact analysis
               -- Durall et al. CVPR 2020
      -> CIB clustering (CibCluster):
             timing co-ordination: shared burst posting patterns
               -- botometer-derived temporal features (Davis et al. ICWSM 2016)
             content co-ordination: near-duplicate hash (SimHash Charikar 2002)
             network co-ordination: graph community detection (Louvain algorithm,
               Blondel et al. J. Stat. Mech. 2008)
      -> language bias check:
             model applied to Kenyan language test set (SW, KI, LU, KA, EN)
             false positive rate per language per stratum logged
             -> BiasReport updated per model version
      -> ContentFlag created with confidence score
      -> ai-gateway/src/routes/decision_log.rs: log every model decision
      -> human review queue (confidence < threshold -> mandatory human review)
             commission content analyst reviews
             decision logged with actor = analyst_id
      -> platform notification:
             if confidence > threshold AND human confirmed:
             POST to social media platform API with flag
      -> NCIC referral (severity CRITICAL or human-confirmed):
             NcicReferral created -> POST to NCIC referral endpoint
             ncic_receipt_id stored; referral status tracked
      -> emit eel.ai.content_flagged.v1
             eel.ai.ncic_referral_submitted.v1
```

## 5. API surface

Contracts: `packages/openapi-contracts/ecc/speech.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /speech/v1/flags` | Platform webhook (HMAC) or citizen (OTP) | Submit content for flagging |
| `GET /speech/v1/flags` | Commission analyst (JWT + SpiceDB) | Review flag queue |
| `PATCH /speech/v1/flags/{id}/decision` | Commission analyst (JWT + SpiceDB) | Record human decision |
| `GET /speech/v1/clusters` | Commission analyst (JWT + SpiceDB) | CIB cluster view |
| `POST /speech/v1/referrals` | Commission analyst (JWT + SpiceDB) | Create NCIC referral |
| `GET /speech/v1/bias-reports` | Commission admin + public | Published bias reports |
| `GET /speech/v1/models` | Commission admin (JWT + SpiceDB) | Registered model inventory |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Content analyst console | `services/ai-platform-svc/ex/` (Phoenix or React) | Flag review, decision logging, NCIC referral |
| Bias report public portal | `services/transparency-svc/astro/` | Published language-bias reports |
| Citizen report intake | M11 public portal or PWA | URL submission for flagging |

Languages: analyst console in English + Kiswahili. Bias reports published per language (SW, KI, LU, KA, EN minimum).

## 7. Integrity + anti-tamper

- **PII strip before model ingestion** (`ai-gateway/src/pii_strip.rs`) -- names, IDs, and phone numbers removed before any model processes input; no personal data in model training path.
- **No raw content storage** -- only content_hash and URL hash stored; raw content never persists in this system.
- **Immutable decision log** (`ai-gateway/src/routes/decision_log.rs`) -- every model and human decision appended with actor, ts_hlc; no in-place mutation.
- **Model registry versioning** (`ai-gateway/src/routes/registry.rs`) -- each model version registered with hash; decisions traceable to specific model version.
- **Human review gate** -- flags below confidence threshold go to mandatory human review before any action; fully-automated flagging is not possible.
- **NCIC referral receipt** -- NCIC receipt ID stored; referral cannot be marked submitted without receipt; provides external accountability.
- **Bias report publication** -- bias reports signed with transparency-svc Ed25519 key and published to P05 tlog leaf; tampering detectable.
- **Audit trail** -- every flag, decision, referral, and platform notification logged to EccAuditEvent with actor and ts.

## 8. Kenya statute alignment

- **National Cohesion and Integration Act 2008 (NCIC)** -- NCIC referral interface implements the statutory channel for reporting election-related hate speech and incitement; receipt ID provides the formal referral record.
- **Computer Misuse and Cybercrimes Act 2018 s.22 (false publication)** -- ContentFlag with human confirmation provides the documented basis for referral to enforcement authorities.
- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- content_hash only; no personal data stored; PII strip before model processing.
- **Kenya Data Protection Act 2019 s.41 (automated processing)** -- no automated enforcement; human review gate mandatory for any action; decision log records human actor.
- **ARTICLE 33 (freedom of expression, Constitution of Kenya 2010)** -- advisory flags only; no automated content removal; human and judicial due process preserved.

## 9. Honest caveats + external prerequisites

- **Model accuracy on Kenyan languages** (Kikuyu, Luo, Kamba, Somali) -- all three detection models were originally trained on English/European corpora; Kenyan-language fine-tuning and bias evaluation are in flight and must complete before G3. Bias reports are the accountability mechanism.
- **Social media platform API MoUs** -- M17 can submit flags to platforms only if those platforms have agreed to receive them; no platform is compelled to act on flags. Platform API credentials are external prerequisites.
- **NCIC API endpoint** -- NCIC referral integration requires NCIC to expose a structured intake API; current integration is a structured POST but the live endpoint and credentials are pending NCIC agreement.
- **Deepfake detection false-positive rate** on Kenyan video (compressed, variable-lighting) is unknown; field calibration on representative content required before live deployment.
- **CIB clustering** relies on account graph data from platforms; this data is not available without platform MoU; without it, only timing and content co-ordination signals are available.
- M17 is currently **in flight**: `ai-gateway/crates/ai-gateway/` exists but Python model workers and NCIC integration adapter are deferred to Wave 4 (G4) per plan.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (PII strip, model routing, deepfake score threshold, CIB cluster merge, bias report aggregator) | nextest + pytest | M17-UNIT- |
| Property (no raw content persists; decision log immutable; bias report monotone accumulation) | proptest | M17-PROP- |
| Contract (platform webhook; NCIC referral; tlog bias-report anchor) | Pact | M17-PACT- |
| Integration (model stub; NCIC double; platform webhook double; tlog double) | nextest + testcontainers | M17-INT- |
| Bias evaluation (language-stratified false-positive / false-negative measurement on test corpus) | pytest + bias harness | M17-BIAS- |
| Security (PII leakage probe; model inversion attack; HMAC bypass; NCIC referral forgery) | Manual + ZAP | M17-SEC- |

Runbook (to create): `docs/runbooks/ai-platform-svc-degradation.md` (model worker down; NCIC endpoint down; platform webhook flood; bias report publication failure).

Full spec: `docs/plans/modules/speech-integrity.md`. Crates: `services/ai-platform-svc/crates/ai-gateway/`. Transparency anchoring: `services/transparency-svc/crates/tlog-api/`, `tlog-ingest/`. Bias report publish: `services/transparency-svc/astro/`.
