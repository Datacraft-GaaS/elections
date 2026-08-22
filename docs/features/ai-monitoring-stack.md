# AI monitoring stack (M28)

Module M28 (`ai-monitoring-stack`) owned by `ai-platform-svc`. Governs all AI and ML model usage across the IEMS: a typed model registry with five tiers (A analytics, B biometric, C classification, D detection, E generation), cosign-signed model artefacts verified at load time, decision logs anchored in the P05 transparency log, a PII-stripping circuit-breaker applied before any data reaches LLM or ML layers, LiteLLM + NLLB as Kenyan-jurisdiction doubles for commercial models, and a gated chatbot workflow requiring DPO approval before any LLM is exposed to election data.

## Contents

1. What the AI monitoring stack does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What the AI monitoring stack does (vs related modules)

- **M26 Anomaly detection** = consumer of M28-managed models (IsolationForest for Layers 1-2); M26 calls the model registry to resolve current model versions.
- **M28** = governance layer over all AI/ML: registry, signing, audit logging, PII controls, and the chatbot gate. M28 does not implement any detection logic itself.
- **M27 Intent preservation** = consumer of M28 A4 vision model for ballot mark classification.

M28 is the single mandatory approval gate for any model that processes election data. No model is deployed to production without a cosign-signed artefact and a DPO review for data-processing implications.

## 2. Data plane

Five aggregates under `services/ai-platform-svc/crates/ai-registry-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **ModelRecord** | Registry entry: model_id (UUID7), tier (A/B/C/D/E), name, version, cosign_digest, approved_by (DPO), data_classes_permitted (list), deployed_at. | B |
| **DecisionLog** | Per-inference audit record: log_id, model_id, consumer_module, input_hash (no raw input), output_summary, pii_strip_applied (bool), ts_hlc, tlog_ref. | A |
| **PiiStripEvent** | PII detection and strip result: event_id, consumer_module, detected_entities (count, types), stripped (bool), circuit_breaker_tripped (bool), ts_hlc. | A |
| **ChatbotRequest** | Gated LLM interaction: request_id, requestor_module, prompt_hash, dpo_approved (bool), approval_ref, response_summary, ts_hlc. | B |
| **ModelTierPolicy** | Tier-level data access policy: tier, permitted_data_classes (list), requires_dpo_review (bool), requires_hsm_sig (bool). | D |

Model tier catalogue:

| Tier | Codes | Examples | Data classes permitted |
|---|---|---|---|
| A (analytics) | A1-A8 | IsolationForest, z-score, intent classifier | Aggregated D-class; never individual C/B/A |
| B (biometric) | B1-B7 | Fingerprint match, face liveness | B-class biometric under strict HSM; never leaves biometric-edge-svc |
| C (classification) | C1-C4 | Category classifier (M23), OCR (M22) | B/C-class under PII strip |
| D (detection) | D1-D4 | Ballot mark vision (M27), anomaly ensemble (M26) | Aggregated B-class; image hash only |
| E (generation) | E1-E2 | Chatbot (gated), NLLB translation | Approved non-PII only; DPO approval mandatory |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Model artefact upload (CI/CD pipeline) | Per release | OCI image + cosign signature | CI service account (SVID) | Upload rejected if cosign verification fails |
| Consumer module inference calls | Real-time | gRPC / REST | SVID mTLS | Inference blocked if model not in registry or cosign invalid |
| PII scanner (eel-pii-strip-rs) | Per inference | Internal | Library call | Circuit breaker trips on PII detection; inference halted; DPO alert |
| DPO approval workflow (Django) | Per chatbot request | Form | JWT + SpiceDB DPO_OFFICER | Chatbot request held until approval received |
| LiteLLM / NLLB router (62.169.25.77:4000) | Per E-tier request | REST | SVID mTLS | Fallback: LiteLLM -> NLLB -> local rule-based; circuit breaker on LiteLLM failure |

## 4. Pipeline

```
model deployment (CI/CD):
  OCI image built; cosign sign (Sigstore, keyless via OIDC)
  -> cosign verify at registry upload
  -> ModelRecord created; DPO review required for E-tier and new data classes
  -> model available for consumer modules after DPO approval

inference request (consumer module):
  consumer calls ai-registry-api with model_id + input_hash
  -> cosign digest verified against registry
  -> PII strip applied (eel-pii-strip-rs):
       detected PII (DPA 2019 sensitive categories):
         PiiStripEvent(circuit_breaker_tripped=true)
         inference halted; DPO alert; caller receives HTTP 422
       no PII: inference proceeds
  -> model inference executed (on-host or LiteLLM router)
  -> DecisionLog created: input_hash, output_summary, pii_strip_applied
  -> DecisionLog.tlog_ref = P05 Merkle anchor (within 60 s)

chatbot (E1/E2):
  requestor_module submits ChatbotRequest with prompt_hash
  -> DPO approval check:
       dpo_approved = false: request held; DPO notified
       dpo_approved = true: proceed
  -> PII strip applied (mandatory; no bypass)
  -> routed to LiteLLM (62.169.25.77:4000)
  -> if LiteLLM unavailable: NLLB (for translation tasks); rule-based fallback
  -> DecisionLog anchored; response_summary stored (no raw prompt/response)
```

## 5. API surface

Contract: `packages/openapi-contracts/ai-platform/registry.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /ai/v1/models` | CI service account (SVID) | Register model artefact with cosign digest |
| `GET /ai/v1/models/{id}` | Consumer module | Resolve current model version and cosign status |
| `POST /ai/v1/infer` | Consumer module (SVID) | Request inference (PII strip + audit applied) |
| `GET /ai/v1/decisions` | DPO / IAB (JWT + SpiceDB) | Decision log feed |
| `GET /ai/v1/pii-events` | DPO (JWT + SpiceDB) | PII strip event feed |
| `POST /ai/v1/chatbot` | Approved module (SVID) | Submit gated chatbot request |
| `PATCH /ai/v1/chatbot/{id}/approve` | DPO officer (JWT + HSM) | DPO approval for chatbot request |
| `GET /ai/v1/policies` | Public (CDN-cached) | Tier policy catalogue |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Model registry admin (Django) | `services/ai-platform-svc/django/ai-admin/` | DPO reviews model releases; sets tier policies; approves chatbot requests |
| AI monitoring dashboard (Grafana) | `platform/observability/ai-platform-svc/grafana-ai.json` | Model inference rate, PII strip event rate, circuit breaker status, decision log anchor lag |
| Public tier policy page (Astro) | `services/ai-platform-svc/astro/ai-public/` | Published tier policies and data class permissions |

WCAG 2.2 AA on all web surfaces. Languages: EN + SW.

## 7. Integrity + anti-tamper

- **Cosign verification at load** -- every model inference call verifies the cosign digest against the registry before executing; a digest mismatch returns HTTP 409 and logs a `MODEL_TAMPER` event that pages the IEBC SOC.
- **PII strip circuit-breaker** -- the `eel-pii-strip-rs` library uses Named Entity Recognition to detect DPA 2019 Schedule 2 sensitive categories (voter ID, biometric reference, financial data) before any data reaches an ML or LLM layer; the breaker is not configurable by the consumer module; only DPO officers can update the detection corpus.
- **Decision log P05 anchor** -- every `DecisionLog` is anchored in the P05 transparency log within 60 s; the anchor log is the authoritative record of what the AI systems processed during the election.
- **Chatbot response not stored** -- the raw LLM response is not persisted; only a `response_summary` (operator-written) and the response hash are stored; this prevents the LLM output from becoming a long-term PII liability.
- **Regression corpus for PII strip** -- a static regression corpus of 1 000 synthetic ballots is run against the PII strip library before every release; any corpus failure blocks the release.

## 8. Kenya statute alignment

- **DPA 2019 s.40 (automated processing)** -- every model that influences a decision affecting a voter's rights requires DPO review and is logged in the `ModelRecord.approved_by` field; the DPO review record is retained for 7 years.
- **DPA 2019 s.26 (minimisation)** -- PII strip circuit-breaker enforces minimisation at the model boundary; no more data is passed to a model than its `data_classes_permitted` policy allows.
- **DPA 2019 s.31 (data processor agreements)** -- LiteLLM routes to the Kenyan-jurisdiction double at 62.169.25.77:4000 by default; commercial models (OpenAI, Anthropic) are E-tier and require explicit DPO approval and a data processor agreement before being enabled.
- **Elections (Technology) Regulations 2017** -- AI model audit trail retained for 7 years; cosign signatures and decision logs form part of the statutory audit trail.

## 9. Honest caveats + external prerequisites

- The NLLB model (on 62.169.25.77) covers Swahili, Kikuyu, and Luo but translation quality for legal and electoral terminology has not been independently evaluated; NLLB output must be reviewed by a human officer before being used in any public-facing communication.
- The LiteLLM proxy at 62.169.25.77:4000 is a single point of failure for E-tier inference; the fallback to NLLB covers translation tasks only; chatbot requests cannot be served offline.
- The DPO approval workflow adds latency to chatbot requests; any chatbot feature that requires real-time response cannot use the gated workflow and must be pre-approved for a specific prompt category.
- The cosign keyless OIDC signing approach requires a working Sigstore Fulcio and Rekor instance; the infrastructure team must confirm these are accessible from the CI/CD environment before the first model release.
- The PII strip NER model must be trained and evaluated on Kenyan electoral documents; using a generic English NER model without domain adaptation will produce false negatives for Kenyan names and ID number formats.

## 10. Test coverage + operational runbooks

Coverage under `services/ai-platform-svc/crates/ai-registry-api/tests/`: unit tests for cosign verifier, PII strip circuit-breaker, tier policy enforcer, decision log anchor, chatbot approval gate; regression corpus (1 000 synthetic ballots) run in CI; property-based tests asserting DecisionLog always created for every inference call; contract tests (Pact) for M26 anomaly model calls, M27 intent model calls, M23 category classifier; integration tests with SoftHSM DPO approval, Sigstore stub, LiteLLM stub, NLLB stub, P05 tlog stub; e2e scenario covering model registration, DPO approval, PII-strip pass and trip, chatbot gated request, and decision log anchor; load tests (k6) for 10 000 inference calls per minute.

Runbooks under `docs/runbooks/`:

- `ai-monitoring-degradation.md` (create): cosign verification failure; PII circuit-breaker trip; LiteLLM unavailable; decision log anchor lag.
- `model-tamper-incident.md` (create): immediate response for MODEL_TAMPER event.

Dashboards: `platform/observability/ai-platform-svc/grafana-ai.json` (inference rate by tier, PII strip event rate, cosign failure count, decision log anchor lag, chatbot approval queue depth). SLO: cosign verification <100 ms; PII strip <50 ms; decision log anchor <90 s.

Related: `docs/plans/modules/ai-monitoring-stack.md`; `services/ai-platform-svc/docs/CAPABILITIES.md`; edge-case matrix EC-AIM-* in `docs/plans/08-edge-case-matrix.md`.
