# Product Requirements Document — the Commission Integrated Elections Management System

**Version:** 3.0
**Date:** 2026-08-21
**Owner:** Datacraft Limited (software)
**Consumer:** Independent Electoral & Boundaries Commission of Kenya
**Contract reference:** the tender specification, 3-year Framework Contract
**IPR:** Custom Software vests in the Commission per GCC 15.4; Standard Software licensed perpetually + irrevocably to the Commission per GCC 16

---

## 1. Executive Summary

The Integrated Elections Management System is Kenya's sovereign end-to-end election technology stack. It runs the biometric voter registration for approximately 30 million voters, provides the on-device polling-day voter identification at approximately 46,000 polling stations, transmits and tallies results from up to 60,000 concurrent sessions across six elective positions plus referenda, exposes public transparency portals, manages candidate nominations, and orchestrates a fleet of up to 65,287 KIEMS biometric kits over a three-year contract that spans the 2027 general election.

Every design choice in this PRD is dominated by two failure conditions the system must never enter: **(i) results tampered without detection** and **(ii) system-wide unavailability during a polling window**. Every requirement is priced against those two conditions.

The system is built around ten principles: **sovereign by design** (data residency in Kenya), **cryptographic transparency** (Merkle-logged, publicly verifiable results), **offline-first** (poll clerks operate for hours without connectivity), **fail-visibly** (never silent failure), **two-person integrity** (no single operator can move critical state), **defence in depth** (compromise of any layer does not cascade), **standards compliance** (FBI, ICAO, ISO, ANSI/NIST, Kenya DPA), **portability** (no vendor lock-in), **auditability** (every state change is signed and immutable), and **accessibility** (WCAG 2.2 AA plus tactile / audio channels for disabled voters).

---

## 2. Scope Evolution — v1 to v2

Version 1.0 (2026-08-18) covered 12 core IEMS modules (M01-M12) and 32 platform layers, integrations, operational modules, and compliance instruments. Version 2.0 extended to 28 modules. Version 3.0 extends to the full 69-module scope based on the complete module inventory in `sw/modules/`.

Source strategy documents:

- **CLIENT-EXCELLENCE-STRATEGY.md** (Parts I-XXI, 2,692 lines) — election-proofing architecture, public confidence programme, reproducibility commitment, fraud prevention, ballot chain of custody, queue transparency, boundary integrity, speech integrity, rules compliance, anomaly detection, and gap-closure register
- **INVENTORY-MANAGEMENT.md** — full IMS design including identifier architecture, 8 custody workflows, RFID infrastructure, issue management, and post-petition disposition
- **HARDWARE-REQUIREMENTS-FROM-MITIGATIONS.md** — hardware specifications traceable to mitigation commitments

**Progressive scope extension:**

| Category | v1 | v2 | v3 |
|---|---|---|---|
| Core application modules | M01-M12 | M01-M28 | M01-M69 (41 new modules) |
| Mobile field apps | None | None | M29-M34 (6 mobile apps) |
| Register maintenance | M02 only | M02 only | M02 + M35 + M36 + M53 + M64 |
| Party and candidate | M12 only | M12 only | M12 + M45 + M46 + M47 + M67 |
| Special elections | Not covered | M24-M25 | + M48 Recall + M49 Referendum + M50 By-election |
| Transparency | M14, M15 | M14, M15 | + M57 + M58 + M59 + M66 |
| Field operations | M16, M19 | M16, M19 | + M31-M34 + M55 + M56 + M60 |
| Compliance and audit | M21 | M21 | + M40 + M61 + M62 + M63 |
| Long-term archive | Not covered | Not covered | M69 (15-year archive) |
| Kit fleet scale | 60,000 KIEMS kits | 65,287 kits + 14,100 legacy | Same; +6 mobile apps for officers |
| Accessibility standard | WCAG 2.1 AA | WCAG 2.2 AA | WCAG 2.2 AAA for referendum content |
| Languages | EN + SW | EN + SW + 9 regional | Same; referendum content via community pipeline |

---

## 3. Vision & Success Criteria

### 3.1 Vision statement

*A national elections system whose results any Kenyan citizen can verify independently on their phone, whose hardware never fails a polling clerk in the field, and whose operator can defend every result in court with cryptographic evidence.*

### 3.2 Success criteria (measurable)

| Dimension | Target |
|---|---|
| Voter register coverage | 100% of enrolled voters captured with all mandated biometric modalities |
| Register accuracy | ABIS false-match rate <= 0.01% at operational threshold |
| Polling-day availability | >= 99.99% during the polling window; no kit downtime > 15 minutes |
| RTS throughput | 60,000 concurrent sessions with p99 authorisation latency <= 100 ms |
| Results publication cycle | First provisional publication <= 1 hour after last polling station closes |
| Public verification | Any citizen can retrieve a Merkle-inclusion proof for any polling station within 30 seconds |
| Recount from paper | Any specific Form 34A can be reproduced deterministically from the signed image in <= 5 minutes |
| Reproducibility | Any graduate can re-run the full national tally from published inputs in <= 2 hours on a 16 GB laptop |
| Security incidents (P1) | Zero undetected P1 security incidents during the contract |
| Data-protection incidents | Zero unauthorised bulk exports of the voter register |
| Court petitions | Every petition receives a complete cryptographically-proven evidence pack within 24 hours of request |
| Missing kits at T+30 | 0 (target) |

### 3.3 Non-goals

- Voter education beyond what is specified in M13 and M17 digital channels.
- Financial payment rails (kit-workers paid via Commission HR/payroll, not this system).
- Ballot printing itself (CRMS generates ballot proofs; printing contracted separately via Tender 02).
- Criminal prosecution (IMS, M17, and M18 produce evidence packages; prosecution is ODPP's function).

---

## 4. Stakeholders & Personas

### 4.1 Primary user personas

| Persona | Volume | Access channel | Key tasks |
|---|---|---|---|
| Enrolment Clerk | 20K+ | KIEMS device | Capture biographic + biometric during registration windows |
| Presiding Officer (PO) | 46K | KIEMS device + tablet | Open station, authenticate voters, close station, transmit 34A |
| Deputy PO / Clerk | ~92K | KIEMS device | Assist under PO authority |
| Returning Officer (RO) | 290 (constituencies) | Constituency workstation | Tally, sign 34B, transmit to national |
| County Returning Officer (CRO) | 47 | County workstation | Aggregate 34C, publish |
| Party Agent | ~2K per polling station peak | Read-only + verification | Observe, verify, request inclusion proofs |
| Registered Voter | 30M | Web / mobile / kiosk / KIEMS | Self-register (VPRS), verify register (BVV), verify results (M13) |
| Party Official | ~50 parties | Web portal | Nominate candidates (CRMS) |
| Candidate | ~15K per general election | Web portal | Submit nomination + attend clearance |
| the Commission Commissioner | 7 | Executive dashboard | Oversight, publication authorisation, s.14A declarations |
| the Commission Secretariat | ~200 | Admin console | Operational management |
| the Commission ICT Admin | ~30 | Privileged bastion | Infrastructure, configuration, deployment |
| the Commission County ICT Officer | 47 | County console | County-level operations |
| Field Support Engineer (Datacraft) | ~200 election-day | Support console + phone | Level 1-3 troubleshooting + kit dispatch |
| IAB Member | 7-10 | M15 IAB Dashboard | Continuous assurance, quarterly reports, escalation |
| Auditor (independent) | ~5-20 | Read-only + escrow mirror | Continuous certification, post-election audit |
| Court / Petitioner | ad hoc | Evidence-pack request | Retrieve inclusion proofs, signed forms, audit trails |
| ODPC officer | ~2 | Compliance console | Data-protection oversight |
| NCIC Liaison | 1 | M17 referral portal | Hate-speech evidence package receipt |
| International Observer | ~200 (EU EOM, Carter Center, ELOG, IFES) | Sanitised SOC feed | Independent real-time monitoring |
| Data Journalist | ~50 | Public API / M13 dataset | Statistical analysis, verification reporting |

### 4.2 Non-human personas (systems)

- **IPRS** — Integrated Population Registration System.
- **KRA** — tax-status queries on candidates.
- **Telco carriers** — Safaricom, Airtel, Telkom Kenya; VPN endpoints, SMS gateways, eSIM.
- **HSM appliance** — signing services.
- **the Commission BI (Power BI)** — downstream analytics.
- **Mechrevo MES** — OEM factory system providing birth records and FAT data.
- **Government Printer** — Kenya Gazette Merkle root publication.
- **KBC Radio** — daily hash bulletin.
- **Transparency-log verifiers** — external observers polling the public log.

---

## 5. Scope

### 5.1 In scope (Datacraft builds)

- **69 application modules** (M01-M69): see §8 for full list.
- **10 platform layers** (P01-P10): see §8.2.
- **10 integrations** (I01-I10): see §8.3.
- **6 operational subsystems** (O01-O06): see §8.4.
- **4 compliance instruments** (C01-C04): see §8.5.

### 5.2 Explicitly out of scope

- KIEMS device manufacturing (Mechrevo + OEM).
- Data-centre server, storage, networking hardware (Commission-provided).
- Telco VPN infrastructure (Commission-provided).
- The ~14,100 legacy Android devices themselves (Commission-provided; must integrate).
- HSM appliance hardware (procured from Thales/Utimaco/Entrust; Datacraft integrates).
- Physical printing of ballots (post-CRMS handoff).
- Criminal prosecution (Datacraft produces evidence; ODPP prosecutes).

### 5.3 Boundary interfaces

- **Northbound to the Commission**: administrative consoles, executive dashboards, audit exports.
- **Eastbound to public**: VPRS, BVV, Public Results Portal, M13 Public Verifier, M18 Boundary Explorer.
- **Southbound to devices**: KIEMS kits, tallying tablets, county workstations, field engineer mobile.
- **Westbound to external systems**: IPRS, KRA, telcos, HSM, Power BI, transparency log, NCIC, DCI, ODPP, Auditor-General, Supreme Court IT Registrar.

---

## 6. Regulatory & Legal Context

Every requirement in this PRD complies with:

| Framework | Reference |
|---|---|
| Elections Act 2011 (as amended 2016) | §§ 6A, 14A, 30, 39(1C), 44, 44A, 53-54, 55, 69, 72, 78, 80, 83-84 |
| Elections (Technology) Regulations 2017 | Full applicability |
| Election Offences Act 2016 | s.6-s.12 (bribery, impersonation, undue influence, false statements) |
| Election Campaign Financing Act 2013 | s.24 (spending disclosure); monitoring integration |
| Kenya Data Protection Act 2019 | Full applicability; ODPC oversight |
| National Cohesion and Integration Act 2008 | s.13, s.62 (hate speech referrals via M17) |
| Computer Misuse and Cybercrimes Act 2018 | s.22-s.23 (CIB / deepfake referrals via M17) |
| Public Procurement and Asset Disposal Act 2015 | Contract governance |
| Whistleblower Protection Act 2010 | Grievance channel legal-privilege preservation |
| ISO/IEC 27001:2022 | ISMS certification target |
| ANSI/NIST-ITL 1-2011 Part 1 + ITL 2-2008 XML | Biometric interchange |
| FBI IAFIS IQS + EBTS IAFIS-DOC-01078-9.0 | Fingerprint |
| FBI-certified WSQ (IC-0110) + JPEG-2000 (ISO/IEC 15444-1:2019) | Fingerprint compression |
| ISO/IEC 19794-5 + ICAO 9303 | Facial |
| ISO/IEC 19794-6 | Iris |
| WCAG 2.2 AA | Web / portal / app accessibility (upgraded from 2.1) |
| Constitution 2010 Articles 26, 33, 38, 54, 81-82, 86-89, 138-140 | Foundational election rights framework |

---

## 7. System Architecture (Summary)

Full detail in `ARCHITECTURE.md`. Highlights:

- **Two-site active/mirror** deployment on Commission-provided converged infrastructure (primary Anniversary Towers area, mirror at the Commission DR site). Synchronous replication for write-critical paths (register, results); asynchronous for BI marts.
- **Zero-trust internal network** with mTLS for every service-to-service call, short-lived tokens issued by a central authorisation service (OPA policy engine).
- **Per-kit digital certificates** issued by an internal PKI with air-gapped root CA, offline ceremony-signed with M-of-N quorum.
- **Merkle-log-backed immutability** for every state-change event, with public transparency-log root published every 10 minutes to five independent channels, and daily Kenya Gazette anchoring.
- **Offline-first on the device**: BVR / BVI / RTS device clients store, sign, and forward when connectivity is unavailable.
- **Static-first public read fabric**: Public Results Portal and M13 Public Verifier are CDN-cached signed snapshots so DDoS on origin does not stop citizens from verifying results.
- **Reproducibility-first pipeline**: every aggregation stage is a pure function in a pinned container; `make reproduce` on any laptop re-derives the national tally bit-identically.

---

## 8. Module Requirements

Full functional and non-functional detail per module lives in `modules/*.md`. Below is the master module list with one-line descriptions. Every module inherits all §9 non-functional requirements unless overridden.

### 8.1 Core IEMS Modules (M01-M12 — original scope)

| # | Module | Description | Doc |
|---|---|---|---|
| M01 | BVR Device Client | On-device biometric capture and registration workflow | `modules/M01-bvr-device.md` |
| M02 | BVR Back-End (VRCS) | Central voter registration system, ABIS interface, register management | `modules/M02-bvr-backend.md` |
| M03 | ABIS | 1:N biometric deduplication engine, 30M-record scale | `modules/M03-abis.md` |
| M04 | VPRS | Public self-pre-registration portal (greenfield) | `modules/M04-vprs.md` |
| M05 | BVI Device Client | On-device polling-day biometric identification | `modules/M05-bvi-device.md` |
| M06 | BVI Back-End | Biometric identification back-end, station-binding, geofence enforcement | `modules/M06-bvi-backend.md` |
| M07 | BVV | Biometric voter verification portal for register inspection | `modules/M07-bvv.md` |
| M08 | RTS Device Client | On-device result capture, Form 34A generation, transmission | `modules/M08-rts-device.md` |
| M09 | RTS RO / Constituency Tally | Returning officer aggregation module, Form 34B | `modules/M09-rts-ro.md` |
| M10 | RTS Back-End | Central results ingestion, validation, national aggregation | `modules/M10-rts-backend.md` |
| M11 | Public Results Portal | CDN-first public results publication, Merkle-proofed | `modules/M11-public-portal.md` |
| M12 | CRMS | Candidate registration, nomination vetting, ballot-proof generation | `modules/M12-crms.md` |

### 8.2 Citizen Integrity Modules (M13-M20 — v2 additions, this agent)

| # | Module | Description | Doc |
|---|---|---|---|
| M13 | Public Verifier App | Voter mobile/PWA + SMS/USSD + WhatsApp result verification, queue, safety, grievance | `modules/M13-public-verifier-app.md` |
| M14 | Reproducibility Bundle | Laptop-runnable `make reproduce`, hash-anchored pipeline, Gazette publication, participation tokens | `modules/M14-reproducibility-bundle.md` |
| M15 | IAB Dashboard | Independent Assurance Board read-only console, escalation ladder L1-L4, evidence packages | `modules/M15-iab-dashboard.md` |
| M16 | Election Command Centre | Unified E-SOC + COC + CAOC operator workstations, s.44A workflow, field dispatch | `modules/M16-election-command-centre.md` |
| M17 | Speech Integrity | Hate-speech + CIB + deepfake + voter-suppression detection, prebunking, evidence packaging | `modules/M17-speech-integrity.md` |
| M18 | Boundary Transparency | Boundary explorer + simulator + fairness metrics + redistricting audit trail + geofence service | `modules/M18-boundary-transparency.md` |
| M19 | Queue Transparency | Real-time queue map + station-alert workflow + ECC triggers + s.14A recommendations | `modules/M19-queue-transparency.md` |
| M20 | Inventory Management | IMS: RFID + GPS + Merkle chain of custody for 65,287 kits + 1.9M seals + consumables | `modules/M20-inventory-management.md` |

### 8.3 Advanced Integrity Modules (M21-M28 — parallel agent scope)

| # | Module | Description | Doc |
|---|---|---|---|
| M21 | RLA + SHANGRLA Audit Tool | Post-election risk-limiting audit with ballot-polling and comparison methods | `modules/M21-rla-audit.md` |
| M22 | Campaign Finance Transparency | ECFA spending disclosure portal, real-time monitoring, the Commission + Registrar reporting | `modules/M22-campaign-finance.md` |
| M23 | Grievance Channels | Multi-channel intake (app, SMS, WhatsApp, short-code), SLA tracking, evidence routing | `modules/M23-grievance-channels.md` |
| M24 | Diaspora Voting | Embassy-hosted KIEMS, sealed-transmission queue, diplomatic network fallback | `modules/M24-diaspora-voting.md` |
| M25 | Special Voter Workflows | Remand prisoners, hospital voters, KDF/GSU, IDP workflows | `modules/M25-special-voter-workflows.md` |
| M26 | Anomaly + Fraud Detection | Seven-layer detection stack: kit telemetry, cross-station stats, distributional analysis, cryptographic, RLA, cross-source, tips | `modules/M26-anomaly-detection.md` |
| M27 | Kit-Side Pre-Mark Validation | Ballot-image quality check before deposit, spoilt-ballot tracking, intent preservation | `modules/M27-premark-validation.md` |
| M28 | AI Monitoring Stack | On-device and central AI models: liveness, quality, OCR, anomaly, NLP translation, TTS | `modules/M28-ai-monitoring.md` |

### 8.4 Voter Identification + Verification — extended (M29)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M29 | Mobile Citizen Verify | Public-facing mobile app: register lookup, polling station finder, result verification via Merkle proof, civic info | `modules/M29-mobile-citizen-verify.md` | M11, M13, M64 |

### 8.5 Field Operations — Mobile Apps (M30-M34)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M30 | Mobile Party Agent | Accredited party-agent app: station authentication, Form 34A independent verification, cryptographic counter-signing, incident reporting | `modules/M30-mobile-party-agent.md` | M09, M10, M23 |
| M31 | Mobile Poll Clerk | Deputy PO / clerk companion: reference materials, incident logging, voter-lookup backup, paper-backup guidance, kiosk mode during inspection | `modules/M31-mobile-poll-clerk.md` | M05, M06, M19 |
| M32 | Mobile Field Support | Datacraft field-engineer console: support-ticket queue, remote diagnostics, spare-parts lookup, escalation, end-of-day reporting | `modules/M32-mobile-field-support.md` | M20, M16 |
| M33 | Mobile Personnel Dispatch | Worker dispatch and attendance for ~250K polling-day personnel: GPS check-in, assignment delivery, incident reporting, payroll trigger | `modules/M33-mobile-personnel-dispatch.md` | M16, M20, M55 |
| M34 | Mobile Observer | Accredited observer app: structured incident reporting, cryptographic result verification, observer findings feed | `modules/M34-mobile-observer.md` | M15, M43, M57 |

### 8.6 Voter Register Maintenance (M35-M36, M53, M64)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M35 | CVR — Continuous Voter Register | Inter-election continuous maintenance of the register: life-event ingestion from CRS/NRB, ward transfer workflow, accuracy metrics dashboard | `modules/M35-cvr-continuous-voter-register.md` | M02, M03, M36 |
| M36 | Voter Deregistration Appeal | Statutory appeal workflow for voters removed from the register: case file, hearing scheduling, adjudicator decision, court referral | `modules/M36-voter-deregistration-appeal.md` | M35, M02, M40 |
| M53 | Voter ID Card Production | ID-card order management: NRB integration, card-manufacturing partner API, QC scan, distribution tracking, replacement workflow | `modules/M53-voter-id-card.md` | M02, M35, M64 |
| M64 | Voter List Public Inspection | Statutory T-90 to T-60 public-inspection portal: per-station roll download (DPA-redacted), objection intake, objection resolution, auto-referral on deadline breach | `modules/M64-voter-list-public-inspection.md` | M02, M36, M40 |

### 8.7 Training and Education (M37-M38)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M37 | Voter Education CMS | Multi-format, multi-language voter education content management, distribution to all channels, campaign scheduling | `modules/M37-voter-education-cms.md` | M13, M11, M68 |
| M38 | Poll Worker Training LMS | Training, assessment, and certification LMS for ~290K poll workers; offline-capable tablet app; certification gating for election-day access | `modules/M38-poll-worker-training-lms.md` | M33, M16 |

### 8.8 Ballot Chain of Custody (M39, M63)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M39 | Ballot Printing Chain of Custody | Printer-to-warehouse custody tracking with RFID: print run records, quality-control scans, transfer manifests, warehouse arrival confirmation | `modules/M39-ballot-printing-chain.md` | M20, M63, M12 |
| M63 | Ballot Serial Chain | Warehouse-to-polling-station individual ballot tracking: serial-range assignment, spoilt/replaced/reconciled disposition, station-level reconciliation | `modules/M63-ballot-serial-chain.md` | M39, M20, M40 |

### 8.9 Detection and Audit (M40-M21, M61-M62, M66)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M40 | Election Court IT Interface | Evidence package system for election petitions: signed Form 34A/B/C retrieval, audit-log extraction, chain-of-custody export, Supreme Court IT Registrar integration | `modules/M40-election-court-it-interface.md` | M10, M14, M21, M63 |
| M41 | Post-Election Review | Lessons-learned and recommendation tracking platform: finding registry, responsible-party assignment, closure verification, cross-cycle trend analysis | `modules/M41-post-election-review.md` | M16, M21, M59 |
| M61 | Secrecy Assurance Testing | Automated ballot-secrecy assurance test harness: anonymity-set analysis, cross-module data-flow audits, DPA 2019 purpose-limitation verification | `modules/M61-secrecy-assurance-testing.md` | M02, M05, M10 |
| M62 | Transmission Legal-Basis Audit | Per-transmission lawful-basis audit log: Elections Act citation, DPA 2019 purpose code, actor, timestamp, data category — for every cross-system data movement | `modules/M62-transmission-legal-basis-audit.md` | M10, M06, M56 |
| M66 | Election Financial Forensics | Post-election published analytics for cost-per-vote conversion ratios by candidate and constituency; econometric methods; ODPP / FRC referral API | `modules/M66-election-financial-forensics.md` | M22, M45, M47 |

### 8.10 Accreditation and Transparency (M42-M43, M57-M59)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M42 | Media Accreditation Portal | Journalist and media-house accreditation: application intake, credential review, badge issuance, access-zone management, renewal | `modules/M42-media-accreditation-portal.md` | M16, M15 |
| M43 | Observer Accreditation Portal | Civil-society and international observer accreditation: organisation vetting, individual credential, station-assignment, reporting access | `modules/M43-observer-accreditation-portal.md` | M34, M15, M57 |
| M57 | International Observer Coordination | Multi-mission coordination portal for EU EOM, Carter Center, ELOG, IFES: mission briefing, data feeds, diplomatic communication log | `modules/M57-international-observer-coordination.md` | M43, M15, M34 |
| M58 | Parliamentary and Judiciary Reporting | Structured reporting interface for National Assembly, Senate, and Judiciary: elections-committee data packs, statutory returns, audit reports | `modules/M58-parliamentary-judiciary-reporting.md` | M10, M59, M51 |
| M59 | Auditor-General Interface | OAG-scoped read-only data portal: financial audit extracts, kit-disposition reports, DPA-compliant access logging, Auditor-General signed attestations | `modules/M59-auditor-general-interface.md` | M54, M20, M14 |

### 8.11 Analytics (M44)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M44 | Turnout Forecasting Analytics | Real-time and predictive turnout analytics: live BVI event aggregation, historical model, constituency heat-map, ECC feed, public dashboard | `modules/M44-turnout-forecasting-analytics.md` | M06, M16, M19 |

### 8.12 Candidate and Party (M45-M47, M67)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M45 | Party Registration Management | Registrar of Political Parties workflow: constitution submission, membership verification, office-bearer records, deregistration process | `modules/M45-party-registration-management.md` | M12, M46, M67 |
| M46 | Party Primaries Oversight | the Commission oversight of party primary elections: nomination schedule filing, results upload, dispute intake, gender-rule compliance check | `modules/M46-party-primaries-oversight.md` | M45, M12, M23 |
| M47 | Nomination Fee Payment | Candidate nomination fee collection: M-Pesa / bank-transfer integration, receipt generation, fee-waiver workflow for gender-rule candidates, refund tracking | `modules/M47-nomination-fee-payment.md` | M12, M45 |
| M67 | Party Symbol and Colour Registry | Trademark-style registry for party visual identity: Tanimoto similarity check for symbols, perceptual-hash deduplication, colour exclusivity per cycle, ballot-design export API | `modules/M67-party-symbol-colour-registry.md` | M45, M12 |

### 8.13 Special Contexts (M48-M50, M65, M68)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M48 | Recall Election Workflow | End-to-end workflow for constituency member recall: petition intake, signature verification, threshold calculation, the Commission gazette trigger, ballot generation | `modules/M48-recall-election-workflow.md` | M12, M35, M52 |
| M49 | Referendum Mechanics | KIEMS mode-switch and back-end configuration for constitutional referenda: question registry, YES/NO tally logic, constitutional-threshold computation, result declaration | `modules/M49-referendum-mechanics.md` | M08, M10, M11, M68 |
| M50 | By-Election Operations | Reduced-scope election operations for a single constituency: partial kit deployment, constrained nomination window, same cryptographic guarantees as general election | `modules/M50-by-election-operations.md` | M10, M12, M16 |
| M65 | Diaspora Ballot Physical Logistics | Physical return leg for diaspora ballots: tamper-evident container packing, customs pre-clearance, diplomatic-pouch coordination, chain of custody, tally-centre arrival | `modules/M65-diaspora-ballot-physical-logistics.md` | M24, M10, M20 |
| M68 | Referendum Explainer Platform | Multi-language, multi-format voter-facing explainers for each referendum question: constitutional context, Yes/No meaning, fact-check review, community translation pipeline | `modules/M68-referendum-explainer-platform.md` | M49, M37, M13 |

### 8.14 Support and Administration (M41, M51-M52, M54-M56, M60, M69)

| # | Module | Description | Doc | Related |
|---|---|---|---|---|
| M51 | Result Declaration and Certification | Formal result declaration workflow from constituency to national: Form 34-38 certification pipeline, Commissioner digital signature, gazette-ready export, Supreme Court notification | `modules/M51-result-declaration-certification.md` | M10, M09, M40 |
| M52 | Election Calendar and Gazette Notice | Authoritative election-calendar store: gazetted dates, notice generation, Government Printer integration, cross-module date-authority API | `modules/M52-election-calendar-gazette.md` | M35, M48, M12 |
| M54 | Election Budget and Expenditure Portal | the Commission budget tracking and public expenditure transparency: budget line management, expenditure upload, public dashboard, OAG export | `modules/M54-election-budget-expenditure.md` | M59, M54 |
| M55 | HR and Payroll for Temporary Election Staff | 290K temporary-staff lifecycle: engagement workflow, M-Pesa bulk-disbursement, tax withholding, dispute resolution, NHIF/NSSF integration | `modules/M55-hr-payroll-temporary-staff.md` | M33, M38 |
| M56 | County and National Government Coordination | Secure data-sharing interface with county governments, Interior Ministry, and security agencies: pre-election logistical data, polling-day situation feeds, DPA-lawful-basis logging | `modules/M56-county-national-coordination.md` | M16, M18, M62 |
| M60 | Medical and First-Response Coordination | Polling-day medical and security incident coordination: incident report from M33 feed, ambulance dispatch integration, hospital status, county health authority portal | `modules/M60-medical-first-response-coord.md` | M33, M16 |
| M69 | Election Data Long-Term Archive | 15-year retention archive: transition to preservation formats, KNADS integration, format-migration pipelines, signed preservation attestations, researcher-access API | `modules/M69-election-data-long-term-archive.md` | M14, M10, M59 |

### 8.4 Platform Layers

| # | Layer | Purpose |
|---|---|---|
| P01 | Geo-fencing / GNSS enforcement | Polygon store per station; kit-side and back-end verification (also served by M18) |
| P02 | eSIM / SIM central management | Provisioning, revocation, carrier rotation, metering |
| P03 | Inventory management (platform) | Kit lifecycle platform layer (full application: M20) |
| P04 | IAM + PKI + MFA | Central authentication, authorisation, certificate mgmt, MFA, hardware key support |
| P05 | Audit and tamper-evident logging | Signed, hash-chained, Merkle-log-backed audit trail (consumed by M14, M15, M20) |
| P06 | Encryption + HSM integration | Key management, digital-signature service, envelope encryption |
| P07 | Sandbox + change mgmt + rollback | Full-scale sandbox, blue-green orchestration, roll-back |
| P08 | Monitoring / observability | SNMP + OpenTelemetry + alerting + dashboards |
| P09 | Backup / restore / DR automation | Cross-site replication, backup cycle testing, failover |
| P10 | BI + reporting fabric | Semantic model to the Commission Power BI |

### 8.5 Integrations

| # | Integration | Purpose |
|---|---|---|
| I01 | IPRS | Voter identity verification against national ID |
| I02 | MRZ parsers | 2nd-gen ID, Maisha, Passport |
| I03 | Telco VPN carriers | Traffic steering, cost metering |
| I04 | the Commission Power BI | Downstream analytics consumer |
| I05 | Existing ABIS (if retained) | Integration adapter |
| I06 | Legacy Android device port | Feature-matrix compatibility layer for ~14,100 units |
| I07 | HSM integration | Signing operations |
| I08 | SMS / USSD gateways | Citizen notification + RTS fallback + M13 + M19 |
| I09 | Kenya DPA reporting | Data-breach notification pipeline to ODPC |
| I10 | KEBS / OEM warranty data feed | Warranty tracking; KEBS PVoC certificate import into M20 |

### 8.6 Operational Modules

| # | Module | Purpose |
|---|---|---|
| O01 | MDM console for KIEMS fleet | Remote configuration, wipe, OTA updates |
| O02 | Helpdesk / ticketing (Kenya-based) | Multi-channel intake, SLA tracking |
| O03 | Training / e-learning platform | Train-the-trainer content, assessment, certification |
| O04 | Warranty and defect-repair tracking | Ticket to repair-cycle mapping with OEM |
| O05 | Personnel dispatch and attendance | GPS-verified attendance for poll clerks |
| O06 | Data-migration toolkit | ETL from existing register, six-week deadline |

### 8.7 Compliance Instruments

| # | Instrument | Purpose |
|---|---|---|
| C01 | Kenya DPA compliance toolkit | DPIA, consent mgmt, SAR workflow, retention scheduler |
| C02 | ISO 27001:2022 ISMS docs + controls | Certification support |
| C03 | Elections Tech Reg 2017 test harness | Stress, load, penetration, functional |
| C04 | Source-code + IPR transfer toolkit | Repo escrow, build reproducibility, handover runbook |

---

## 9. Requirements Traceability Matrix

Cross-reference from CLIENT-EXCELLENCE-STRATEGY.md Parts I-XXI to implementing modules:

| Strategy Part | Topic | Implementing Module(s) |
|---|---|---|
| Part I | Petition-ground taxonomy (2013-2022) | All modules (cross-cutting threat model basis) |
| Part II §PM-01 | PMO co-location | O01-O06 operational programme |
| Part II §PM-02 | Tri-weekly steering + published minutes | C02 ISMS; P07 CCB log |
| Part II §PM-03 | CCB with HSM-signed change log | P05 + P06 + P07 |
| Part II §PM-04 | Escalation ladder tied to kit failure thresholds | M16 ECC; M19 queue |
| Part II §PM-05 | Communication cadence tightening | M16 ECC |
| Part II §PM-07 | Train-the-trainer cascade | O03 training platform |
| Part II §PM-08 | Independent Assurance Board | M15 IAB Dashboard |
| Part II §PM-09 | Documentation regime — immutable log | P05 audit log |
| Part II §PM-10 | Dry-run election 60 days before | O03 + M16 (operational commitment) |
| Part II §PM-11 | Election Command Centre | M16 |
| Part II §PM-12 | KIEMS asset tracking | M20 IMS |
| Part III §PB-01 | IAB secretariat (pro-bono) | M15 infrastructure |
| Part III §PB-02 | Public Verifier app (pro-bono) | M13 |
| Part III §PB-03 | WhatsApp + SMS gateway (pro-bono) | M13 |
| Part III §PB-04 | Code escrow | C04 |
| Part III §PB-06 | Bug-bounty programme | M14 |
| Part IV §A | Public transparency toolkit | M11 + M13 + M16 |
| Part IV §B | Legal proofing: Merkle, retention, server protocol | P05 + M14 + M15 |
| Part IV §C | Procedural proofing: s.44A protocol | M16 + M19 |
| Part IV §D | Technical proofing: cryptographic architecture | P06 + M14 + M20 |
| Part IX | Public confidence programme | M13 + M14 + M17 (prebunking) |
| Part X | AI utilisation | M17 + M26 (parallel) + M27 (parallel) + M28 (parallel) |
| Part XI | Reproducibility of outcomes | M14 |
| Part XII | No-challenge guarantee: four-wall architecture | M14 + M15 + M16 + P05 + P06 |
| Part XIII | Fraud prevention: ballot stuffing, ghost stations, dead voters | M20 + M26 (parallel) |
| Part XIV | One person, one vote, once | M03 ABIS + M05/M06 BVI + M27 (parallel) |
| Part XV §XV.1 | Public vote-counted verification | M13 + M14 |
| Part XV §XV.2 | Attendance counting | M19 |
| Part XV §XV.3 | Real-time queue tracking | M19 |
| Part XV §XV.4 | Ballot-to-voter chain of custody | M20 (smart-lid integration) + M27 (parallel) |
| Part XVI §XVI.1 | Boundary integrity | M18 |
| Part XVI §XVI.2 | Hate speech + CIB + deepfake detection | M17 |
| Part XVI §XVI.3 | Violent disruption detection | M17 |
| Part XVI §XVI.4 | Public information awareness | M13 + M17 |
| Part XVI §XVI.5 | E-SOC unified command | M16 |
| Part XVII | Automated vote counting + minimising rejected ballots | M08 + M09 + M10 + M27 (parallel) |
| Part XVIII §XVIII.1-2 | Rules compliance, guardrails by default | M08 (kit-side) + M16 (COC) |
| Part XVIII §XVIII.3 | Compliance dashboard (COC) | M16 |
| Part XVIII §XVIII.4 | Legality assurance programme | M16 + M23 (parallel) |
| Part XVIII §XVIII.5 | Voter process awareness | M13 (Learn tab) + M17 (prebunking) |
| Part XVIII §XVIII.6 | CAOC unified operations | M16 |
| Part XIX | Anomaly, fraud, and rigging detection | M26 (parallel) + M14 + M15 |
| Part XIX §XIX.2 | Risk-limiting audits (SHANGRLA) | M21 (parallel) |
| Part XIX §XIX.4 | Audit governance | M15 IAB |
| Part XX | Minimising cash-to-votes conversion | M23 (parallel) + M17 |
| Part XXI.A.2 | Diaspora voting architecture | M24 (parallel) |
| Part XXI.A.3 | Special voter categories | M25 (parallel) |
| Part XXI.A.4 | Ballot printing chain of custody (RFID) | M20 |
| Part XXI.A.7 | Public Verifier app hardening | M13 |
| Part XXI.A.14 | IAB escalation ladder | M15 |
| INVENTORY-MANAGEMENT.md §§1-11 | IMS full specification | M20 |
| Part XXI.A.5 | Ballot serial tracking | M63 |
| Part XXI.A.6 | Voter list public inspection | M64 |
| Part XXI.A.8 | Long-term archive + researcher access | M69 |
| Part XXI.A.9 | Referendum explainer + civic education | M68 + M37 |
| Part XXI.A.10 | Party symbol / colour uniqueness | M67 |
| Part XXI.A.11 | Recall and by-election operations | M48 + M50 |
| Part XXI.A.12 | Field-worker mobile tools | M31 + M32 + M33 + M34 |
| Part XXI.A.13 | Party-agent independent verification | M30 |
| Part XXI.A.15 | County coordination + security feeds | M56 + M60 |
| Part XXI.A.16 | Continuous voter register maintenance | M35 + M36 |
| Part XXI.A.17 | Voter ID card | M53 |
| Part XXI.A.18 | Party primaries oversight | M46 |
| Part XXI.A.19 | Financial forensics (cost-per-vote) | M66 |
| Part XXI.A.20 | Turnout forecasting | M44 |
| Part XXI.A.21 | Budget and expenditure transparency | M54 |
| Part XXI.A.22 | OAG interface | M59 |
| Part XXI.A.23 | Parliamentary reporting | M58 |
| Part XXI.A.24 | International observer coordination | M57 |
| Part XXI.A.25 | HR / payroll for election workers | M55 |
| Part XXI.A.26 | Referendum mechanics | M49 |
| Part XXI.A.27 | Election calendar and gazette | M52 |
| Part XXI.A.28 | Result declaration and certification | M51 |
| Part XXI.A.29 | Transmission legal-basis audit | M62 |
| Part XXI.A.30 | Ballot secrecy assurance testing | M61 |
| Part XXI.A.31 | Post-election review | M41 |
| Part XXI.A.32 | Media accreditation | M42 |
| Part XXI.A.33 | Observer accreditation | M43 |
| Part XXI.A.34 | Diaspora ballot physical logistics | M65 |
| Part XXI.A.35 | Nomination fee payment | M47 |
| Part XXI.A.36 | Mobile citizen verification | M29 |

---

## 10. Cross-Cutting Non-Functional Requirements

Every module must exhibit these properties. Individual module PRDs may set stricter targets.

### 10.1 Availability

- Register core: 99.99% monthly availability, no scheduled downtime during registration windows.
- RTS: 99.99% during polling window.
- Public Results Portal + M13 + M14 bundle: 99.99% read availability; degrade to signed static snapshots under DDoS.
- M16 ECC: 99.99% during E-Day.
- M20 IMS: 99.99% during election window.
- Recovery time objective (RTO): <= 30 minutes for RTS write path; <= 4 hours for BVR bulk enrolment; <= 15 minutes for Public Portal.
- Recovery point objective (RPO): <= 5 minutes for RTS; <= 15 minutes for BVR; <= 1 minute for HSM signing.

### 10.2 Scalability

- Concurrent RTS sessions: 60,000 sustained, 90,000 burst.
- ABIS records: 30,000,000 with linear 1:N throughput scaling.
- Public Portal peak: 5 million unique visitors within 1 hour of publication, 50 million total in first 24 hours.
- M13 Public Verifier: 30 million simultaneous active sessions at Form 34C publication.
- Device fleet: 65,287 KIEMS kits + 14,100 legacy Android units + 290 tallying tablets + 47 county workstations.

### 10.3 Performance

- BVR device enrolment: full biometric capture in <= 3 minutes per voter.
- BVI device identification: fingerprint match in <= 3 seconds; iris in <= 5 seconds; facial in <= 5 seconds.
- RTS auth: p99 <= 100 ms.
- ABIS 1:N against 30M records: p99 <= 8 seconds single-modality.
- Public portal page load: p99 <= 2 seconds at peak load.
- M13 tally lookup: p99 <= 500 ms.
- M18 geofence query: p99 <= 20 ms.
- M20 custody transfer: p99 <= 2 seconds (mobile app, including biometric sign).

### 10.4 Security (see SECURITY.md for full detail)

- ISO/IEC 27001:2022 certified within 90 days of Contract Effective Date.
- TLS 1.3 in transit; AES-256-GCM at rest; ephemeral session keys with perfect forward secrecy.
- Multi-factor authentication mandatory on every non-anonymous entrypoint.
- Per-kit digital certificates with sub-minute revocation propagation.
- HSM-backed signing for every state-change event.
- Merkle-log-backed audit trail with independent escrow mirror.
- Zero-trust internal service mesh; mTLS for all service-to-service calls.
- Bug-bounty programme active from T-365: KES 10K to KES 2M per validated finding.
- Pre-election adversarial red-team audit at T-90 by independent accredited firm.

### 10.5 Privacy

- Kenya DPA 2019 compliant with ODPC registration in place at go-live.
- Data minimisation: only fields the system genuinely needs are collected.
- Purpose limitation: voter records used only for the Commission statutory mandate.
- Storage limitation: retention scheduler enforces statutory limits.
- Rights fulfilment: subject-access-request workflow with 30-day SLA.
- Data-breach notification workflow to ODPC within 72 hours.
- No biometric data in any citizen-facing module (M13, M17, M18, M19, M20 public views).
- Computer-vision headcount in M19: no images stored, no face identification, no transmission.

### 10.6 Portability

- All biometric data stored in ISO/IEC 19794 formats.
- All exports available in CSV, JSON, and PDF with OpenAPI 3.1 specifications.
- No proprietary binary formats for authoritative data.
- IPR-transferred source code buildable and deployable by the Commission without vendor cooperation.
- M14 reproduce-me bundle: 15-year reproducibility commitment against Ubuntu 26.04 LTS.

### 10.7 Interoperability

- Every module exposes REST + gRPC APIs with OpenAPI 3.1 / protobuf schemas.
- Standardised event bus (CloudEvents 1.0 envelope, JSON payloads) for asynchronous integrations.
- M18 boundary data: GeoJSON (EPSG:4326) + WMS/WFS for GIS clients.
- M20 IMS: RFID ISO 14443A / ISO 15693 compatibility; Code 128 barcode fallback.

### 10.8 Resilience

- Offline-first for every device client with local encrypted queue, signed submission, deferred sync.
- Circuit breakers on every external dependency; graceful degradation on non-critical failures.
- Retry with exponential backoff and jitter on transient failures.
- Idempotency keys on every mutating request.
- Rate limiting at every public boundary.
- M13: three-replica CDN architecture; DDoS on origin does not stop citizen verification.

### 10.9 Auditability

- Every user action captured with 5-tuple: (who, what, which entity, from where, when).
- Signed and countersigned events with HSM.
- Hash-chained per-constituency Merkle logs; national root published every 10 minutes.
- Daily Kenya Gazette anchoring of Merkle root.
- 90-day online retention; permanent immutable archive.
- Independent escrow mirror to Auditor-General and IAB.

### 10.10 Accessibility

- WCAG 2.2 AA on every web surface (upgraded from 2.1 in v1).
- Device clients: tactile overlays, Braille cards, audio-guided workflow.
- M13: full TalkBack/VoiceOver support; font scaling to 200%; high-contrast mode.
- Screen-reader compatibility: NVDA, JAWS, VoiceOver, TalkBack.
- Physical-challenge type and assistance-required flag propagated end-to-end.

### 10.11 Localisation

- Interface at launch: English + Kiswahili on every surface.
- Voter-facing modules (M13, M17, M18, M19): additionally EN, SW, Kikuyu, Luo, Kalenjin, Luhya, Kamba, Meru, Kisii, Somali, Turkana, Maasai (per-module list in individual PRDs).
- All error messages and UX copy translated professionally.
- Language matrix extensible without code change.
- Date-time in EAT for presentation, UTC in storage and logs.

### 10.12 Observability

- Structured logs (JSON) with correlation IDs propagated end-to-end.
- Metrics via OpenTelemetry, aggregated in Prometheus, visualised in Grafana.
- Distributed traces on every request path exceeding two hops.
- Alerting via multi-channel (email, SMS, WhatsApp, dashboard).
- SLO dashboards public inside the Commission.
- Live health page fed to M16 ECC during polling window.

---

## 11. Data Ownership, IPR, Retention

### 11.1 Data ownership

All voter registration records, results data, candidate data, audit logs, inventory data, and biometric templates are the **property of the Independent Electoral & Boundaries Commission** at all times. Datacraft, Mechrevo, and any subcontractor is a data processor, not a data controller.

### 11.2 IPR classification

- **Custom Software** (VPRS greenfield, all M13-M28 modules, integrations, compliance toolkits): IPR vests in the Commission per GCC 15.4 at Operational Acceptance.
- **Open-source releases** (M13 app source, M14 reproduce-me bundle, M18 boundary dataset, M18 simulator front-end): released under MIT licence / CC BY 4.0 as specified per module.
- **Standard Software** (Datacraft pre-existing platforms: Lindela, Askari, Hyperion-X, dArchiva, Flowforge, MeGuard, Shule): licensed to the Commission on perpetual, fully paid-up, irrevocable terms per GCC 16.
- **Third-party software** (HSM SDK, biometric matcher, ICR engine, monitoring stack): licensed to the Commission per each vendor's OEM terms.

### 11.3 Data retention

| Data class | Retention | Rationale |
|---|---|---|
| Voter register | Duration of contract + 5 years | Kenya electoral law |
| Biometric templates | Duration of contract + 5 years | Same |
| Results (Forms 34-38 signed images + tallies) | Permanent | Constitutional archive |
| Audit logs | 90 days online + permanent immutable archive | Contract + petition defence |
| IMS chain-of-custody records | 7 years + permanent Merkle root archive | OAG audit; petition window; Limitation of Actions Act |
| M17 evidence packages (hate speech, CIB) | 7 years | NCI Act s.13; potential criminal proceedings |
| M17 social media captures | 7 years | Same |
| M19 queue data | 7 years | Disenfranchisement evidence |
| Participation tokens (M14) | 7 years | Petition window evidence |
| Voice/session recordings on privileged access | 5 years | Insider-threat forensic trail |
| Sandbox / test data | Session lifetime | Synthetic data only |
| Personnel data | Employment duration + 7 years | Kenya labour + tax law |
| M19 SMS opt-in phone numbers | T+30 days | Deleted on schedule per DPA 2019 consent basis |

---

## 12. Testing, Certification, Acceptance

### 12.1 Testing regime

| Test type | Scope | Owner | Timing |
|---|---|---|---|
| Unit | Per code module, >= 80% coverage (>= 90% for M14 aggregation) | Dev team | Continuous |
| Integration | Cross-module contracts | QA team | Per PR |
| Performance / load | 60K concurrent RTS; 30M ABIS; 5M Portal; 55K IMS streams; 10M M13 users | QA + Independent | Pre-milestone |
| Stress | 2× target load until degradation | QA + Independent | Pre-election |
| Chaos | Kill random components; measure survival | SRE | Weekly |
| Security | SAST, DAST, dependency scanning, secret scanning | Security team | Per PR + annual pen test |
| Penetration | Full-scope black-box + white-box | Independent accredited firm | Annual + T-90 pre-election |
| Failover / DR | Primary-to-mirror failover exercise | SRE + the Commission | Quarterly |
| Field trial | Geo-fencing, offline sync, cellular in Kenyan constituencies | Operations + the Commission | Pre-pilot |
| User acceptance | End-to-end with the Commission operators | the Commission + Datacraft | Per milestone |
| Regulatory certification | Elections (Technology) Regulations 2017 | Independent certifier | Before every general election |
| Biometric accuracy | FAR / FRR / throughput | Independent lab | Annual |
| Accessibility | WCAG 2.2 AA automated (axe-core) + manual (screen reader) + independent audit | QA + Independent | T-90 pre-election |
| Reproducibility | M14 bit-identical re-run on 3 platform/OS combinations | QA + IAB | T-60 dry run + post-Form 34C |
| Election-day dress rehearsal | Full end-to-end with mock election data on sandbox | Everyone | T-60 |
| s.44A protocol drill | All workflow steps with real kits | Operations | T-60 |
| IMS full-fleet dry run | All 65,287 UIDs provisioned and cascaded | Datacraft Ops | T-60 |

### 12.2 Acceptance gates

- **Factory Acceptance Test (FAT)**: on Datacraft/Mechrevo premises.
- **Site Acceptance Test (SAT)**: on the Commission infrastructure post-installation.
- **Operational Acceptance Test (OAT)**: full end-to-end after commissioning.
- **Regulatory certification**: per Elections (Technology) Regulations 2017.
- **Milestone-based acceptance** per Preliminary Project Plan.

---

## 13. Delivery Milestones

| Phase | Indicative date | Key deliverables |
|---|---|---|
| Mobilisation | CED + 30 days | Project Plan, securities, PKI ceremony, sandbox, the Commission on-boarded, IAB chartered |
| IMS live (Nairobi provisioning centre) | CED + 90 days | M20 + M39 + M63 primary + mirror active; Mechrevo MES integration live |
| Data migration | Weeks 3-8 from CED | Existing register migrated with reconciliation reports |
| BVR readiness | Month 3 | M01, M02, M03 ready |
| VPRS + CVR launch | Month 4 | M04 public portal live; M35 continuous register pipeline live |
| CRMS + Party systems launch | Month 6-8 | M12 party portal + RO clearance; M45 party registration; M67 symbol registry |
| Election calendar live | Month 4 | M52 gazette notice management active |
| M13 + M29 public beta | Month 6 | Public Verifier app + the Commission Verify App in Play Store / App Store |
| M18 open data live | Month 4 | Boundary explorer public |
| Voter ID card system | Month 5 | M53 NRB integration live |
| Party primaries oversight | Month 7 | M46 active for nomination season |
| Nomination fee payment | Month 7 | M47 M-Pesa integration live |
| Poll worker training LMS | T-180 | M38 live; certification tracking active |
| Voter education CMS | T-180 | M37 content deployed across all channels |
| Media + observer accreditation | T-180 | M42 + M43 portals live; M57 international missions on-boarded |
| Turnout forecasting | T-180 | M44 analytics pipeline connected to M06 events |
| County coordination | T-150 | M56 county government data-sharing live |
| Mobile field apps (party agent, observer) | T-180 | M30 + M34 distributed via MDM |
| Mobile field apps (clerk, field support, dispatch) | T-120 | M31 + M32 + M33 distributed via MDM |
| HR / payroll system | T-150 | M55 worker engagement workflow live |
| Voter list public inspection | T-100 | M64 portal open to public (T-90 statutory window) |
| Deregistration appeal | T-100 | M36 adjudication workflow live |
| Ballot secrecy assurance | T-120 | M61 automated test suite passing |
| M17 + speech integrity monitoring live | T-90 (May 2027) | Speech integrity pipeline active |
| BVI + RTS end-to-end | T-120 (April 2027) | All device + back-end ready |
| ECC activated | T-90 (May 2027) | M16 24x7 staffed; M60 medical coordination live |
| Regulatory certification | T-60 (June 2027) | Elections Tech Reg 2017 pass |
| Field trial | T-90 (May 2027) | Multi-county mock election; all M29-M34 apps in-field |
| Transmission legal-basis audit | T-90 | M62 audit trail active across all data-sharing flows |
| Dress rehearsal | T-60 (June 2027) | Full end-to-end; s.44A drilled; IMS full-fleet; all 69 modules exercised |
| Software freeze | T-30 (July 2027) | No changes except dual-approved security patches |
| Gazette hash commitment | T-30 (July 2027) | All 69-module software hashes in Kenya Gazette |
| Pre-emptive evidentiary bundle | T-14 (July 2027) | 15-item bundle to Supreme Court IT Registrar |
| Election Day | 2027-08-10 | All 69 modules live; ECC staffed 24x7; SRE on-call; field support in 47 counties |
| Form 34C + reproducibility bundle | 2027-08-10 + 4 hours | Full national tally + `make reproduce` published |
| Result declaration and certification | T+1 to T+7 | M51 formal certification pipeline; M40 evidence packs on demand |
| Post-election financial forensics | T+30 | M66 cost-per-vote analysis published |
| Post-election reconciliation report | T+30 (September 2027) | M20 all-kit disposition to Auditor-General via M59 |
| Post-election review | T+45 | M41 findings registered; M69 archive transition begins |
| Post-election handover | T+60 (October 2027) | Full source-code transfer for all 69 modules, documentation, sandbox handover |
| Petition window support | T+30 to T+180 | Evidence packs on demand via M40; expert witness support |
| Long-term archive stabilisation | T+180 | M69 operational; KNADS integration complete; first format-migration pipeline verified |

---

## 14. Assumptions & Constraints

### 14.1 Assumptions

- the Commission provides the converged server + storage + networking + HSM.
- the Commission provides telco VPN infrastructure and SIM/eSIM commercial arrangements.
- IPRS test environment access granted within 30 days of CED.
- ~14,100 legacy Android devices with known make/model/OS/SDK.
- the Commission nominates an independent certifier under Elections (Technology) Regulations 2017.
- IAB chartered with genuine independence (not a rubber-stamp) per Strategy §PM-08.
- Mechrevo MES API available for IMS birth-record ingestion from factory start.
- Al Ghurair ballot printer provides RFID chip manifest compatible with IMS §2.3 ballot UID format.

### 14.2 Constraints

- Fixed-price 3-year contract; LDs 0.5%/week capped at 10%.
- Custom Software IPR vests in the Commission; source-code transfer mandatory.
- Kenyan data residency for all production data (including M17 AI models).
- 40% local content per Form 8.
- Six-week data-migration deadline from Contract Effective Date.
- 24-hour repair SLA during 14-day pre-election window.
- M14 reproduce-me bundle: 15-year reproducibility commitment (Ubuntu 26.04 LTS support window).
- M17 deepfake detection: honest limitation — state-level deepfakes at frontier adversarial capability may evade current detectors; human review required.

---

## 15. Risks (top 15 software-specific)

| # | Risk | Mitigation |
|---|---|---|
| R01 | ICR misreads Form 34A causing tally errors | Human review below confidence threshold; M14 reproducibility makes any error visible to any independent verifier |
| R02 | ABIS false-match rate above target | Multi-modal fusion; independent lab certification; M14 second-implementation validation |
| R03 | RTS thundering herd at station-close | Sharded queues per station; back-pressure; kits retry with jitter |
| R04 | Public Portal / M13 / M29 DDoS at publication | 3-replica CDN-first architecture; upstream scrubbing; signed static snapshots; USSD fallback for M29 |
| R05 | Insider tamper with results | Two-person integrity; M15 IAB continuous log access; M14 reproducibility; transparency log |
| R06 | 200+ kits missing post-election (repeat of 2022) | M20 IMS + M63 ballot serial chain with Merkle chain of custody; daily Auditor-General extract via M59; IAB alert on any seal breach |
| R07 | Hate-speech-driven violence (repeat of 2007-2008) | M17 monitoring live from T-90; prebunking at scale; NCIC referral < 15 min SLA |
| R08 | Social media platform API revoked during election | Pre-signed data-sharing agreements; web-scraping fallback for public content; M17 independence from any single platform |
| R09 | M14 reproducibility bundle fails to be bit-identical | Determinism enforced by CI matrix (3 OS/arch combinations); second-implementation validation at T-60 |
| R10 | IAB independence undermined by the Commission pressure | MoU with genuine independence provisions; IAB L4 emergency declaration routes directly to Supreme Court IT Registrar, bypassing the Commission |
| R11 | 69-module integration complexity causes cascade failures | Service mesh Linkerd 2 isolates failures; circuit breakers on every dependency; chaos engineering weekly from T-300; module-boundary contracts enforced by OpenAPI contract tests |
| R12 | M35 CVR data quality (life-event ingestion from CRS/NRB) introduces systematic register errors | Reconciliation checks at every CRS/NRB batch; anomaly-detection rules on unexpected change rates; M36 appeal pipeline as safety valve; M64 public inspection as final correction window |
| R13 | M33 personnel dispatch GPS fraud (fake check-in) | Geofence cross-check with known station coordinates; photo-at-check-in with device EXIF; M16 ECC real-time anomaly flag on clustered check-ins |
| R14 | M55 M-Pesa bulk disbursement failure on election night | Pre-tested with Safaricom for 290K-transaction batch; retry logic with idempotency; manual bank-transfer fallback authorised in runbook |
| R15 | Election calendar change post-mobilisation destabilises 69-module delivery plan | All milestones are T-relative; re-anchor on gazette change; contractual change-order process; contingency buffer of 30 days built into T-300 to T-180 phase |

---

## 16. Glossary (abbreviated)

| Term | Definition |
|---|---|
| ABIS | Automated Biometric Identification System |
| BVI | Biometric Voter Identification (polling-day) |
| BVR | Biometric Voter Registration |
| BVV | Biometric Voter Verification (public register inspection) |
| CAOC | Compliance and Awareness Operations Centre |
| CED | Contract Effective Date |
| CIB | Coordinated Inauthentic Behaviour |
| COC | Compliance Operations Centre |
| CRMS | Candidate Registration and Management System |
| CRO | County Returning Officer |
| DPA | Kenya Data Protection Act 2019 |
| DPIA | Data Protection Impact Assessment |
| E-SOC | Election Security Operations Centre |
| ECC | Election Command Centre (unified E-SOC + COC + CAOC) |
| HSM | Hardware Security Module |
| IAB | Independent Assurance Board |
| the Commission | the electoral authority |
| IEMS | Integrated Elections Management System |
| IMS | Inventory Management System (M20) |
| IPRS | Integrated Population Registration System |
| KIEMS | Kenya Integrated Elections Management System (device) |
| MDM | Mobile Device Management |
| NCIC | National Cohesion and Integration Commission |
| ODPC | Office of the Data Protection Commissioner |
| ODPP | Office of Director of Public Prosecutions |
| OAT | Operational Acceptance Test |
| PKI | Public Key Infrastructure |
| PO | Presiding Officer |
| RLA | Risk-Limiting Audit |
| RO | Returning Officer |
| RTS | Results Transmission System |
| s.44A | Elections Act 2011 s.44A — complementary mechanism declaration |
| SHANGRLA | Sharper Discrepancy Bounds for Risk-Limiting Audits (Stark 2020) |
| VPRS | Voter Pre-Registration System |

Full glossary in `glossary.md` (to be produced).

---

## 17. Change Management

- Post-award changes governed by the Configuration Control Board (`../POST_AWARD_PROGRAMME.md` §4).
- No production configuration change deployed without dual-control sign-off.
- Software freeze T-30 before every gazetted election event.
- Emergency-change protocol: 3-hour window, joint the Commission + Datacraft + Security sign-off, post-hoc CCB ratification within 24 hours, entry in transparency log.
- All CCB decisions HSM-signed and appended to the public audit trail (P05 + M15).

---

## 18. Document Control

| Version | Date | Author | Change |
|---|---|---|---|
| 1.0 | 2026-08-18 | Datacraft Solutions Architecture | Initial issue with bid |
| 2.0 | 2026-08-21 | Datacraft Solutions Architecture | Extended to 28 modules (M13-M20 this document; M21-M28 parallel track); added scope evolution §2, requirements traceability matrix §9, updated personas §4, updated constraints §14, 10 new risks §15; WCAG upgraded to 2.2 AA; kit fleet updated to 65,287; reproducibility commitment added |
| 3.0 | 2026-08-21 | Datacraft Solutions Architecture | Extended to full 69-module scope (M29-M69 added); module index grouped by function §8; traceability matrix extended to Parts XXI.A.5-36; delivery milestones extended for all 69 modules §13; risk register expanded to top-15 §15; scope evolution table updated §2 |

Subsequent revisions post-award tracked under Configuration Control Board (P07).
