# governance-svc capabilities and algorithms

Comprehensive capability catalogue for the governance and legal compliance service. Every capability lists the algorithm, the source reference (paper, standard, statute, or formal method), and the crate or file that implements it.

## Contents

1. Module scope
2. Risk-limiting audit (M21)
3. Election court interface (M40)
4. Post-election review (M41)
5. Recall petition verification (M48)
6. Referendum mechanics and TLA+ verification (M49)
7. By-election short-notice detection (M50)
8. County coordination (M56)
9. International observer coordination (M57)
10. Parliamentary and judiciary reporting (M58)
11. Auditor-General interface (M59)
12. Transmission legal basis audit (M62)
13. Election financial forensics (M66)
14. Operational excellence
15. Standards and compliance
16. Data lifecycle, API surface, and events

---

## 1. Module scope

| Module | Name | Crate | Port |
|---|---|---|---|
| M21 | Risk-Limiting Audit | rla | 8091 |
| M40 | Election Court IT Interface | court-interface | 8092 |
| M41 | Post-Election Review | review | (shared with M40) |
| M48 | Recall Petitions | recall | (shared gateway) |
| M49 | Referendum Mechanics | referendum | (shared gateway) |
| M50 | By-Election Operations | by-election | (shared gateway) |
| M56 | County-National Coordination | county-coord | (shared gateway) |
| M57 | International Observer Coordination | intl-observer | (shared gateway) |
| M58 | Parliamentary and Judiciary Reporting | reporting | 8094 |
| M59 | Auditor-General Interface | oag | 8095 |
| M62 | Transmission Legal Basis Audit | legal-audit | 8093 |
| M66 | Election Financial Forensics | forensics | (shared gateway) |

Tier: T2. Evidence bundle delivery SLO: < 60 seconds. RLA sampling SLO: < 10 minutes per constituency. RTO/RPO: 15 minutes (Kafka consumer resumption from committed offset + snapshot replay).

Architecture: five axum microservices on separate ports, one Django admin (`governance-admin`, port 8096), one Tauri offline audit desktop (RLA), one Python SHANGRLA statistics engine. All writes append-only; audit shadow via PG trigger. Evidence bundles signed and pinned to IPFS via transparency-svc.

---

## 2. Risk-limiting audit (M21)

Crate: `crates/rla`. Python engine: `py/shangrla/`. Tauri desktop: `apps/rla-desktop/`.

### 2.1 SHANGRLA method

The audit implements SHANGRLA (Sets of Half-Average Nulls Generate Risk-Limiting Audits), which unifies ballot-polling, comparison, and batch-comparison audits under a single mathematical framework.

Reference: Stark, P.B. "Sets of Half-Average Nulls Generate Risk-Limiting Audits: SHANGRLA", in "Multistep and Multiobjective Voting", CRC Press, 2020. Also: Stark, P.B. and Teague, V. "Verifiable European Elections: Risk-limiting Audits for D'Hondt and Its Relatives", JETS vol 1, 2014.

The audit uses a risk limit `alpha` stored in `rla_audit_runs.risk_limit` (PG CHECK `> 0 AND < 1`). Three audit methods are supported (`ballot_polling`, `comparison`, `batch_comparison`) per the `audit_method` column.

### 2.2 Entropy seed

Seed for the pseudo-random ballot sampler is produced from three combined sources (table `rla_entropy_sources`, column `source_type IN ('dice','beacon','hardware','iab_card')`):

- NIST Randomness Beacon (beacon.nist.gov): 512-bit SHA-512 pulse retrieved at run initiation and pinned to `rla_entropy_sources.value_hash`.
- HSM hardware entropy (`source_type: hardware`): 256-bit sample from the on-site HSM.
- IAB (Independent Audit Board) card (`source_type: iab_card`): 128-bit value read from the physical IAB card at the witness event.

Combined seed: `blake3( beacon_hash || hsm_sample || iab_card_value || run_id )`. All witness signatures stored in `rla_witness_records`.

### 2.3 Sample drawing and verdict recording

Ballots are drawn by the Tauri offline desktop application (`rla-desktop`), which operates without network connectivity. Sample records (`rla_sampled_ballots`) accumulate locally (SQLite mirror) with `synced: false`. On reconnection the desktop syncs records to the central PostgreSQL via the `POST /v1/governance/rla/runs/{run_id}/samples` endpoint.

### 2.4 T-statistic auto-recount trigger

The Python engine (`py/shangrla/t_statistic.py`) computes the SHANGRLA T-statistic after each batch of verdicts. When the T-statistic exceeds the configured `risk_limit` threshold, the run status transitions to `escalated` and a `governance.rla.recount.triggered` event is emitted. The conclusion is recorded in `rla_audit_reports.conclusion` as `recount_triggered`.

### 2.5 Ballot manifest

`rla_audit_manifests` stores `station_count`, `total_ballots`, and `manifest_hash` (SHA-256 of the ballot manifest CSV). Manifest integrity is verified before each sample draw.

---

## 3. Election court interface (M40)

Crate: `crates/court-interface`. Tables: `court_petitions`, `court_evidence_bundles`, `court_technical_witnesses`, `court_supplementary_requests` (V2 migration).

### 3.1 Petition lifecycle

FSM: `filed -> served -> hearing -> judgment -> closed`. State transitions are appended to `court_petitions` with `updated_at`; no in-place mutations.

### 3.2 Evidence bundle composition

`POST /v1/governance/court/petitions/{id}/evidence-bundles` accepts up to 15 evidence items per bundle (JSONB array in `court_evidence_bundles.items`). Each item is a typed reference: `{ item_type, source_ref, hash, produced_at }`. The bundle is Ed25519-signed (HSM key role `court-evidence`) and pinned to transparency-svc IPFS.

### 3.3 eCourt integration

Bundles are delivered to the eCourt API (`ecourt_ref` field). If the eCourt API is unreachable, bundles are buffered locally and replayed on reconnection (circuit breaker in `court-interface/src/ecourt_client.rs`).

### 3.4 Technical witness pre-registration

`court_technical_witnesses` allows system experts to be pre-registered (`pre_registered_at`) and linked to petitions before testimony dates.

---

## 4. Post-election review (M41)

Crate: `crates/review`.

The post-election review module aggregates findings from RLA runs, court petition outcomes, and field incident reports into structured review documents. Findings are versioned: each revision appends a new entry to `review_finding_versions`; no in-place updates. Approved findings can be escalated to parliamentary referral via `POST /v1/governance/review/findings/{id}/refer`. Referral events are consumed by M58 (reporting).

---

## 5. Recall petition verification (M48)

Crate: `crates/recall`.

### 5.1 Article 104 thresholds (Constitution of Kenya 2010)

Two statutory thresholds apply:
- Art. 104(2): A recall petition must be signed by at least 30 % of the registered voters in the affected constituency.
- Art. 104(3): A recall referendum requires a YES vote by a simple majority of voters who cast valid votes.

Both thresholds are computed using integer-only arithmetic (`registered_count * 30 / 100` rounded up, using integer ceiling division) to avoid floating-point rounding errors. Implementation: `recall/src/thresholds.rs`.

### 5.2 Signatory de-duplication and OCR verification

Physical petition forms are scanned and OCR-processed (`py/recall/ocr_verifier.py`) using Tesseract OCR (Smith, "An Overview of the Tesseract OCR Engine", ICDAR 2007). Each extracted signature entry is matched against the registered voter roll (`bvr-svc` extract) by national ID and name. Duplicate signatures (same national ID appearing more than once) are rejected. Unmatched entries are flagged for manual review.

The de-duplication check is: `SELECT COUNT(DISTINCT voter_id) FROM recall_signatures WHERE petition_id = $1` after bulk insert; any row with `voter_id` appearing more than once sets `duplicate_flag: true`.

### 5.3 Petition outcome

When the 30 % threshold is met by verified, de-duplicated signatures, the petition status transitions to `verified` and a `governance.recall.petition.verified` event is emitted. The by-election module (M50) is notified to prepare a special election.

---

## 6. Referendum mechanics and TLA+ verification (M49)

Crate: `crates/referendum`.

### 6.1 24-of-47 county threshold

Kenya's Constitution Art. 255(2) requires a referendum question to pass in at least 24 of the 47 counties. County results are submitted via `POST /v1/governance/referendum/{id}/county-results`. The engine counts `counties_passed` as the number of county results where `yes_votes / (yes_votes + no_votes) > 0.5` (integer arithmetic, no floats). The national result is certified only when `counties_passed >= 24` AND the overall national majority is met.

### 6.2 TLA+ model and invariants

The county-threshold logic is specified in `docs/tla/Referendum.tla`. The model defines three invariants verified by TLC model checker:

1. `TypeInvariant`: all county results have non-negative integer votes and `county_id` is drawn from the fixed set of 47 codes.
2. `ThresholdMonotonicity`: once `counties_passed` reaches 24, no subsequent result submission can reduce it below 24.
3. `CertificationOnce`: the `certified` flag is set at most once per referendum instance.

### 6.3 Tie-breaker

If a county result is exactly 50/50 (yes_votes == no_votes), the county is counted as NOT passed (integer arithmetic; no fractional majority). A national-level exact tie (total yes == total no) is escalated to the commission for public announcement per the commission's tie-breaker procedure.

---

## 7. By-election short-notice detection (M50)

Crate: `crates/by-election`.

### 7.1 21-day notice constraint

Kenya's Elections Act §15 requires at least 21 days' notice before a by-election. The module detects when a vacancy event (from M48 recall, M41 court petition outcome, or manual declaration) results in a by-election date within 21 days of the vacancy date:

```
short_notice = (election_date - vacancy_date).num_days() < 21
```

When `short_notice: true`, a `governance.special.election.declared` event carries the `short_notice: true` flag and an alert is raised to the commission operations team.

### 7.2 Cost tracking

`eel_special_election_cost_reports` tracks per-line expenditure for the by-election. Reports are submitted to M59 (OAG interface) via `POST /v1/governance/special-elections/{id}/cost-report`.

---

## 8. County coordination (M56)

Crate: `crates/county-coord`.

County officials access the coordination workspace via SAML 2.0 federated authentication against county IdP identity providers. SAML attributes (`county_code`, `officer_role`) are validated on every request; JWT sessions are issued for subsequent calls. The circuit breaker to read-only mode fires when the SAML metadata endpoint is unavailable: cached county roles (from the last successful JWT) are used for up to 4 hours.

MoU records (`eel_county_mou`) track the signed MoU between each county and the national commission. MoU approval unlocks `POST` endpoints for county coordination events.

---

## 9. International observer coordination (M57)

Crate: `crates/intl-observer`.

Each accredited observer mission is given an isolated workspace (`mission_id`). Briefing packs are composed from: election calendar data (boundary-svc), approved observer deployment zones (crms-svc), and sanitised field reports (field-mobile-svc). Joint statements are version-controlled with a draft / finalised / published FSM. Published statements are signed and anchored to transparency-svc.

Observer mission data is classified `PII_RESTRICTED` for individual observer contact details; aggregate mission reports are `PII_PUBLIC`. Compliance with UN General Assembly Resolution A/RES/68/163 on electoral observation (United Nations, 2013) is addressed by ensuring observers receive timely access to the full briefing pack at least 48 hours before election day.

---

## 10. Parliamentary and judiciary reporting (M58)

Crate: `crates/reporting`. Tables: `reporting_statutory_reports`, `reporting_testimony_workspaces`, `reporting_judiciary_handovers`, `reporting_sc_preregistrations` (V4 migration).

### 10.1 Report types

Reports are typed (`Assembly`, `Senate`, `Judiciary`, `SC`, `Committee`) per `reporting_statutory_reports.report_type`. The generation pipeline (`reporting/src/generator.rs`) assembles sections from the data warehouse and renders a DOCX draft. The draft is versioned in `reporting_testimony_workspaces` and signed off before delivery.

### 10.2 Transmission rules engine

The rules engine in `reporting/src/rules.rs` evaluates per-event verdicts and produces a party summary. Each rule references a section of the Elections Act. Verdicts are stored in `legal_compliance_verdicts` (M62 migration V3).

### 10.3 Judiciary handover

Signed reports are delivered to the Judiciary system via `POST /v1/governance/parliament-reporting/{report_id}/handover`. The `reporting_judiciary_handovers` table records `handed_to`, `handover_ref`, and `signed_at`.

---

## 11. Auditor-General interface (M59)

Crate: `crates/oag`. Tables: `oag_engagements`, `oag_evidence_extracts`, `oag_responses`, `oag_published_reports` (V4 migration).

Evidence is exported to OAG via IFMIS/PPRA-compatible extract. The OAuth2 mTLS client (`oag/src/ifmis_client.rs`) uses a PKCS#11 client certificate from the HSM (`/dev/pkcs11/oag-cert`). Evidence extracts (`oag_evidence_extracts`) record `query_hash`, `row_count`, and `output_url` for audit traceability. OAG engagement scope transitions: `scoping -> active -> responded -> closed`.

---

## 12. Transmission legal basis audit (M62)

Crate: `crates/legal-audit`. Tables: `legal_transmission_records`, `legal_compliance_verdicts`, `legal_rules` (V3 migration).

### 12.1 Rules engine

Each transmission event (form34A upload, results declaration, tally update) is evaluated against the active rule set in `legal_rules` (columns: `rule_id`, `section_ref`, `description`, `applies_to`, `active`). Rules reference Elections Act §39(1C) and related regulations. The evaluation produces one `legal_compliance_verdicts` row per rule per transmission event (`passed: bool`, `failure_reason`, `tlog_hash`).

### 12.2 Transparency log hash

Every compliance verdict carries a `tlog_hash`: the Sigstore Rekor transparency log entry hash for the transmission event. This anchors the verdict to an immutable external record.

### 12.3 Real-time verdict stream

Verdicts are published as `governance.transmission.verdict.issued` CloudEvents and consumed by the LiveView dashboard (`ex/`) for real-time legal compliance monitoring during the transmission window.

---

## 13. Election financial forensics (M66)

Crate: `crates/forensics`. Python models: `py/forensics/`.

### 13.1 Anomaly detection methods

Three causal inference and econometric methods are implemented in `py/forensics/models.py` using pandas and scipy:

| Method | Abbreviation | Reference |
|---|---|---|
| Regression Discontinuity | RD | Imbens, G. and Lemieux, T. "Regression Discontinuity Designs: A Guide to Practice", Journal of Econometrics, 2008 |
| Instrumental Variables | IV | Angrist, J. and Pischke, J.-S. "Mostly Harmless Econometrics", Princeton University Press, 2009 |
| Difference-in-Differences | DiD | Card, D. and Krueger, A.B. "Minimum Wages and Employment", American Economic Review, 1994 |

All three methods operate on aggregated cycle-level spending data (from `eel_analysis_cycles`). No individual voter data is used as input.

### 13.2 CBK and FRC integration

Campaign finance data is pulled from Central Bank of Kenya (CBK) and Financial Reporting Centre (FRC) APIs via OAuth2 mTLS clients. Anomalies above a configurable score threshold are submitted as referrals via `POST /v1/governance/forensics/referrals` and stored in `eel_forensics_referrals`.

### 13.3 Anomaly scoring

The ML anomaly score is a composite of: RD discontinuity magnitude, IV Wald estimator deviation from the national baseline, and DiD parallel-trend violation score. Scores are normalised to [0, 1]. Scores > 0.7 trigger an automated `governance.forensics.anomaly.detected` event.

---

## 14. Operational excellence

| Capability | Path | Mechanism |
|---|---|---|
| RLA offline desktop | apps/rla-desktop | Tauri app; SQLite local storage; sync-on-reconnect |
| RLA beacon fallback | rla/src/entropy.rs | Cached seed from prior election if NIST API times out |
| Court bundle local queue | court-interface/src/ecourt_client.rs | Buffer to MinIO on eCourt unreachable; replay on reconnect |
| SAML metadata circuit breaker | county-coord/src/saml.rs | Cached JWT roles for 4 h when IdP unavailable |
| Kafka consumer lag alert | Prometheus | Lag > 10 k messages fires warning |
| Forensics detector threshold | ConfigMap | Adjustable in real-time without redeploy |
| RLA sample SLO | Grafana | 99.9 % of samples drawn within 10 min |
| Evidence bundle SLO | Grafana | 99.9 % of bundles delivered within 60 s |
| Recall verification SLO | Grafana | 99.5 % of petition verifications within 5 min |

Key metrics: `governance_rla_sample_size{jurisdiction}`, `governance_rla_verdict_latency_ms{outcome}`, `governance_court_bundle_delivery_ms`, `governance_forensics_anomaly_score{type}`.

---

## 15. Standards and compliance

| Standard / statute | Coverage |
|---|---|
| SHANGRLA (Stark 2020, CRC Press) | Risk-limiting audit statistical method |
| NIST SP 800-90B (2018) | Randomness beacon entropy quality |
| Constitution of Kenya 2010 Art. 87, 104, 255 | Election management, recall, referendum |
| Elections Act Cap 7 §§15, 39(1C), 75-84 | Transmission legality, court evidence, by-election notice |
| Election Offences Act | Chain-of-custody evidence handling |
| Political Parties Act Cap 7A | Recall petition party eligibility |
| UN General Assembly Resolution A/RES/68/163 (2013) | Electoral observation standards |
| ISO/IEC 27001:2022 | Information security management for evidence bundles |
| TLA+ (Lamport, "Specifying Systems", Addison-Wesley, 2002) | Formal verification of referendum county-threshold logic |
| Sigstore Rekor transparency log | Compliance verdict immutable anchoring |
| CloudEvents 1.0 (CNCF, 2022) | Event envelope |

---

## 16. Data lifecycle, API surface, and events

### 16.1 Data lifecycle

| Table | Retention |
|---|---|
| rla_audit_runs, rla_sampled_ballots | Election cycle + 15 years (KNADS Act) |
| court_petitions, court_evidence_bundles | Permanent (court record) |
| legal_compliance_verdicts | Election cycle + 15 years |
| oag_engagements, oag_evidence_extracts | 7 years (Public Audit Act) |
| reporting_statutory_reports | Permanent (parliamentary record) |
| recall_signatures | Election cycle + 10 years |
| forensics anomaly records | 10 years |

All tables with audit events are append-only (PG trigger blocks UPDATE/DELETE). PII fields (petitioner hash, officer hash) are stored as SHA-256 hashes; cleartext is never persisted.

### 16.2 API surface (selected endpoints)

- `POST /v1/governance/rla/runs` + `POST /v1/governance/rla/runs/{run_id}/samples` + `GET /v1/governance/rla/runs/{run_id}/report`
- `POST /v1/governance/court/petitions` + `POST /v1/governance/court/petitions/{id}/evidence-bundles`
- `POST /v1/governance/recall/petitions` + `GET /v1/governance/recall/petitions/{id}`
- `POST /v1/governance/referendum/{id}/county-results` + `GET /v1/governance/referendum/{id}/result`
- `POST /v1/governance/special-elections/{id}/cost-report`
- `GET /v1/governance/legal-audit/transmissions/{id}/verdict`
- `GET /v1/governance/forensics/cycles/{cycle_id}/anomalies`
- `POST /v1/governance/oag/audit-engagements`
- `GET /v1/governance/parliament-reporting/{report_id}/draft`

All endpoints require JWT + SpiceDB authz. Evidence bundle downloads stream multipart/form-data with 1 MB chunks and resumable offset header.

### 16.3 Events

**Emitted:**

| Event type | Trigger |
|---|---|
| `governance.rla.completed` | Audit run concludes (pass or escalate) |
| `governance.rla.recount.triggered` | T-statistic exceeds risk limit |
| `governance.petition.bundle.delivered` | Evidence bundle delivered to eCourt |
| `governance.recall.petition.verified` | 30 % signatory threshold met |
| `governance.referendum.result.certified` | 24-of-47 county threshold met |
| `governance.special.election.declared` | By-election declared (with short_notice flag) |
| `governance.transmission.verdict.issued` | Compliance verdict for a transmission event |
| `governance.forensics.anomaly.detected` | Anomaly score > 0.7 |

**Consumed:**

| Event type | Source | Purpose |
|---|---|---|
| `rts.form34.published` | rts-svc | RLA sampling baseline; referendum tally |
| `rts.declaration.signed` | rts-svc | Court evidence and legal audit |
| `transparency.sth.published` | transparency-svc | Evidence bundle archival anchoring |
| `field-mobile.attendance.checked-in` | field-mobile-svc | By-election staff cross-verify |
| `crms.party.registered` | crms-svc | Recall petition party eligibility |

---

## Summary

Total footprint: 12 Rust crates across 5 axum microservices, 1 Tauri offline desktop (RLA), 1 Django admin, 1 Python SHANGRLA + forensics engine, 4 SQL migrations (V1-V4, 20+ tables), Kafka outbox. Core correctness guarantees: SHANGRLA-verified RLA with NIST beacon + HSM + IAB card entropy, TLA+-verified referendum county-threshold logic, integer-only recall threshold arithmetic, idempotent court evidence bundles with transparency-log anchoring.

---

## Appendix A: RLA Tauri offline desktop flow

The `apps/rla-desktop` Tauri application implements the following offline workflow:

```
1. Commission officer loads RLA run parameters (run_id, risk_limit, audit_method)
2. Desktop fetches manifest hash from server and verifies ballot manifest integrity
3. Desktop derives sample using SHANGRLA sampler seeded from blake3(beacon || hsm || iab_card)
4. Auditor physically locates each sampled ballot and records interpretation offline
5. Desktop computes T-statistic after each batch; displays current risk level
6. On network reconnect: POST /v1/governance/rla/runs/{id}/samples with all pending records
7. Server merges sync, updates run status, and re-evaluates T-statistic server-side
```

The local SQLite database on the desktop encrypts `auditor_interpretation` fields with a session key derived from the officer's HSM-backed credential. This prevents offline tampering of verdict records between the field and the sync step.

---

## Appendix B: Degradation contract

| Mode | Behaviour |
|---|---|
| Normal | All workflows live: NIST beacon, real-time court interface, live anomaly detection |
| Degraded | RLA from cached beacon seed (prior election); court bundles queued locally; forensics batch-only |
| Read-only | Pre-computed RLA reports from archive; court returns cached bundles; all writes rejected with 503 |

---

## Appendix C: Security controls

| Control | Mechanism |
|---|---|
| Evidence bundle integrity | Ed25519 HSM signature + Sigstore Rekor tlog pin |
| RLA offline tampering | Session-key-encrypted SQLite; server-side re-verify on sync |
| OAG mTLS | PKCS#11 client cert from HSM (/dev/pkcs11/oag-cert); OAuth2 token |
| County IdP federation | SAML 2.0 with circuit breaker to read-only on IdP outage |
| Audit trail immutability | Append-only PG tables; UPDATE/DELETE blocked by trigger |
