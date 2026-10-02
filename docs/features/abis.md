# ABIS (M03)

Module M03 (`abis`) owned by `abis-svc`. Deduplicates the Kenya voter register at 30M+ scale using multi-modal biometric matching (fingerprint, iris, facial) inside confidential-compute enclaves (Intel TDX / AMD SEV-SNP). Feeds adjudication queues for ambiguous matches and provides the register integrity guarantee that no citizen is enrolled twice.

## Contents

1. What M03 does
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

## 1. What M03 does

M03 is the definitive 1:N dedup engine for the voter register. M01 includes an on-device ABIS-lite first-pass; M03 performs the authoritative gallery search. It also handles 1:1 verification (M05 polling-day identification), batch dedup, cancelable template management, and FAR/FRR monitoring across demographic strata.

The full algorithm catalogue is in `services/abis-svc/docs/CAPABILITIES.md` (500+ lines). This document extracts the key pipeline, algorithm citations, and external prerequisites; readers needing algorithm-level detail should consult that file directly.

## 2. Data plane

Migrations `V001`-`V032` under `services/abis-svc/migrations/`:

| Aggregate | Purpose | Data class |
|---|---|---|
| **TemplateIndex** | index_id, voter_id, modality, encrypted feature vector, enclave_attestation_id | A |
| **MatchRequest** | request_id, voter_id, modalities, status, enqueued_at | C |
| **MatchResult** | result_id, request_id, outcome, confidence_score, modality_scores | C |
| **AdjudicationCase** | case_id, match_result_id, voter_ids, status, investigator_id, decision, reason_code | B |
| **EnclaveAttestation** | enclave_id, worker_id, TEE quote (TDX / SEV-SNP), verified_at | C |

Templates stored field-encrypted inside enclave-only storage; raw templates never observable outside the enclave or encrypted store.

## 3. Ingest sources

| Source | Notes |
|---|---|
| BVR back-end (M02) | 1:N match requests via Kafka `eel.abis.match.v1`; gRPC `POST /abis/v1/match/1n` for on-demand |
| BVI back-end (M06) | 1:1 verification requests on polling day via gRPC `POST /abis/v1/match/11` |
| Batch import (initial load) | `crates/abis-bulk-ops/src/import.rs`; streaming ingest with progress tracking |
| Soft-hold from CVR (M35) | `POST /internal/soft-hold` retains template during adjudication; no hard delete without dual confirm |

Modalities: fingerprint (10-print + latent), face, iris, palm (10 palms x 5 zones), signature, periocular, ear, gait, vein, voice (see `services/abis-svc/docs/CAPABILITIES.md` section 1).

## 4. Pipeline

Crates: `services/abis-svc/crates/abis-enrolment/src/pipeline.rs`, `abis-api/`, `abis-adjudication/`, `abis-accelerator/`, `abis-matcher/`.

```
MatchRequest enqueued (Kafka / gRPC)
      -> abis-attest: verify enclave TEE quote (TDX / SEV-SNP)
             -- IETF RATS RFC 9334 remote attestation
      -> abis-enhancement: ridge enhancement (Gabor filter + Otsu),
             slap segmentation, iris/face segmentation
      -> quality gate: NFIQ 2.2 (NIST SP 500-333) / ISO 29794-5 face /
             ISO 29794-6 iris
      -> PAD: fingerprint liveness (Level 1/2 ISO 30107-3),
              face PAD (3D Morphable Model via abis-face-3dmm,
              Basel Face Model, Paysan et al. AVSS 2009),
              iris PAD
      -> abis-cancelable: fuzzy-commitment transform (Dodis-Reyzin-Smith
             IEEE S&P 2004) or random-projection biohash
             -- cancelable template; original not stored
      -> 1:N search:
             demographic Bloom pre-prune (abis-search-planner)
          -> HNSW ANN top-K (Malkov-Yashunin IEEE TPAMI 2020)
          -> exact re-rank (per-modality matcher ensemble):
               fingerprint: MCC (Cappelli et al. IEEE TPAMI 2010)
                           + DeepPrint (Engelsma et al. IEEE TPAMI 2021)
                           + Bozorth-style anchor + SourceAFIS
               face: ArcFace / AdaFace / CosFace cosine (CVPR 2019/2022/2018)
               iris: Hamming distance on 2048-bit IrisCode
          -> multi-modal fusion via WeightedVoting (abis-fusion-advanced)
             with Platt-scaled per-matcher calibration
      -> route decision:
             score > threshold -> open AdjudicationCase
             no match          -> EnrolmentJob -> Enrolled
             below quality     -> Rejected, re-enrolment request
      -> abis-drift-monitor: Population-Stability-Index drift alert
             (PSI Siddiqi 2006) on FAR/FRR per demographic stratum
      -> HSM countersign MatchResult (P06)
      -> emit eel.abis.match_result.v1 (Kafka eel.abis.match.v1)
```

Throughput target: >= 10,000 enrolment 1:N matches/hr; >= 100 concurrent 1:N queries sustained. Hardware acceleration dispatch via `crates/abis-accelerator/src/dispatcher.rs` (CPU / GPU / FPGA).

## 5. API surface

Contract: `packages/openapi-contracts/abis/openapi.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /abis/v1/match/1n` (gRPC) | BVR back-end (SVID mTLS) | 1:N dedup request from enrolment pipeline |
| `POST /abis/v1/match/11` (gRPC) | BVI back-end (SVID mTLS) | 1:1 polling-day verification |
| `POST /abis/v1/enrol` (gRPC) | BVR back-end (SVID mTLS) | Add template to gallery |
| `POST /abis/v1/soft-hold` | CVR (SVID mTLS) | Retain template during CVR adjudication |
| `GET /abis/v1/adjudication` | Commission investigator (JWT + SpiceDB) | Adjudication workbench queue |
| `PATCH /abis/v1/adjudication/{id}` | Commission investigator (JWT + SpiceDB) | Approve / reject; biometric-auth + dual-control |
| `GET /abis/v1/reports` | Commission admin (JWT + SpiceDB) | FAR/FRR, throughput, fairness metrics |
| `GET /abis/v1/enclave/attestation` | Third-party certifier (read-only) | TEE quote + match-set evidence |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Adjudication workbench (Django) | `services/abis-svc/django/abis-admin/` | Investigator minutiae overlay + side-by-side comparison; dual-control resolution |
| Reports endpoint | `crates/abis-api/src/routes/reports.rs` | FAR/FRR trends, throughput, adjudication SLA, fairness per stratum |
| Third-party certifier read-only interface | REST (read-only SpiceDB role) | Independent audit of test-set results and enclave evidence |

Adjudication SLA: first response <= 4 h, resolution <= 24 h; timer visible in workbench.

## 7. Integrity + anti-tamper

- **Enclave attestation** (Intel TDX / AMD SEV-SNP) verified on every worker via `crates/abis-attest/` (IETF RATS RFC 9334). Unattested workers cannot access template data.
- **Cancelable templates** via fuzzy-commitment (Dodis, Reyzin, Smith, IEEE S&P 2004) or random-projection biohash (`crates/abis-cancelable/`); original biometric not stored after transform; templates revocable and renewable.
- **HSM countersignature** on every MatchResult; signing queued if HSM unavailable; alert paged.
- **Split-view and drift detection** via `crates/abis-drift-monitor/`; PSI-based alerts on FAR/FRR population shift.
- **SHAP/LIME explainability** (`crates/abis-explainability/`) for every adjudicated case; saliency map logged alongside decision.
- **Audit shadow** (`V008__audit_shadow.sql`) records every API call with actor and outcome.
- **Soft-hold on CVR** prevents hard delete of any template while a removal is under adjudication.
- **Enclave correlation** (`V026__correlation_and_integrity.sql`) ties each MatchResult to its verified enclave_id.
- **Template protection** (`V031__template_protection.sql`) enforces cancelable token lifecycle.

## 8. Kenya statute alignment

- **Kenya Data Protection Act 2019 s.26 (minimisation)** -- cancelable templates only; no original biometric stored post-enrolment.
- **Kenya Data Protection Act 2019 s.31 (security)** -- TEE enclave isolation; field encryption; TLS 1.3 minimum.
- **Kenya Data Protection Act 2019 s.41 (automated processing)** -- every match above threshold routes to human adjudication; no automated rejection or removal.
- **Elections (Technology) Regulations 2017** -- independent third-party certifier interface (`GET /abis/v1/enclave/attestation`) for KEBS / NIST FRVT certification evidence.

## 9. Honest caveats + external prerequisites

- **NIST FRVT participation** for the face embedding model is an external prerequisite; FRVT test submission must be completed and results published before G3.
- **ISO/FBI MINEX III certification** for the fingerprint matcher stack is a shared risk with M01; both resolved together.
- **Confidential-compute hardware allocation** (TDX / SEV-SNP nodes at scale) requires infrastructure provisioning before G3.
- **Per-group fairness thresholds** across Kenyan demographic strata need the Commission data-governance sign-off; fairness tolerance parameters are configurable but baseline calibration requires representative enrolment data.
- **ABIS-lite on M01** uses a compact ISO 19794-2 matcher; ABIS-lite and M03 gallery share no state; sync mechanism is server-driven.
- Third-party certifier MoU (read-only access to test results and enclave evidence) is a legal prerequisite outside this repo.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (matcher algorithms, enclave attestation parser, cancelable transform, quality gates) | nextest | M03-UNIT- |
| Property (no false accept above threshold at stated FAR; PSI drift within tolerance) | proptest | M03-PROP- |
| Contract (BVR 1:N callback; BVI 1:1 callback) | Pact | M03-PACT- |
| Integration (SoftHSM; testcontainers PG + Kafka; enclave simulation) | nextest + testcontainers | M03-INT- |
| Load (10,000 1:N/hr; 100 concurrent; 30M gallery) | k6 | M03-LOAD- |
| Chaos (enclave worker kill; HSM kill; Kafka ISR shrink) | Chaos Mesh | M03-CHAOS- |
| Security (Semgrep PII ban; ASVS L3; enclave integrity probe) | Semgrep + ZAP | M03-SEC- |

Runbook (to create): `docs/runbooks/abis-svc-degradation.md` (enclave worker failure; adjudication backlog; HSM unavailable).

Full spec: `docs/plans/modules/abis.md`. Algorithm catalogue: `services/abis-svc/docs/CAPABILITIES.md`. Test harness: `services/abis-svc/crates/*/tests/`.
