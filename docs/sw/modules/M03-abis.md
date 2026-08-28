# M03 — ABIS (Automated Biometric Identification System)

**Runs on:** Primary + mirror DC with confidential-compute matching workers
**Owner:** Datacraft (or integration with existing IEBC ABIS, pending clarification C6)
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Deduplicate the Kenya voter register at scale of ≥ 30 million records. Provide 1:1 and 1:N matching across fingerprint, iris, and facial modalities with multi-modal fusion. Feed adjudication queues for ambiguous matches. Support the register's integrity guarantee that no citizen is enrolled twice.

## 2. Personas

- BVR Back-End (system principal — ingests match requests)
- Investigator (uses adjudication UI)
- Independent biometric certifier (annual FAR/FRR audit)
- Auditor

## 3. Functional Requirements

- **FR-M03-001** 1:N match for fingerprint (10 fingers), iris (both eyes), facial against ≥ 30M records.
- **FR-M03-002** 1:1 match for verification against a specific voter ID.
- **FR-M03-003** Multi-modal fusion: combine fingerprint, iris, facial scores per configurable policy.
- **FR-M03-004** Match / no-match / adjudicate outcome per configurable thresholds; thresholds set by IEBC.
- **FR-M03-005** Adjudication queue with side-by-side comparison, minutiae overlay, reference-image display, decision logging with reason codes.
- **FR-M03-006** Investigator workbench: search by biometric, by ID, by name; case management with SLA timer.
- **FR-M03-007** Configurable escalation: below-threshold quality returns "insufficient" outcome; above-threshold ambiguity returns "adjudicate".
- **FR-M03-008** Standards compliance: FBI IAFIS IQS + EBTS IAFIS-DOC-01078-9.0, ANSI/NIST-ITL 1-2011 Part 1 + ITL 2-2008 XML, WSQ + JPEG-2000 (500 PPI, ≤ 15:1 average), ISO/IEC 19794-2/5/6, ICAO 9303.
- **FR-M03-009** All matching runs inside confidential compute enclave (Intel TDX or AMD SEV-SNP); unencrypted biometric templates never leave enclave memory.
- **FR-M03-010** Continuous integration with the register: on new enrolment, 1:N against full register; on rejection, no template retained.
- **FR-M03-011** Optional integration with an existing IEBC ABIS (if retained per clarification C6): federation adapter delegates matching to existing ABIS with fallback.
- **FR-M03-012** Batch job: full-register dedup on demand or on schedule (e.g., quarterly).
- **FR-M03-013** ABIS reports: dedup counts, false-match rates, throughput, adjudication SLA compliance.
- **FR-M03-014** Independent third-party certification pathway: expose test-set interface for external certifier.
- **FR-M03-015** Every match request + outcome signed + audit-logged.

## 4. Non-Functional Requirements

- **NFR-M03-001** 1:N against 30M records: fingerprint p99 ≤ 8 s; iris p99 ≤ 5 s; facial p99 ≤ 6 s.
- **NFR-M03-002** Throughput: 100+ concurrent 1:N queries sustained.
- **NFR-M03-003** FAR ≤ 0.01% at operational threshold; FRR ≤ 1% at same threshold.
- **NFR-M03-004** Adjudication queue SLA: first response ≤ 4 h; resolution ≤ 24 h.
- **NFR-M03-005** Confidential compute attestation on every enclave; failure quarantines the worker.

## 5. Integrations

- BVR Back-End (M02) — ingest
- Confidential Compute (Intel TDX / AMD SEV-SNP)
- HSM (P06) — signing
- Audit (P05)
- BI (P10)
- Existing IEBC ABIS (I05, if retained)

## 6. Failure Modes

| Failure | Response |
|---|---|
| Enclave attestation fails | Worker quarantined; matching redistributed to healthy workers |
| Match latency exceeds SLO | Auto-scale; alert |
| Adjudication backlog | Additional investigator capacity called in |
| False-match rate drift | Weekly review with independent certifier |

## 7. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M03-01 | 1:N against 30M synthetic records p99 ≤ 8 s | Bench |
| A-M03-02 | FAR ≤ 0.01% + FRR ≤ 1% on IEBC test set | Independent certifier |
| A-M03-03 | Confidential compute attestation verified | Live attestation |
| A-M03-04 | Adjudication workflow closes cases within 24 h SLA | UAT |
| A-M03-05 | Standards compliance package accepted by independent certifier | Certifier report |

## 8. Cross-References

- `M02-bvr-backend.md`
- `../SECURITY.md` §3.8, §3.9
- `../DATA-MODEL.md` §1.2
