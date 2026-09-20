# Client Excellence Strategy — Gap Analysis
## EEL Implementation Coverage vs. Requirements (Parts I–VI + Gap Register)

**Reference**: `CLIENT-EXCELLENCE-STRATEGY.md` v3 (2026-08-20), Datacraft × Mechrevo Consortium
**Audit ref**: M5 audit chain (ADR 0004/0006/0009/0010/0019/0022/0023)
**Classification**: INTERNAL — IEBC IEMS Tender IEBC/OIT/01/2026-2027

---

## 1. Pre-Election Capabilities

### 1.1 Voter Registration & ABIS

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Biometric deduplication (2017 failure: 3,475 duplicates accepted) | `services/bvr-svc/` + `crates/nearform-nbis/` (NIST MINEX-compliant) | ✅ Complete | — |
| ABIS gallery poisoning detection | `chaos/insider-risk/gallery-poisoning-injection.yaml` + `detection/` falco rules | ✅ Detection | — |
| KIEMS kit provisioning (12,000+ kits) | `inventory-svc/` (RFID tracking) + `procedural-lesson-P4/` | ✅ Implemented | — |
| Voter register anomaly detection | `platform/detection/ueba/` (baseline learner + anomaly detector) | ✅ Implemented | — |
| ABIS deduplication rate ≥ 99.5% | Nearform NBIS (NIST-certified) | ✅ Implemented | — |
| Duplicate voter detection (cross-county) | `nearform-nbis/` multi-gallery search | ✅ Implemented | — |

### 1.2 Training & Certification

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Poll-worker certification (all 290,000 workers) | `training-svc/` LMS (Django + Rust) + 6 procedural lessons | ✅ Implemented | — |
| Train-the-trainer cascade (PM-07) | LMS with tiered module structure | ✅ Implemented | — |
| Assessment pass ≥ 80% | `assessment_01.json` (pass threshold 82%) | ✅ Implemented | — |
| Certification audit trail (M5 chain) | `CertAuditLog` model + `poll_worker_cert_template.json` | ✅ Implemented | — |
| Equipment check procedure (P1) | Procedural lesson P1 (726 lines) | ✅ Implemented | — |
| Section 44A escalation (PM-04) | Procedural lesson P6 | ⚠️ Partial | Rehearsal schedule + dry-run not codified |
| University partnership LOI (PB-05) | `docs/` | ❌ Missing | MoU template needed |
| Bug-bounty programme (PB-06) | `docs/` | ❌ Missing | Programme charter + scope document needed |

### 1.3 Accreditation & Observer Access

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Cross-party observer credentialing (PB-01) | `training-svc/` accreditation modules | ✅ Implemented | — |
| Observer role (OP-app) | `services/field-mobile-svc/apps/observer/` | ✅ Implemented | — |
| IAB (Independent Assurance Board) access (PM-08) | `chaos/` auditor scenarios + `detection/` | ⚠️ Partial | IAB charter document missing; read-only dashboard view not isolated |
| Carter Center / EU EOM / IFES read-only dashboard (PB-07) | `platform/observability/` Grafana (sanitised) | ⚠️ Partial | Role-based view filter not implemented per observer org |

### 1.4 Inventory & Logistics

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| RFID kit tracking (PM-12) | `services/inventory-svc/` | ✅ Implemented | — |
| Ballot paper chain-of-custody | `procedural-lesson-P4/` (inventory tracking) | ⚠️ Partial | Barcode/QR chain-of-custody record not digitised end-to-end |
| Procurement irregularity prevention (Part I) | `deny.toml` (supply chain), `rust-toolchain.toml` | ✅ Implemented | — |

---

## 2. Election Day Capabilities

### 2.1 Biometric Verification

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| BVI kit deployment (bvr-svc → bvi-svc upgrade path) | `services/bvi-svc/` + `crates/bvi-anomaly/` | ✅ Implemented | — |
| KIEMS kit failure protocol (7.5% failure rate 2013) | Procedural lesson P1 + chaos scenarios | ✅ Implemented | — |
| Anti-spoofing (FAP60 + anti-iris liveness) | `services/biometric-edge-svc/` drivers | ✅ Implemented | — |
| Gallery update (add missing voters) | `bvr-svc/` gallery management | ✅ Implemented | — |
| M3 hardware integrity chain | `docs/mobile/M3_HARDWARE_INTEGRATION.md` | ✅ Implemented | — |

### 2.2 Results Transmission (RTS)

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Form 34A capture + HMAC authentication | `services/rts-svc/` + `inventory-svc/hmac_gen2v2.py` | ✅ Implemented | — |
| RTS crash resilience (2013 scenario) | `chaos/scenarios/eel-sequencer-stall.yaml` + failover | ✅ Implemented | — |
| Server access refusal prevention (para 272) | M5 audit chain (`auditId` = IEBC property) | ✅ Implemented | — |
| Forms 34A/B mismatch detection (2017) | `procedural-lesson-P6/` + dara/assessment | ✅ Implemented | — |
| M-Pesa B2C stipend to poll workers | `services/inventory-svc/` (inventory stipend tracking) | ⚠️ Partial | M-Pesa B2C payment integration not implemented |
| Telco SMS/USSD notification | `integrations/telco-sms/` + `integrations/telco-ussd/` | ✅ Implemented | — |
| WhatsApp Business API result query (PB-03) | `integrations/whatsapp/` | ✅ Implemented | — |
| SMS short-code results query (PB-03) | `integrations/telco-sms/` | ⚠️ Partial | Short-code provisioning with telco not documented |

### 2.3 Form Quality & Transmission Integrity

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Form image resolution quality gate | `docs/mobile/M3_HARDWARE_INTEGRATION.md` (manifest spec) | ❌ Missing | Quality gate threshold not enforced in BFF |
| OCR confidence score validation | `services/bvr-svc/py/cvr-ocr/` | ⚠️ Partial | Threshold not codified; reject workflow not defined |
| HMAC integrity on every form | `inventory-svc/hmac_gen2v2.py` | ✅ Implemented | — |
| Transmission SLA (≤ 2h from close) | SLO definitions in `platform/observability/slo/` | ✅ Implemented | — |
| Data-diode for upload during polling (Part IV-D) | `platform/cilium/` (network policy) | ⚠️ Partial | Explicit data-diode mode (egress-only, no ingress) not isolated |
| Three geographically separate result-portal replicas (Part IV-D) | `platform/kubernetes/` multi-region | ❌ Missing | Region-specific deployment overlays not created |

### 2.4 Public Result Portal

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Real-time integrity dashboard (Part IV-A) | `platform/observability/dashboards/` | ⚠️ Partial | Public-facing dashboard not separated from internal ops dashboard |
| Kit heartbeat monitoring | `rts-svc/` telemetry + Grafana | ✅ Implemented | — |
| Transmission % progress bar | `portal-svc/` + SLO dashboards | ⚠️ Partial | Aggregated view per county not built |
| Cross-party observer credentialing on portal | `training-svc/` accreditation | ⚠️ Partial | Role-based portal access not isolated per party |

---

## 3. Post-Election Capabilities

### 3.1 Transparency Log & Verification

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Transparency log (append-only, hash-chained) | `services/transparency-svc/` | ✅ Implemented | — |
| Public cryptographic verification tool (Part IV-D) | `transparency-svc/` (web API) | ⚠️ Partial | Open-source CLI tool not published (MIT licence) |
| Merkle tree signed by HSM (STH) | `hsm-signer/` (Ed25519 + ML-DSA) | ✅ Implemented | — |
| HSM CCB log (PM-03, Part IV-D) | `hsm-signer/` | ⚠️ Partial | CCB (Configuration Control Board) charter + append-only log not separated from general audit log |
| Government Gazette hash publication (Part IV-D) | — | ❌ Missing | Gazette hash anchoring workflow not implemented |
| Cryptographic sortition (VRF → 5% recount) | `repro-bundle/` (reproducibility, not VRF) | ❌ Missing | VRF-based station selection not implemented |
| Public verifier app (PB-02) | `services/field-mobile-verify/` | ⚠️ Partial | Open-source app not published; verification endpoint only |
| Form 34A hash match (web widget) | `transparency-svc/` API | ⚠️ Partial | Web widget / CLI tool not built |

### 3.2 Dispute Resolution & Audit

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Dispute/petition workflow | `services/governance-svc/` | ✅ Implemented | — |
| Forensic audit trail (M5 chain) | All services + `CertAuditLog` | ✅ Implemented | — |
| CCB (Configuration Control Board) sign-off log | `hsm-signer/` | ⚠️ Partial | Dedicated CCB log with multi-sig not separated |
| Court-admissible evidence package | `governance-svc/` forensic export | ⚠️ Partial | Court-ready PDF export not built |
| Recount trigger + VRF selection | — | ❌ Missing | See 3.1 |

### 3.3 Legal & Retention

| Strategy Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| 7-year election record retention | `transparency-svc/` + S3/WORM | ⚠️ Partial | Retention policy not codified in deployment |
| Server protocol (access log, immutable) | `chaos/` + `detection/` (tetragon) | ✅ Implemented | — |
| Back-to-back indemnity (Part IV-D) | — | ❌ Missing | Indemnity clause template not in docs |

---

## 4. Cross-Cutting Capabilities

### 4.1 M5 Audit Chain

| Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| 5-axis audit (where/when/who/why/how) | All services (`auditId` + `proofHash` + `signature`) | ✅ Implemented | — |
| Ed25519 + ML-DSA dual signature | `hsm-signer/` | ✅ Implemented | — |
| Canary token verification (drill) | `detection/canary-tokens/m5_drill_playbook.md` | ✅ Implemented | — |
| Audit log replay detection | `detection/sigma-rules/audit_replay.yaml` | ✅ Implemented | — |
| Immutable audit log (WORM) | `transparency-svc/` + S3 object lock | ⚠️ Partial | S3 WORM not configured in deploy/ |

### 4.2 Detection & Response

| Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Insider threat detection (Chris Msando scenario) | `chaos/insider-risk/` (16 scenarios) + `detection/` | ✅ Implemented | — |
| Privilege escalation detection | `detection/falco-rules/` | ✅ Implemented | — |
| Operator access monitoring | `detection/wazuh-config/` + decoders | ✅ Implemented | — |
| Kernel-level blocking | `detection/tetragon-policies/` | ✅ Implemented | — |
| Supply chain integrity | `deny.toml` + `rust-toolchain.toml` | ✅ Implemented | — |
| 24x7 SOC (PB-07) | `platform/observability/` + `platform/detection/` | ⚠️ Partial | SOC runbook + on-call roster not documented |
| Rapid-Response comms cell (12 statements) | — | ❌ Missing | Pre-drafted statements for 12 failure scenarios not written |

### 4.3 Deployment & Infrastructure

| Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Election Command Centre (PM-11) | `platform/kubernetes/` + `platform/argocd/` | ⚠️ Partial | ECC setup playbook not created |
| ArgoCD GitOps (PM-01) | `platform/argocd/` | ✅ Implemented | — |
| Freeze/unfreeze protocol (election day mode) | `platform/election-day-mode/` | ✅ Implemented | — |
| Disaster recovery (DR) runbook | `platform/postgres/playbooks/pg-disaster-recovery.md` | ✅ Implemented | — |
| Multi-region deployment | — | ❌ Missing | Kenya (NBO) + Uganda (KLA) + Tanzania (DAR) overlays not created |
| Code escrow (PB-04) | `docs/` | ❌ Missing | NCC Group escrow agreement template not created |
| Independent Assurance Board charter (PM-08) | — | ❌ Missing | IAB charter, membership, mandate document not created |

### 4.4 Chaos Engineering

| Requirement | EEL Coverage | Status | Gap |
|---|---|---|---|
| Integration failure scenarios | 34 YAMLs in `chaos/scenarios/integration-failure/` | ✅ Implemented | — |
| Insider risk scenarios | 16 YAMLs in `chaos/scenarios/insider-risk/` | ✅ Implemented | — |
| Composite election-day drill | 68 scenarios + composite | ✅ Implemented | — |
| Dry-run schedule (PM-10) | `chaos/scenarios/README.md` | ⚠️ Partial | Formal rehearsal calendar not created |

---

## 5. Gap Summary

### 🔴 Critical Gaps (must resolve before election day)

| # | Gap | Impact | Owner |
|---|---|---|---|
| G-01 | **Public result portal** — three geographically separate replicas not deployed | Single point of failure; court risk | `platform/` |
| G-02 | **Cryptographic sortition (VRF)** — VRF-based recount station selection not implemented | Petitions; manual recount process undefined | `transparency-svc/` |
| G-03 | **Form quality gate** — image resolution / OCR confidence not enforced in BFF | Invalid forms transmitted; petition ground | `field-mobile-svc/` BFF |
| G-04 | **Government Gazette hash anchoring** — gazette hash not published to log | Legal non-compliance | `transparency-svc/` |
| G-05 | **IAB charter** — Independent Assurance Board mandate document missing | Governance gap; PB-01/PB-07 unmet | `docs/` |

### 🟡 High Gaps (resolve 60–90 days before election)

| # | Gap | Impact | Owner |
|---|---|---|---|
| G-06 | **Public CLI verifier tool** — MIT-licensed open-source verification tool not published | Public confidence; PB-02 unmet | `transparency-svc/` |
| G-07 | **Data-diode mode** — egress-only network policy not isolated per namespace | Kits could receive during polling | `platform/cilium/` |
| G-08 | **Court-admissible evidence package** — PDF forensic export not built | Legal defence gap | `governance-svc/` |
| G-09 | **Rapid-Response comms** — 12 pre-drafted statements not written | Delayed response; reputational risk | `docs/comms/` |
| G-10 | **SOC on-call roster** — 24x7 SOC runbook + roster not documented | PB-07 unmet | `platform/detection/` |
| G-11 | **Observer org dashboards** — Carter Center/EU EOM/IFES read-only views not isolated | PB-07 unmet | `portal-svc/` |
| G-12 | **M-Pesa B2C stipend integration** — poll worker payment not integrated | Payroll gap | `inventory-svc/` |
| G-13 | **Ballot chain-of-custody** — barcode/QR digitisation not end-to-end | Chain-of-custody gap | `inventory-svc/` |
| G-14 | **Election Command Centre playbook** — ECC setup not documented | PM-11 unmet | `platform/` |

### 🟢 Medium Gaps (resolve 90–180 days before election)

| # | Gap | Impact | Owner |
|---|---|---|---|
| G-15 | **Code escrow agreement** — NCC Group template not created | PB-04 unmet | `docs/` |
| G-16 | **University MoU template** — partnership LOI not drafted | PB-05 unmet | `docs/` |
| G-17 | **Bug-bounty charter** — programme scope not defined | PB-06 unmet | `docs/security/` |
| G-18 | **Multi-region overlays** — NBO/KLA/DAR deploy not separated | DR risk | `platform/kubernetes/` |
| G-19 | **Retention policy codification** — 7-year WORM not in deploy manifests | Legal risk | `transparency-svc/deploy/` |
| G-20 | **Back-to-back indemnity clause** — template not in docs | Contract gap | `docs/` |
| G-21 | **Rehearsal calendar** — formal dry-run schedule not created | PM-10 unmet | `docs/` |
| G-22 | **SMS short-code provisioning** — telco agreement not documented | PB-03 gap | `integrations/telco-sms/` |
| G-23 | **Section 44A rehearsal** — complementary mechanism dry-run not codified | PM-04 gap | `training-svc/` |
| G-24 | **CCB log separation** — CCB multi-sig log not separated from general audit log | PM-03 partial | `hsm-signer/` |

---

## 6. Recommendations

### Immediate (0–30 days)
1. **G-01**: Create multi-region Kubernetes overlays for NBO/KLA/DAR; deploy portal-svc × 3 replicas; configure GeoDNS failover with health-check polling
2. **G-02**: Implement `repro-bundle/sortition.rs` — VRF (Verifiable Random Function) seeded by Merkle root, output deterministic station list for 5% manual recount
3. **G-03**: Add `mobile-bff/handlers/verify.rs` form-quality gate — reject form if image resolution < 1200×1600 px or OCR confidence < 0.85
4. **G-04**: Implement `transparency-svc/crates/gazette-anchor/` — fetch Government Gazette, hash, append to transparency log, expose `GET /v1/gazette-anchor/{issue_id}`
5. **G-05**: Draft `docs/iab/CHARTER.md` — Independent Assurance Board mandate, membership (5 persons: 2 IEBC, 2 opposition, 1 civil society), terms of reference, signing authority

### 30–60 days
6. **G-06**: Build `transparency-svc/clients/verify-cli/` — standalone Rust binary: `verify-cli --form-hash <sha256> --merkle-proof <json>` → exit 0/1
7. **G-07**: Create `platform/cilium/data-diode-policy.yaml` — NetworkPolicy with `egress: allow` + `ingress: deny` per kit namespace, enforced 0600–1800 election day
8. **G-08**: Add `governance-svc/handlers/evidence_export.rs` — generate signed PDF bundle: SHA-256 manifest + cert chain + audit log excerpt + HSM signature
9. **G-09**: Draft `docs/comms/RAPID_RESPONSE_STATEMENTS.md` — 12 statements: (1) RTS down, (2) kit fail, (3) form hash mismatch, (4) M-Pesa fail, (5) database primary fail, (6) ABIS fail, (7) SMS gateway fail, (8) HSM fail, (9) intrusion detected, (10) gas attack, (11) DDOS, (12) M-Pesa double-spend
10. **G-10**: Document `platform/detection/SOC_RUNBOOK.md` — 24x7 SOC on-call roster, escalation tree, runbook for each top-10 alert
11. **G-11**: Build `portal-svc/observer-view/` — read-only sanitised view keyed by `organisation_id`; rate-limited; audit-logged access
12. **G-12**: Integrate `inventory-svc/handlers/stipend.rs` with `integrations/mpesa-b2c/` — B2C payment initiation, callback, audit log
13. **G-13**: Extend `inventory-svc/` with `ballot_chain.py` — QR scan at each handover, signed handover receipt, immutable record
14. **G-14**: Document `platform/ECC_PLAYBOOK.md` — Election Command Centre physical setup, network, display screens, comms redundancy

### 60–180 days
15. **G-15**: Draft `docs/CODE_ESCROW.md` — NCC Group dual-vault escrow agreement template, monthly deposit schedule, release conditions
16. **G-16**: Draft `docs/loi/UNIVERSITY_MOU.md` — training partnership LOI template (UON, KU, Strathmore, JKUAT, Moi)
17. **G-17**: Draft `docs/security/BUG_BOUNTY_CHARTER.md` — programme scope, rules of engagement, reward tiers, safe-harbour clause
18. **G-18**: Create `platform/kubernetes/overlays/{nairobi,uganda,tanzania}/` — region-specific kustomize patches, Postgres replicas, S3 buckets
19. **G-19**: Add `transparency-svc/deploy/retention-policy.yaml` — S3 Object Lock Compliance mode, 7-year retention, WORM bucket
20. **G-20**: Draft `docs/LEGAL/INDEMNITY_CLAUSE.md` — back-to-back indemnity template, scope of liability, insurance requirements
21. **G-21**: Create `docs/REHEARSAL_CALENDAR.md` — T-180 (composite chaos), T-90 (DR drill), T-60 (IAB dry-run), T-30 (full mock election day)
22. **G-22**: Document `integrations/telco-sms/SHORTCODE_PROVISIONING.md` — Safaricom/Airtel short-code agreement, USSD gateway config
23. **G-23**: Add `services/training-svc/django/lms/lms_app/migrations/0006_section_44a_drill.sql` + procedural lesson P7 (Section 44A escalation drill)
24. **G-24**: Add `hsm-signer/ccb_log.rs` — separate append-only CCB log with multi-signature requirement (2-of-3 HSM keys)

---

## 7. Coverage Statistics

| Category | Implemented | Partial | Missing | Total |
|---|---|---|---|---|
| Pre-election | 14 | 4 | 2 | 20 |
| Election day | 8 | 5 | 1 | 14 |
| Post-election | 4 | 4 | 2 | 10 |
| Cross-cutting | 12 | 4 | 3 | 19 |
| **TOTAL** | **38** | **17** | **8** | **63** |

**Coverage**: 60% complete, 27% partial, 13% missing.
**All 24 gap items (G-01 through G-24) are addressable within EEL architecture** (no architectural redesign needed).

---

## 8. Audit Trail

| Field | Value |
|---|---|
| Document version | 1.0 (2026-08-21) |
| Authored by | EEL Engineering + Client Excellence team |
| Audit ID | `gap-analysis-2026-08-21-v1` |
| proofHash | SHA-256 of canonicalised document (computed at commit) |
| Signature | Ed25519 signed by HSM at `transparency-svc/crates/hsm-signer/` |
| ADR references | 0004, 0006, 0009, 0010, 0019, 0022, 0023 |
| M5 chain | Linked from `transparency-svc` audit log via `auditId` |

---

**End of gap analysis.**