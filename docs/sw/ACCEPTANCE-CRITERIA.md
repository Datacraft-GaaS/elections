# Acceptance Criteria — IEBC IEMS 2027

**Version:** 2.0 — Full 69-module scope
**Companion to:** `PRD.md`, `SECURITY.md`, `DATACRAFT_SOFTWARE_SCOPE.md`
**Election day:** 2027-08-10
**Teaming Agreement reference:** §4.1 (Operational Acceptance triggers IP vesting)

---

## 1. Acceptance Framework

### 1.1 Definition of Done

A module is "done" when all of the following are satisfied:

1. All stated functional requirements (FR-MXX-NNN) pass in the staging environment under real test data (synthetic or anonymised production replica).
2. All non-functional requirements (NFR-MXX-NNN) pass with numeric targets met (measured, not estimated).
3. Integration acceptance: all upstream and downstream modules with a declared dependency sign off on the contract.
4. User acceptance: each named persona sign-off list for the module gives written acceptance (may be a structured UAT session record or a signed UAT report).
5. Security acceptance: pen-test findings for this module are resolved to P2 or below; DPIA is filed with ODPC; relevant SR-* controls in `SECURITY.md` have evidence artefacts.
6. Regulatory acceptance: applicable KEBS PVoC certificates (for hardware-adjacent software), ODPC DPIA on file, PPRA compliance, and IEBC Governance Board sign-off.
7. Operational Acceptance: criteria in §1.3 below are met, triggering IP vesting per Teaming §4.1.
8. Documentation: runbook, on-call playbook, ADR log, and user manual are complete and reviewed by IEBC ICT.

### 1.2 Acceptance Test Hierarchy

| Level | Scope | Owner | Environment |
|---|---|---|---|
| Unit | Individual functions, classes, models | Datacraft developer | CI (isolated, no network) |
| Integration | Service-to-service contracts, Kafka topics, DB schemas | Datacraft QA | Integration environment |
| End-to-end (E2E) | Full user journey across modules | Datacraft QA + IEBC ICT | Staging environment |
| User Acceptance (UAT) | Persona-driven scenario execution | IEBC business users + Datacraft | Staging environment |
| Operational Acceptance (OAT) | Full system, SLA measurement, DR, security | Datacraft + IEBC + IAB | Pre-production environment |
| Regulatory Certification | Independent certifier nominated by IEBC | Third party | Pre-production (clean) |

### 1.3 Operational Acceptance (Teaming §4.1 — IP Vesting Trigger)

Per Teaming Agreement §4.1, Custom Software IP vests in IEBC at Operational Acceptance. Operational Acceptance is achieved when:

- (a) OAT certificate signed by IEBC Project Manager per GCC 27
- (b) All OAT criteria in §3 pass for the module
- (c) ISO 27001 surveillance audit covering the module shows no critical findings
- (d) Source code deposit made to escrow custodian within 24 hours of OAT certificate
- (e) Reproducibility build from escrow verified by IEBC ICT

Until OAT certificate, IP remains with Datacraft. Partial OAT certificates per module class are permissible per Teaming §4.1(b).

### 1.4 Test Data Management

- No production personal data in any environment below pre-production.
- Synthetic data generated using Faker + custom Kenyan demographic distributions; 30 million synthetic voter records for ABIS and RTS load tests.
- Anonymised production replicas: aggregate station counts and historical result patterns used for load calibration; no individual voter records.
- Test data classified per the module data classification in `SECURITY.md` §13; synthetic RESTRICTED data handled with same controls as production.
- Test data lifecycle: destroyed (cryptographic erasure) at end of each test cycle; erasure certificate filed.

### 1.5 Test Environment Topology

| Environment | Purpose | Data | Network |
|---|---|---|---|
| Dev | Developer unit + integration tests | Synthetic (minimal) | Isolated, no kit hardware |
| Integration | Contract tests, Kafka consumer/producer | Synthetic (30M records) | Closed loop |
| Staging | E2E, UAT, performance | Synthetic + anonymised | Full kit hardware (20 kits), reduced carrier |
| Pre-production | OAT, DR, security acceptance | Synthetic (production scale) | Full carrier integration, 3-site topology |
| Production | Live election | Real | Full |
| Air-gapped build | CI/CD, reproducibility verification | None | No internet egress |

---

## 2. Cross-Cutting Acceptance Criteria

These apply to all 69 modules unless a module-specific override is stated in §3.

### 2.1 Availability SLA

| Module class | SLA | Measurement window | Error budget |
|---|---|---|---|
| Election-critical (M01, M02, M03, M05, M06, M08, M09, M10, M27, M49, M50, M51) | 99.99% | 30-day rolling | 4.38 min/month |
| Cycle-critical (M04, M11, M12, M13, M14, M15, M16, M20, M21, M22, M24, M25, M26, M28, M35, M39, M40) | 99.9% | 30-day rolling | 43.8 min/month |
| Routine operations (all others) | 99.5% | 30-day rolling | 3.65 hr/month |

Measurement: Prometheus `up` metric sampled every 30 seconds. SLA dashboard reviewed weekly by SRE and monthly by IAB.

### 2.2 Performance SLA

| Endpoint class | P95 latency | P99 latency | Throughput |
|---|---|---|---|
| Kit biometric authentication (BVI 1:1) | 800 ms | 1,500 ms | 18,000 req/s (60,000 stations × 0.3 req/s average) |
| Kit RTS form submission | 500 ms | 1,000 ms | 60,000 concurrent sessions |
| ABIS 1:N match (BVR dedup) | 5 s | 8 s | 10,000 enrollments/hour sustained |
| Public Portal page load (CDN-served) | 200 ms | 500 ms | 5M unique visitors/hour |
| CRMS nomination submission | 1 s | 2 s | 15,000 nominations over 30-day window |
| Grievance submission | 2 s | 4 s | 10,000/day peak |
| Merkle-inclusion proof retrieval | 500 ms | 1 s | Any observer |
| DSAR response generation | — | — | 30 calendar days (workflow SLA) |

Performance tests run with k6 against pre-production at 2× design load (per Elections Technology Regulations 2017 §stress-test requirement).

### 2.3 Accessibility

WCAG 2.2 AA baseline for all modules with a user-facing interface. WCAG 2.2 AAA target for M68 (Referendum Explainer Platform — widest public audience including low-literacy users). Accessibility audit by independent WCAG specialist at SAT. Assistive technology compatibility: VoiceOver (iOS), TalkBack (Android), NVDA + Chrome (desktop). Kenyan Sign Language video content required on M37, M68. Audio-guided workflow required on M01, M05 (kit-side).

### 2.4 Security Baseline

All modules must pass:

- OWASP ASVS Level 3 for election-critical modules (M01-M12, M27, M40, M51, M63, M69); Level 2 for all others
- No critical or high-severity findings unresolved in Trivy (container SCA) or Snyk (dependency SCA) at release; medium-severity findings tracked with 30-day resolution plan
- SBOM (SPDX 2.3) published and signed per image; hash in transparency log within 1 hour of build
- Source-code escrow deposit within 24 hours of tagged release
- Gitleaks secret scan clean in CI; no secrets in any git history

### 2.5 Data Protection

Per module handling personal data:
- DPA 2019 lawful basis documented per endpoint (see `SECURITY.md` §11.4.1)
- DPIA filed with ODPC before any data processing begins in pre-production or production
- Breach-notification runbook specific to the module reviewed and approved by IEBC DPO

### 2.6 Observability

Every module must ship with:
- OpenTelemetry instrumentation: traces (every request), metrics (RED + USE), logs (structured JSON, correlation ID on every log line)
- RED metrics: request rate, error rate (by HTTP status + gRPC code), duration (P50, P95, P99 histograms) per endpoint
- USE metrics (infrastructure): utilisation, saturation, errors per CPU/memory/disk/network
- Correlation ID: UUID7 injected at API gateway; propagated through all downstream services via W3C Trace-Context headers
- Grafana dashboard per module: RED dashboard + SLO error-budget dashboard; reviewed in UAT before acceptance
- Alert rules: at minimum, SLO breach (burn rate alert), P0 security event (from SIEM), service `down` (from Prometheus `up`)

### 2.7 Reproducibility

Every module must produce a byte-identical build artefact from a tagged commit + declared input manifest. Verified in CI: three consecutive builds from the same tag must produce identical hashes. Non-reproducibility is a P1 finding and blocks acceptance. Build manifest published with each release. Applies to: container images, APKs (Android), firmware binaries, kit-side data bundles.

### 2.8 Multi-Language Support

- English (EN) + Kiswahili (SW): mandatory baseline for all user-facing modules
- Additional Kenyan languages per module: see §3 (module-specific notes for M01, M05, M08, M25, M37, M68 which have constituency-specific language requirements)
- Right-to-left: not required for Kenyan languages; verify layout is not broken by longer Kiswahili strings
- Translation review: by native speaker approved by IEBC, not machine-translation-only, before UAT

### 2.9 Documentation

Every module must have, at FAT:
- Runbook: deployment, startup, shutdown, configuration, known failure modes, recovery steps
- On-call playbook: alert → diagnosis → mitigation → escalation for every defined alert
- Architecture Decision Records (ADRs): one per significant architectural decision; stored in `docs/adr/` in the module repository
- User manual: for each persona, a step-by-step guide covering their primary workflows; reviewed and signed by a representative from IEBC for that persona

---

## 3. Per-Module Acceptance Criteria

### M01 — BVR Device (Biometric Voter Registration Device)

**Functional:** FR-M01-001 through FR-M01-020 pass in staging (full enrolment flow, offline queue, sync, transfer, distress finger, firmware OTA, language toggle, accessibility modes). Verified with 20 physical kits in staging environment.

**Non-functional:** Fingerprint capture throughput ≥ 12 enrolments/hour per kit (target 15). Kit offline autonomy: 7-day queue ring buffer without data loss. Battery: full day operation (8 hours continuous, 12 hours light) without external power. Boot to ready: ≤ 90 seconds. P95 biometric capture quality acceptance rate ≥ 95% on IEBC synthetic test set.

**Integration:** M02 BVR Backend signs off that: (a) all enrolment records received match kit-side log hashes, (b) deduplication trigger to M03 fires for every new enrolment, (c) revocation propagates to kit ≤ 60 seconds.

**User acceptance:** Enrolment Clerk persona: IEBC-designated enrolment supervisor conducts UAT with 50 synthetic voters. Supervisor persona: transfer workflow UAT. Accessibility persona: visually impaired voter validates audio-guided flow.

**Security acceptance:** Physical pen-test on kit (tamper seal, JTAG probe, USB attack). Firmware-signing verification. Geo-fence bypass attempt fails. Certificate pinning bypass attempt fails. Distress signal fires on correct finger without alerting suspect.

**Regulatory:** KEBS PVoC certificate for imported KIEMS kit hardware (not software, but software must be installed on PVoC-certified hardware before regulatory acceptance). ODPC DPIA on file for biometric capture. Elections (Technology) Regulations 2017 functional test covers M01 workflows.

**Operational Acceptance (Teaming §4.1):** 20-kit field trial with full enrolment workflow; 99.99% uptime over 7-day sustained test; all SR-DEV-* and SR-PKI-003 controls evidenced.

**Rollout:** Pilot (2 counties, 200 kits) → Pass → National deployment (65,287 kits).

---

### M02 — BVR Backend

**Functional:** FR-M02-* pass: enrolment ingestion, deduplication trigger, register update, export (dual-control enforced), revocation. Idempotency: duplicate submissions from same kit produce exactly one register record.

**Non-functional:** 10,000 enrolments/hour sustained for 30 days without data loss. P99 ingest latency ≤ 2 s. PostgreSQL WAL replication lag < 5 s to mirror. Register export (30M records) completes within 4 hours.

**Integration:** M03 ABIS confirms deduplication triggers received; M35 CVR confirms register updates propagated; M05/M06 BVI Backend confirms voter roll delta sync.

**User acceptance:** IEBC register manager persona: bulk export dual-control workflow. IEBC DPO: DSAR workflow produces correct output within 30-day SLA.

**Security acceptance:** SQL injection suite clean; OPA separation-of-duties policy tested (single account cannot simultaneously read templates + modify register + access audit logs); bulk export enforces dual approval; HSM countersigning verified on all write events.

**Regulatory:** ODPC DPIA covering voter register storage. Elections Act §6A enforcement verified (no records accepted outside registration window).

**Operational Acceptance:** 30-day OAT with synthetic 30M records; full failover to mirror tested; DR RTO ≤ 30 min; RPO ≤ 60 s verified.

**Rollout:** Integration test → OAT → National.

---

### M03 — ABIS (Automated Biometric Identification System)

**Functional:** 1:N biometric deduplication on 30M synthetic records. Adjudication workbench handles low-confidence matches. FMR (False Match Rate) ≤ 0.001% on IEBC test set. FNMR (False Non-Match Rate) ≤ 0.1%.

**Non-functional:** P99 1:N match ≤ 8 s. Throughput: 10,000 matches/hour sustained. Confidential compute enclave attested and functional. Biometric templates never exposed outside enclave memory (verified by memory inspection test).

**Integration:** M02 enrolment trigger arrives → M03 processes → match result returned to M02 with confidence score. M06 BVI 1:1 match path verified.

**User acceptance:** IEBC biometric officer: adjudication workbench UAT with simulated low-confidence cases. IEBC DPO: template deletion workflow (DSAR erasure).

**Security acceptance:** Template exfiltration attempt from enclave blocked. ABIS vendor SDK independent code review complete. No unresolved P1 pen-test findings. Confidential compute attestation report signed by IEBC CISO.

**Regulatory:** Independent biometric accuracy re-certification by accredited lab (FAR/FRR against IEBC-supplied test set). ODPC DPIA covering special-category biometric data. Elections (Technology) Regulations 2017 biometric accuracy test.

**Operational Acceptance:** Full 30M synthetic record load test; FMR/FNMR within targets; enclave attested in pre-production.

**Rollout:** Lab certification → OAT → National.

---

### M04 — VPRS (Voter Pre-Registration System)

**Functional:** FR-M04-*: self-service form, IPRS integration, QR code generation for kit handoff, duplicate submission prevention, confirmation notification.

**Non-functional:** P99 form submission ≤ 2 s. Availability 99.9%. CAPTCHA effectiveness verified. Rate limiting verified (10 submissions/IP/hour).

**Integration:** M01/M05 BVR/BVI devices accept VPRS QR handoff. M35 CVR syncs pre-registration status.

**User acceptance:** Self-registering voter persona: end-to-end VPRS flow with IPRS validation. IEBC pre-registration officer: batch review workflow.

**Security acceptance:** Consent withdrawal workflow tested. Personal data deletion tested. IPRS API key rotation tested. DPIA on file with ODPC.

**Regulatory:** ODPC DPIA; DPA 2019 consent management verified; IPRS data sharing agreement in place.

**Operational Acceptance:** 100,000-user load test. Consent audit log complete.

**Rollout:** Soft launch (Nairobi) → National.

---

### M05 — BVI Device (Biometric Voter Identification Device)

**Functional:** FR-M05-*: voter identification by fingerprint (1:1 on-device against station voter roll), time-fence, geo-fence, distress finger, offline queuing, OTA update, connectivity fallback, language toggle, accessibility.

**Non-functional:** P95 1:1 biometric authentication ≤ 3 s (target ≤ 2 s). False acceptance rate (FAR) ≤ 0.001%. False rejection rate (FRR) ≤ 0.5%. Kit operates 8 hours without recharge. Offline queue: all authentication events preserved for 7 days without network.

**Integration:** M06 BVI Backend receives all authentication events; M26 anomaly detector receives kit telemetry; M16 ECC receives health pings.

**User acceptance:** Poll clerk persona: full election-day simulation UAT (1,000 synthetic voters through 1 kit over 12 hours). Presiding officer: session open/close workflow. Accessibility persona: audio-guided flow for blind voter.

**Security acceptance:** Certificate pinning bypass attempt blocked. Geo-fence bypass (GPS spoof) blocked at TPM pre-app layer. Time-fence bypass attempt blocked. Physical tamper tests (JTAG, USB). Duress finger verified covert.

**Regulatory:** Elections (Technology) Regulations 2017 functional test. KEBS PVoC hardware certificate. ODPC DPIA (1:1 biometric match is special-category processing under DPA s.29). Biometric accuracy re-certification by independent lab.

**Operational Acceptance:** 60-day pre-election field trial (100 stations, 2 counties). 99.99% uptime over 7-day sustained test. All SR-DEV-* and SR-PKI-003 evidenced.

**Rollout:** Field trial → Pilot election (by-election if available) → General election.

---

### M06 — BVI Backend

**Functional:** Receives authentication events from M05 kits. Validates kit certificate. Countersigns event with HSM. Appends to Merkle chain. Maintains authentication audit log. Provides 1:1 match API for online authentication fallback.

**Non-functional:** P99 event processing ≤ 500 ms. Throughput: 18,000 events/s sustained for 12 hours (election day peak). Zero data loss under failover.

**Integration:** M05 (event ingestion), M03 (online fallback 1:1), M10 (turnout metrics), M26 (anomaly alerts), M40 (evidence bundle).

**User acceptance:** IEBC ICT officer: audit log query and export. IAB: Merkle-inclusion proof retrieval.

**Security acceptance:** HSM countersigning verified on all events. Replay attack blocked (monotonic counter checked). Certificate revocation propagation ≤ 60 s verified.

**Regulatory:** ODPC DPIA. Elections Act §44 compliance (authentication log available for petition).

**Operational Acceptance:** Full election-day load test at 2× peak. Failover to mirror RTO ≤ 30 min.

**Rollout:** OAT → National.

---

### M07 — BVV (Biometric Voter Verification Portal)

**Functional:** Voter self-lookup by NID or name; returns polling station and ward; no biometric data returned. Rate-limited public API.

**Non-functional:** P99 lookup ≤ 500 ms. Availability 99.9%. Rate limit: 100 lookups/IP/day. CAPTCHA on suspicious patterns.

**Integration:** M35 CVR (read-only replica). M26 anomaly detector (unusual lookup volume).

**User acceptance:** Registered voter persona: self-lookup before election day. IEBC helpdesk: assisted lookup for voters without smartphone.

**Security acceptance:** NID enumeration prevented (rate limit + CAPTCHA). No biometric data in API response. Audit log of all lookups (anonymised IP + NID hash).

**Regulatory:** DPA 2019 s.30(1)(e) public task lawful basis confirmed. Elections Act §35 (right to inspect register) compliance.

**Operational Acceptance:** 5M lookups/day load test. CDN cached responses verify no PII in cache.

**Rollout:** Open from register cut-off date.

---

### M08 — RTS Device (Results Transmission System Device)

**Functional:** FR-M08-*: capture Form 34A image, OCR pre-fill, presiding officer review, biometric sign-off by RO, transmission (primary LTE, failover dual-SIM, SMS/USSD fallback), offline queue, transmission receipt (signed acknowledgement from back-end). No ballot choices stored or transmitted on the device.

**Non-functional:** Transmission P99 ≤ 30 s on LTE (2 Mbps baseline). SMS fallback delivers Form 34A content within 5 min. Kit offline autonomy: full results queue held for 48 hours without loss. Image quality verification: SSIM ≥ 0.95 against original.

**Integration:** M09 RTS-RO tablet receives constituency summary. M10 RTS Backend accepts events. M27 Intent Preservation seals form before transmission. M26 receives telemetry. M63 Ballot Serial Chain confirms ballot count reconciliation.

**User acceptance:** Presiding officer persona: full results transmission simulation UAT. RO persona: constituency aggregation UAT. Election observer (M34): result visibility UAT.

**Security acceptance:** Man-in-the-middle attempt blocked (cert pinning). Replay of earlier signed form rejected (monotonic counter). RTS metadata analysis confirms no ballot-secrecy leak (transmission does not reveal vote order). Physical coercion simulation: duress flow tested.

**Regulatory:** Elections Act §39(1C) compliance (electronic transmission of results). Elections (Technology) Regulations 2017 functional test. TLA+ formal proof of RTS state machine (no double-counting, no lost writes) submitted as regulatory evidence.

**Operational Acceptance:** 60,000-concurrent-session load test sustained 30 min with P99 auth ≤ 100 ms. SMS/USSD fallback end-to-end field trial. All SR-NET-* controls evidenced.

**Rollout:** Field trial (100 stations) → Pilot election → National.

---

### M09 — RTS-RO (Returning Officer Results Tablet)

**Functional:** Constituency-level aggregation; RO biometric sign-off; Form 34B generation; transmission to national back-end; constituency summary printout.

**Non-functional:** Form 34B generation ≤ 10 s after last station received. P99 submission ≤ 2 s.

**Integration:** M08 station results feed. M10 national aggregation receives Form 34B. M51 result certification depends on M09 sign-off.

**User acceptance:** County RO persona: full aggregation and signing UAT. IAB observer: verification of signed Form 34B.

**Security acceptance:** Dual-custody sign-off verified. HSM countersigning of Form 34B verified.

**Regulatory:** Elections Act §39 compliance. ODPC DPIA (RO identity in signed event).

**Operational Acceptance:** 290 constituency simulations (290 ROs in parallel) in pre-production.

**Rollout:** OAT → National.

---

### M10 — RTS Backend

**Functional:** Receives Form 34A (from M08) and Form 34B (from M09). Validates signatures. Aggregates nationally. Publishes to M11 (Public Portal). Provides Merkle-rooted results with inclusion proofs. TLA+-verified state machine: no double-counting, no lost writes, monotonic aggregation.

**Non-functional:** 60,000 concurrent sessions sustained 30 min with P99 ≤ 100 ms. Merkle root published within 60 s of each constituency result received. Zero data loss under primary site failure.

**Integration:** M08/M09 (ingestion), M11 (publication), M14 (reproducibility bundle), M15 IAB Dashboard, M16 ECC, M21 RLA, M26 anomaly detection, M27 Intent Preservation, M40 court evidence, M51 certification, M69 archive.

**User acceptance:** IEBC national tally officer: aggregation monitoring UAT. IAB: parallel-tally divergence detection UAT.

**Security acceptance:** TLA+ proof reviewed by independent formal-methods expert. Insider-simulation: parallel-tally drift detector catches simulated tamper. HSM countersigning on all aggregation events. Replay attack blocked.

**Regulatory:** Elections Act §39. Elections (Technology) Regulations 2017 stress + load test. ODPC DPIA.

**Operational Acceptance:** 60,000-concurrent-session load test. DR failover RTO ≤ 30 min. Merkle-root publication cadence verified.

**Rollout:** OAT → Election day.

---

### M11 — Public Results Portal

**Functional:** Static-first CDN delivery of signed result snapshots. Any voter can view results by station, constituency, county, national. Station-level Merkle-inclusion proof accessible. No write API on public surface. Signed snapshots pushed at each results update.

**Non-functional:** 5M unique visitors/hour sustained on CDN. Origin serves ≤ 1% of requests during peak (99% CDN cache hit rate). P95 page load ≤ 200 ms from Nairobi (CDN edge). DDoS test: origin survives total CDN bypass (serves degraded but signed static page).

**Integration:** M10 pushes signed snapshots. M14 reproducibility bundle referenced from portal. M13 Public Verifier App linked.

**User acceptance:** General public persona: results lookup by station. Civil society observer: Merkle-inclusion proof download and verification. Media: structured data feed (JSON-LD).

**Security acceptance:** Signed snapshot verification chain. No PII in any public API response. CSP strict. SRI on all assets. DDoS simulation: origin unaffected; CDN serves static content.

**Regulatory:** Elections Act §39 public notification requirement. Access to Information Act 2016 (results are public data). No ODPC DPIA required (no personal data).

**Operational Acceptance:** 5M visitor load test. CDN failover to origin verified (signed static served). 99.9% availability over 30 days.

**Rollout:** Live from T-48 hours (portal skeleton) → Results published from T+0.

---

### M12 — CRMS (Candidate Registration Management System)

**Functional:** Party official submits candidate nomination. IEBC nomination desk reviews. Ballot proof generated (candidate order per Elections Act §34 randomisation). Nomination fee payment integration (M47). Invalidation workflow. Cleared nominations exported to M39 Ballot Printing.

**Non-functional:** P99 submission ≤ 2 s. 15,000 nominations processed over 30-day nomination window. Audit log queryable by IEBC legal team within 5 min of request.

**Integration:** M45 Party Registration (party eligibility check), M47 Nomination Fee, M39 Ballot Printing Chain, M22 Campaign Finance (candidate finance declarations), M46 Party Primaries.

**User acceptance:** Party official persona: nomination submission UAT. IEBC nomination desk: review and clearance UAT. IEBC legal team: audit log query.

**Security acceptance:** IDOR between party accounts blocked. Ballot proof randomisation verified (chi-squared test on 1,000 draws). Fee payment confirmed before nomination clearance.

**Regulatory:** Elections Act §24-§27 compliance. PPADA 2015 (nomination fee handling). ODPC DPIA.

**Operational Acceptance:** Full nomination window simulation (15,000 nominations). Ballot proof generation verified reproducible (M14).

**Rollout:** Open at T-180 (nomination period opens).

---

### M13 — Public Verifier App

**Functional:** Citizen inputs station code → downloads signed result → verifies Merkle-inclusion proof locally (on-device, no server round-trip for verification) → displays verified result with colour-coded confidence. Works on 3G (feature phone optimised). Offline verification mode: cached result verifiable without network.

**Non-functional:** App size ≤ 15 MB APK. Verification P99 ≤ 2 s on 3G (500 kbps). Feature-phone web fallback (USSD or low-bandwidth site). Accessibility: WCAG 2.2 AA.

**Integration:** M11 Public Portal (result download). M14 Reproducibility Bundle (root hash reference).

**User acceptance:** Low-tech voter persona: USSD fallback verification. Smartphone voter: full app verification. International observer: independent cryptographic verification.

**Security acceptance:** Code-signing via HSM (K1). Certificate pinning. Root/jailbreak detection. Spoofed result injection attempt blocked (signature verification fails).

**Regulatory:** Access to Information Act 2016. Elections Act §39 public verification right. KEBS Play Store submission compliance check.

**Operational Acceptance:** 1M verification requests/hour load test (via CDN). App store listing live on Google Play + Kenya-specific APK distribution site.

**Rollout:** App published T-30 (before election day). Updated immediately after results publication.

---

### M14 — Reproducibility Bundle

**Functional:** Any observer can download the bundle, run the build script, and reproduce the national tally from signed Form 34A images to national totals in ≤ 4 hours on a standard laptop (16 GB RAM, 8-core). Bundle includes: signed source image hashes, tally aggregation scripts, verification scripts, SBOM, dependency lock files.

**Non-functional:** Bundle size ≤ 10 GB. Three consecutive runs from same bundle produce identical national tally output. Script runs on Ubuntu 22.04 LTS and macOS 14 (Sonoma) without proprietary dependencies.

**Integration:** M10 provides signed source events. M11 references bundle. M21 RLA runs from bundle outputs.

**User acceptance:** Civil society observer: end-to-end bundle run on observer's own hardware. Auditor-General's office: bundle run verified independently.

**Security acceptance:** SBOM clean (no critical CVEs in bundle dependencies). Cosign signature on bundle verified. Source manifest hash matches deployed artefact.

**Regulatory:** Elections Act §39 (public accountability). Access to Information Act 2016.

**Operational Acceptance:** Bundle published within 24 hours of polls closing. IAB observer completes independent verification run before results are officially certified.

**Rollout:** Published at T+0 (same time as M11 results).

---

### M15 — IAB Dashboard

**Functional:** Real-time view of all transparency-log events, Merkle roots, high-severity incidents (anonymised), kit health, tally progress, anomaly alerts. Read-only. IAB members can acknowledge alerts and attach notes. Export to PDF for governance record.

**Non-functional:** Dashboard refresh ≤ 5 s. All anomaly alerts from M26 visible within 30 s of detection. FIDO2 login ≤ 10 s.

**Integration:** M26 (anomaly alerts), M10 (tally progress), M16 (ECC events), M28 (AI monitoring alerts), Transparency log (Merkle roots).

**User acceptance:** IAB Chair persona: alert review and acknowledgement UAT. IAB technical expert: Merkle root verification UAT.

**Security acceptance:** FIDO2 mandatory. Session recording for all IAB actions. Read-only replica (no write to production).

**Regulatory:** IEBC Act 2011 (IAB mandate). ISO 27001 access control.

**Operational Acceptance:** IAB full membership onboarded and trained before T-30. 30-day pre-election live access with real telemetry.

**Rollout:** Live T-30.

---

### M16 — Election Command Centre (ECC)

**Functional:** Real-time operational dashboard for IEBC national + county managers. Kit health monitoring (battery, connectivity, queue depth). Escalation workflows. Incident tracking. Resource dispatch (integration with M33). Station problem log. Automated alerts from M26 with manual disposition.

**Non-functional:** Dashboard refresh ≤ 5 s for 55,393 station feeds simultaneously. Alert-to-analyst notification ≤ 30 s. Incident ticket SLA tracking.

**Integration:** M01, M05, M08 (kit telemetry), M20 (inventory), M26 (anomaly alerts), M28 (AI alerts), M33 (dispatch), M56 (county coordination), M60 (medical first response).

**User acceptance:** IEBC national operations manager: dashboard monitoring UAT. County operations officer: county-level view UAT. Field support engineer: incident ticket workflow UAT.

**Security acceptance:** FIDO2 + RBAC verified (county officer sees only their county). Session recording for privileged actions. Insider-simulation: no single ECC account can modify results.

**Regulatory:** Elections Act §44A (failure declaration trigger visible in ECC). IEBC operational governance.

**Operational Acceptance:** Full dress rehearsal T-60: 55,393 simulated stations, 500 concurrent ECC analysts, all escalation workflows exercised.

**Rollout:** Live T-30 (war room open).

---

### M17 — Speech Integrity

**Functional:** Real-time monitoring of public utterances (broadcast + social) for deepfake audio/video, hate speech (National Cohesion and Integration Act 2008 categories), election disinformation. Alert to IEBC communications team. No automated removal — analyst-reviewed.

**Non-functional:** Detection latency ≤ 5 min from public utterance to alert. False positive rate ≤ 5% (analyst review target). Model accuracy ≥ 90% on IEBC test set for deepfake detection.

**Integration:** M28 AI Monitoring Stack (shared signal). M16 ECC (escalation for P1 integrity events).

**User acceptance:** IEBC communications officer: alert review and disposition UAT. Legal team: evidence export for NCIC referral.

**Security acceptance:** No PII of private individuals in detection database. Public figures only in targeted monitoring. ODPC DPIA. No automated action — all escalations human-reviewed.

**Regulatory:** National Cohesion and Integration Act 2008 monitoring obligation. ODPC DPIA for processing public utterances.

**Operational Acceptance:** 30-day pre-election live monitoring run. Analyst team onboarded and trained.

**Rollout:** Live T-60.

---

### M18 — Boundary Transparency

**Functional:** Interactive public map of constituency and ward boundaries. GeoJSON download. Historical boundary version comparison. Changes logged with legal gazette reference.

**Non-functional:** Map load ≤ 3 s on 3G. GeoJSON download ≤ 2 s for any constituency. 99.9% availability.

**Integration:** M12 CRMS (constituency assignment for nominations). M52 Election Calendar (boundary gazette events).

**User acceptance:** Civil society researcher: boundary map download and comparison. Party official: constituency boundary verification for nomination.

**Security acceptance:** No personal data. Boundary data signed with ECDSA. Tampering with boundary data raises SIEM alert.

**Regulatory:** Boundaries drawn per Constitution (Art. 89 boundary review). Access to Information Act 2016.

**Operational Acceptance:** Full 290-constituency boundary set published. Independent legal review confirms gazette reference accuracy.

**Rollout:** Published at T-365 (one year pre-election).

---

### M19 — Queue Transparency

**Functional:** Real-time public view of estimated queue length per station during election day. Station-level counts pushed from M05/M08 kits. No individual voter data. Aggregated 10-minute intervals.

**Non-functional:** Data latency ≤ 10 min. 99.9% availability. CDN-served for public access.

**Integration:** M05 BVI authentication counts (aggregate). M16 ECC (station operational status).

**User acceptance:** Voter persona: pre-planning tool to choose optimal queue time. Media persona: data download for reporting.

**Security acceptance:** No PII in any response. Rate limiting on API. Tampered counts flagged by M26 anomaly detector.

**Operational Acceptance:** Election-day simulation with 55,393 stations reporting.

**Rollout:** Live election day only (T-0 to T+0 close of polls).

---

### M20 — Inventory Management

**Functional:** Full lifecycle asset management for ~65,287 KIEMS kits: registration, provisioning, dispatch, field tracking (GPS + RFID), seal-break detection, return, refurbishment, decommission. Chain-of-custody signed at each transfer. Smart-lid ballot-box RFID integration.

**Non-functional:** Full inventory scan ≤ 30 min. Real-time GPS refresh ≤ 5 min during election window. Seal-break alert to M16 ECC ≤ 60 s.

**Integration:** M01, M05, M08 (kit identity verification), M16 ECC (seal-break alerts), M32 Field Support (dispatch coordination), M39 Ballot Printing (ballot-box RFID chip handoff), M63 Ballot Serial Chain.

**User acceptance:** IEBC logistics manager: dispatch and return workflow UAT. Field engineer: kit handoff with RFID scan UAT. County store manager: inventory report.

**Security acceptance:** Chain-of-custody signature verification. RFID tamper-evident logging verified. Stolen kit honey mode: revoked kit receives sandbox, captures attacker biometrics.

**Regulatory:** PPADA 2015 (asset management). IEBC asset registry obligations.

**Operational Acceptance:** Full 65,287-kit inventory simulation. Real-time tracking verified in field trial.

**Rollout:** Operational T-365 (kit procurement starts T-540).

---

### M21 — RLA Audit Tool (Risk-Limiting Audit)

**Functional:** Statistical sample draw from M14 reproducibility bundle output. Ballot manifest comparison. Per-batch risk calculation. Auditor attestation workflow. Output: risk bound α ≤ 0.05 for declared winner confirms outcome at stated significance.

**Non-functional:** Sample draw completed ≤ 30 min after M14 bundle available. Risk calculation: real-time as ballots sampled. Supports ballot-polling and ballot-comparison audits.

**Integration:** M14 Reproducibility Bundle (source data). M10 RTS Backend (signed result cross-check). M63 Ballot Serial Chain (ballot manifest).

**User acceptance:** Auditor-General's office: sample draw and risk calculation UAT. IAB technical expert: RLA methodology review.

**Security acceptance:** No write access to production results. Audit sample drawn from cryptographically verified manifest. Auditor identity and sample draw logged on transparency log.

**Regulatory:** Elections Act §83 (IEBC certification of results). International best practice (VVSG 2.0 reference).

**Operational Acceptance:** RLA run on synthetic election with known result; outcome confirmed at α ≤ 0.05.

**Rollout:** Available from T+0 (ballot-scanning begins). Final RLA report before M51 certification.

---

### M22 — Campaign Finance Transparency

**Functional:** Party and candidate campaign finance reporting (income + expenditure). Limit enforcement per Election Campaign Financing Act 2013. Public disclosure portal. PPADA compliance for procurement-related expenditure.

**Non-functional:** P99 submission ≤ 2 s. Public portal refresh ≤ 5 min. 99.9% availability.

**Integration:** M12 CRMS (candidate eligibility). M66 Election Financial Forensics (anomaly escalation).

**User acceptance:** Party finance officer: income/expenditure submission UAT. IEBC finance compliance officer: limit breach alert UAT. Civil society researcher: public portal download.

**Security acceptance:** Party finance data isolated by party (IDOR prevention). Limit breach alert fires correctly.

**Regulatory:** Election Campaign Financing Act 2013 full compliance. PPADA 2015. POCAMLA 2009 (money laundering red flags to M66).

**Operational Acceptance:** Campaign window simulation (6-month finance period). All statutory reporting deadlines enforced.

**Rollout:** Open at T-365 (campaign period).

---

### M23 — Grievance Channels

**Functional:** Multi-channel grievance intake (web, SMS, USSD, in-person form). Anonymous or identified submission. Workflow routing by grievance type (electoral, administrative, cybersecurity). SLA tracking. Escalation to M16 ECC for urgent operational grievances. Status updates to complainant.

**Non-functional:** P99 submission ≤ 3 s. SMS/USSD intake ≤ 60 s end-to-end. 99.9% availability. 10,000 submissions/day capacity.

**Integration:** M16 ECC (urgent escalation). M26 Anomaly Detection (pattern detection on grievance volume). M40 Court Interface (grievance as petition evidence).

**User acceptance:** Voter complainant persona: anonymous web submission UAT. Field officer: in-person intake form UAT. IEBC grievance officer: triage and routing UAT.

**Security acceptance:** Anonymous submission stores no identifying data if anonymous path chosen. Complainant identity separated from complaint content (encrypted separately). DPIA on file.

**Regulatory:** Elections Act §74 (election complaints). DPA 2019 s.30(1)(a) consent for identified submissions. ODPC DPIA.

**Operational Acceptance:** 10,000-submission load test. SMS/USSD fallback end-to-end test.

**Rollout:** Open T-90.

---

### M24 — Diaspora Voting

**Functional:** Diaspora voter registration, identity verification (NID + biometric via embassy partners), ballot issuance and secure online submission (for electronic diaspora vote where legally permitted) or physical ballot tracking (M65). Results integrated into national tally via M10.

**Non-functional:** P99 ballot submission ≤ 3 s. 50,000 registered diaspora voters supported. Ballot secrecy: no linkage between voter identity and ballot choice stored.

**Integration:** M35 CVR (diaspora voter roll), M65 Physical Logistics (paper ballot track), M10 RTS Backend (online results ingestion), M03 ABIS (identity verification).

**User acceptance:** Diaspora voter persona: full registration-to-ballot-submission UAT. Embassy officer: identity verification workflow UAT.

**Security acceptance:** Ballot secrecy verified (no voter-choice linkage in database). Double-vote prevention verified. Embassy system mTLS integration tested.

**Regulatory:** Elections Act §52 (diaspora voting provisions). ODPC DPIA. DPA 2019 data transfer to embassy systems (ODPC authorisation obtained).

**Operational Acceptance:** Diaspora simulation (10,000 votes) with M10 integration.

**Rollout:** Registration open T-180.

---

### M25 — Special Voter Workflows

**Functional:** Assisted voting for PWD voters. Hospital/home voting coordination. Preferential queue management. Language support for additional Kenyan languages beyond EN/SW. Large-print and audio guidance on kit. Mobility-impaired voter flow.

**Non-functional:** Disability-status data encrypted with field-level key. No disability data exposed to poll clerk beyond "special assistance required" flag. Audio guidance ≤ 5 s lag behind visual prompt.

**Integration:** M01, M05 (kit accessibility modes), M16 ECC (special voting coordination), M38 Training LMS (poll worker training on special workflows).

**User acceptance:** PWD voter persona: accessibility workflow UAT with representative from disability rights organisation. IEBC presiding officer: special voting form completion UAT.

**Security acceptance:** Disability data (special-category under DPA s.29) field-encrypted with separate DEK. DPIA on file. No disclosure beyond operational necessity.

**Regulatory:** Persons with Disabilities Act 2003. DPA 2019 s.29 (special-category). Elections Act §44 (PWD assistance provisions).

**Operational Acceptance:** Full range of PWD workflow scenarios tested with disability advocates.

**Rollout:** Live election day.

---

### M26 — Anomaly and Fraud Detection

**Functional:** Seven-layer detection (see SECURITY.md §10.3). Benford's Law analysis on tally results. Statistical outlier detection per station. Kit connectivity anomaly. Double-authentication attempt detection. Geographic clustering analysis. Real-time alert routing to M16 ECC + M15 IAB Dashboard. ML models trained on synthetic election data only.

**Non-functional:** Alert latency ≤ 30 s from anomaly to SOC notification. False positive rate ≤ 2% (tuned on synthetic election data). Model retraining pipeline runs weekly.

**Integration:** M05, M08 (authentication and transmission events), M10 (tally stream), M16 ECC (alert routing), M15 IAB, M28 AI Monitoring, M26 shares signals with M61 (secrecy testing).

**User acceptance:** SOC analyst persona: alert triage workflow UAT. IEBC statistical officer: Benford analysis report UAT.

**Security acceptance:** Model inputs are pseudonymised. No live voter PII in model training. Model-poisoning attack simulation (adversarial inputs) tested.

**Regulatory:** ODPC DPIA (pseudonymised data only). No individual voter data processed by detection models.

**Operational Acceptance:** Full election-day simulation with injected anomalies; all anomalies detected within SLA.

**Rollout:** Live T-30 (pre-election monitoring begins).

---

### M27 — Intent Preservation

**Functional:** Seals Form 34A image with ECDSA signature from kit BEFORE transmission. Any modification after seal is detectable. Offline-first: seal happens locally on kit; submission to M08/M10 carries sealed form + original image. Court can verify seal using kit public key from PKI.

**Non-functional:** Seal operation ≤ 500 ms. Sealed form + original image stored locally for 72 hours minimum (offline resilience). Zero silent failures: any seal error surfaces immediately to presiding officer.

**Integration:** M08 RTS Device (form seal before transmission), M10 RTS Backend (seal verified on receipt), M14 Reproducibility Bundle (sealed forms included), M40 Court Interface (sealed form as primary evidence).

**User acceptance:** Presiding officer persona: seal-and-transmit UAT. Independent observer: verify seal on received form.

**Security acceptance:** Seal modification attempt detected (SSIM + signature verification). Replay of earlier sealed form rejected (monotonic counter). Offline seal operates without back-end connectivity.

**Regulatory:** Elections Act §39 (results integrity). Court petition evidence requirement (Art. 87).

**Operational Acceptance:** 60,000 concurrent seal operations sustained without performance degradation.

**Rollout:** Kit firmware update delivered T-30 (with M08 integration).

---

### M28 — AI Monitoring Stack

**Functional:** Monitors social media (Twitter/X, Facebook, TikTok Kenya) and broadcast media for AI-generated disinformation, deepfake content, clone-app distribution, lookalike domain registration. Feeds alerts to M17 and M16. No automated content removal. Compliance monitoring for Kenya-jurisdiction AI processing.

**Non-functional:** Monitoring latency ≤ 5 min. Model inference on Kenya-jurisdiction servers only. No voter PII in any model input.

**Integration:** M17 Speech Integrity (deepfake signals), M16 ECC (alert routing), M61 Secrecy Assurance (clone-app signals).

**User acceptance:** IEBC communications officer: alert triage UAT. NCIC compliance: hate speech escalation workflow UAT.

**Security acceptance:** All AI inference on Kenya-jurisdiction servers (DPA 2019 s.18). No biometric data in AI models. Model cards published per CCB requirement.

**Regulatory:** ODPC DPIA. National Cohesion and Integration Act 2008. Computer Misuse and Cybercrimes Act 2018.

**Operational Acceptance:** 30-day pre-election live monitoring verified with known test signals.

**Rollout:** Live T-90 (campaign monitoring begins).

---

### M29 — Mobile Citizen Verify

**Functional:** Feature-phone-compatible app and web (USSD fallback) allowing any citizen to verify their voter registration status and assigned polling station. QR code scan of station result (wraps M13 functionality for mobile).

**Non-functional:** App size ≤ 10 MB. USSD flow ≤ 5 steps. P99 lookup ≤ 2 s on 3G. WCAG 2.2 AA.

**Integration:** M07 BVV (registration lookup), M13 Public Verifier (result verification), M11 Public Portal.

**User acceptance:** First-time voter persona: registration lookup via USSD. Smartphone voter: app verification.

**Security acceptance:** Code-signing via HSM (K1). Certificate pinning. Root/jailbreak detection. Rate limiting (10 lookups/phone/day). No PII in logs.

**Regulatory:** ODPC DPIA (voter NID self-lookup is DPA s.30(1)(a) consent). App store compliance.

**Operational Acceptance:** 1M daily active users load test. App store listing verified.

**Rollout:** App published T-30.

---

### M30 — Mobile Party Agent

**Functional:** Party agent accreditation token at station. Real-time aggregate tally view (constituency level, not station-level voter choices). Form 34A observation record. Dispute flag submission to M23 Grievance.

**Non-functional:** Aggregate tally refresh ≤ 1 min. App size ≤ 20 MB. No individual voter data accessible.

**Integration:** M10 RTS Backend (aggregate feed), M23 Grievance (dispute submission), M43 Observer Accreditation (agent credentials).

**User acceptance:** Party agent persona: station monitoring UAT. Party coordinator: aggregated view UAT.

**Security acceptance:** No individual voter data accessible. No way to infer voter choices from aggregate. Screen capture prevention on sensitive screens.

**Regulatory:** Elections Act §61 (party agent rights). DPA 2019 (no PII in app).

**Operational Acceptance:** 15,000 concurrent party agents load test.

**Rollout:** Accreditation open T-30.

---

### M31 — Mobile Poll Clerk

**Functional:** Poll clerk check-in, station setup checklist, real-time kit pairing, attendance log, incident reporting, end-of-day checklist. No biometric data on mobile.

**Non-functional:** Kit pairing ≤ 30 s. App size ≤ 15 MB. Offline mode for checklist.

**Integration:** M05 BVI (kit pairing), M16 ECC (incident reporting), M38 Training LMS (training completion verification).

**User acceptance:** Poll clerk persona: full election-day checklist UAT.

**Security acceptance:** OIDC + MFA. No biometric data on mobile. Screen capture prevention.

**Regulatory:** Elections Act §44 (poll clerk duties). DPA 2019.

**Operational Acceptance:** 55,393 concurrent clerks load test (poll opening simulation).

**Rollout:** App distributed T-14 (clerk training).

---

### M32 — Mobile Field Support

**Functional:** Field engineer kit-swap workflow, incident ticket management, kit diagnostic commands, real-time inventory update (M20 integration), geo-tracked dispatch.

**Non-functional:** Kit swap workflow ≤ 10 min end-to-end. Offline mode for areas without connectivity.

**Integration:** M20 Inventory, M16 ECC (ticket routing), M33 Personnel Dispatch.

**User acceptance:** Field engineer persona: kit swap UAT. County logistics coordinator: team view UAT.

**Security acceptance:** OIDC + MFA. Location data retained ≤ 7 days. Chain-of-custody log for kit swap.

**Operational Acceptance:** 500 concurrent field engineers load test.

**Rollout:** App distributed T-30 (engineer training).

---

### M33 — Mobile Personnel Dispatch

**Functional:** Real-time dispatch of IEBC staff and field engineers. Task assignment, GPS tracking (with consent), ETA calculation, completion confirmation.

**Non-functional:** Dispatch-to-confirmation ≤ 2 min. GPS refresh ≤ 2 min. 500 concurrent staff supported.

**Integration:** M16 ECC (task origination), M32 Field Support, M60 Medical First Response.

**User acceptance:** Dispatch coordinator persona: task assignment UAT. Field staff: task receipt and completion UAT.

**Security acceptance:** Location data minimised (station-level after dispatch complete). Staff consent obtained for GPS tracking. Data deleted 7 days post-election.

**Regulatory:** DPA 2019 s.30(1)(b) for location tracking during election duty.

**Operational Acceptance:** 500 concurrent staff tracking test.

**Rollout:** Live T-30.

---

### M34 — Mobile Observer

**Functional:** Observer accreditation token. Station observation form submission. Photo evidence upload (anonymised). Result observation record. Integration with M43 accreditation.

**Non-functional:** Form submission ≤ 3 s. Photo upload ≤ 30 s on 3G.

**Integration:** M43 Observer Accreditation, M16 ECC (urgent observer reports), M23 Grievance (observer complaints).

**User acceptance:** International observer persona: observation form submission UAT.

**Security acceptance:** Observer identity verified at accreditation stage. No voter PII accessible. Photo metadata stripped before storage.

**Regulatory:** Elections Act §73 (observer rights). DPA 2019.

**Operational Acceptance:** 5,000 concurrent observers load test.

**Rollout:** Accreditation open T-60.

---

### M35 — CVR (Continuous Voter Register)

**Functional:** Continuous registration intake (between elections), bulk BVR import from M02, deregistration appeals (M36), transfer between constituencies, public inspection period (M64), register lock at cut-off (Elections Act §6A).

**Non-functional:** 30M record capacity. Register lock enforced exactly at cut-off (±1 second). Bulk import 100,000 records/hour. P99 individual lookup ≤ 500 ms.

**Integration:** M01/M02 BVR (intake), M04 VPRS (pre-registration merge), M05/M06 BVI (voter roll delta sync), M07 BVV (public lookup), M36 Appeals, M64 Public Inspection.

**User acceptance:** IEBC registration officer: individual and bulk registration UAT. IEBC DPO: DSAR workflow. County election official: constituency roll export.

**Security acceptance:** Register lock mechanism tested (no writes accepted post-cut-off). OPA separation-of-duties verified. HSM-signed audit log of every change.

**Regulatory:** Elections Act §5, §6, §6A, §6B compliance. DPA 2019 full mapping. ODPC DPIA.

**Operational Acceptance:** 30M-record load test. Register lock test. Full DR failover.

**Rollout:** Continuous operation; lock at statutory cut-off.

---

### M36 — Voter Deregistration Appeal

**Functional:** Voter challenges exclusion from register. IEBC review workflow. Dual approval for deregistration action. Appeal outcome notification. Audit trail for Elections Court.

**Non-functional:** P99 submission ≤ 2 s. Dual-approval workflow enforced before any deregistration.

**Integration:** M35 CVR (register update), M40 Court Interface (appeal evidence), M23 Grievance (escalation pathway).

**User acceptance:** Voter appellant persona: appeal submission UAT. IEBC registrar: review and disposition UAT.

**Security acceptance:** Dual approval enforced. Appeal record immutable after submission. DPIA on file.

**Regulatory:** Elections Act §22-§23 (registration disputes). DPA 2019.

**Operational Acceptance:** 1,000-appeal simulation.

---

### M37 — Voter Education CMS

**Functional:** Manage voter education content (video, text, infographic) in EN, SW, and additional Kenyan languages. Publish to public website and M68 platform. Content versioning. Mobile-first, low-bandwidth optimised.

**Non-functional:** Content delivery P99 ≤ 2 s on 3G. 99.9% availability. Support 10 Kenyan languages.

**Integration:** M68 Referendum Explainer (content sharing), M38 Training LMS (linked materials).

**User acceptance:** IEBC communications officer: content publishing UAT. Low-literacy voter persona: usability test in target language.

**Security acceptance:** No PII. CSP strict. HTTPS only. Content signing (ECDSA) to prevent defacement.

**Regulatory:** Elections Act (IEBC public education mandate). ODPC (no PII).

**Operational Acceptance:** Full language set published. Mobile usability test passed.

---

### M38 — Poll Worker Training LMS

**Functional:** Training modules for poll workers (video, quiz, simulation). Completion tracking. Certificate issuance. 130,000 temporary poll workers supported. Offline module download for areas without connectivity.

**Non-functional:** 130,000 concurrent users. P99 video load ≤ 5 s on 3G. Offline quiz operation.

**Integration:** M31 Mobile Poll Clerk (training completion verification), M55 HR Payroll (training completion gates payment).

**User acceptance:** Poll worker persona: full training module UAT (EN + SW). IEBC training manager: completion report UAT.

**Security acceptance:** OIDC for all trainees. Completion records signed. No biometric data in training environment.

**Regulatory:** IEBC training mandate. DPA 2019 (poll worker personal data).

**Operational Acceptance:** 130,000-user load test. Offline mode verified.

**Rollout:** Open T-90.

---

### M39 — Ballot Printing Chain

**Functional:** Generates ballot proof (candidate order per M12 CRMS output, randomised per Elections Act §34). Issues ballot serial numbers to M63. Sends print-ready specification to contracted printer. Receives print completion confirmation with serial range. Integrates ballot-box RFID chip assignment.

**Non-functional:** Ballot proof generation ≤ 60 s per constituency. Serial range issued deterministically from M63. Print specification signed before transmission to printer.

**Integration:** M12 CRMS (candidate list), M63 Ballot Serial Chain (serial issuance), M20 Inventory (RFID chip assignment), M67 Party Symbol Registry (symbols on ballot).

**User acceptance:** IEBC print officer: ballot proof review and sign-off UAT. Printer liaison: print specification receipt confirmation.

**Security acceptance:** Ballot proof randomisation verified (chi-squared). Print spec signed with ECDSA; printer must verify before printing. Serial range recorded in M63 before printing begins.

**Regulatory:** Elections Act §34 (candidate order randomisation). PPADA 2015 (printer contract governance).

**Operational Acceptance:** All 290 constituencies' ballot proofs generated and signed. Printer API integration tested.

**Rollout:** T-60 (ballot printing begins).

---

### M40 — Election Court IT Interface

**Functional:** Evidence bundle generation on demand (triggered by court order or IEBC legal team). Bundle includes: signed Form 34A images (from M27), Merkle-inclusion proofs (from M10/M14), kit chain-of-custody records (from M20), ballot serial chain (from M63), authentication audit logs (from M06). Chain-of-custody record for every evidence package. Expert witness affidavit template. Response: ≤ 24 hours from request (Elections Act §75(1)).

**Non-functional:** Bundle generation ≤ 4 hours. Bundle integrity verified before delivery. Access logged with dual-custody.

**Integration:** M14 (bundle source), M20 (kit chain-of-custody), M27 (sealed forms), M06 (auth logs), M63 (ballot serial), M10 (Merkle proofs), M69 (long-term archive access).

**User acceptance:** IEBC legal team: evidence bundle generation UAT. Auditor-General's office: bundle verification UAT.

**Security acceptance:** Dual-custody access for evidence bundle generation. Chain-of-custody record immutable. Court-order verification before bundle generation.

**Regulatory:** Elections Act §75(1) (petition evidence, 24-hour SLA). Constitution Art. 87 (petition right). Kenya Evidence Act (digital evidence admissibility).

**Operational Acceptance:** Dry run: IEBC legal team + independent lawyer verify bundle at T-14. SC IT Registrar dry-run passed (per programme-level acceptance criteria §4).

**Rollout:** Available from election day; priority escalation path active T-0 through T+7 (petition window).

---

### M41 — Post-Election Review

**Functional:** Structured debrief tool for capturing lessons learned per module, per county, per station category. Recommendations tracking. Feeds M41 report to IAB and Auditor-General.

**Non-functional:** Full system review complete within T+30.

**Integration:** M59 Auditor General Interface (report delivery). M16 ECC (incident log review). M26 Anomaly Detection (alert review).

**User acceptance:** IEBC operations manager: review entry UAT. IAB: review approval UAT.

**Security acceptance:** Read-only access to production event logs. No write to production data.

**Operational Acceptance:** Post-election report produced within T+30 and signed by IEBC.

---

### M42 — Media Accreditation Portal

**Functional:** Application and credential management for media organisations. Accreditation badge issuance (QR-code-signed). Station access tier management. Revocation workflow.

**Non-functional:** P99 application submission ≤ 2 s. Badge generation ≤ 5 s.

**Integration:** M16 ECC (revocation of disruptive media), M57 International Observer Coordination (shared accreditation infrastructure).

**User acceptance:** Journalist persona: application and badge receipt UAT. IEBC press officer: review and approval UAT.

**Security acceptance:** Identity verification step (government-issued press card). Badge QR signed and verifiable offline. DPIA on file for journalist PII.

**Regulatory:** Elections Act §73 (media access). ODPC DPIA.

**Operational Acceptance:** 2,000 accredited media representatives load test.

**Rollout:** Open T-90.

---

### M43 — Observer Accreditation Portal

**Functional:** Application and credential management for domestic and international election observers. Organisational endorsement workflow. Accreditation badge (QR-signed). Station assignment. Real-time revocation.

**Non-functional:** P99 application ≤ 2 s. Badge generation ≤ 5 s. 10,000 observers supported.

**Integration:** M34 Mobile Observer (credential token), M57 International Observer Coordination, M42 Media Accreditation (shared infrastructure).

**User acceptance:** Observer organisation persona: application UAT. IEBC accreditation officer: review and approval UAT.

**Security acceptance:** Identity verification at application. Badge signed and verifiable offline. Revocation propagates to M34 ≤ 60 s.

**Regulatory:** Elections Act §73. ODPC DPIA. Bilateral agreements for international observers.

**Operational Acceptance:** 10,000 concurrent observers tested.

**Rollout:** Open T-90.

---

### M44 — Turnout Forecasting Analytics

**Functional:** Real-time station-level turnout aggregation and forecasting. Predictive model for projected national turnout at close of polls. Feeds M16 ECC and M15 IAB Dashboard. No individual voter data in model.

**Non-functional:** Forecast refresh ≤ 5 min. Model P90 accuracy within ±5% of actual turnout (validated on synthetic historical data).

**Integration:** M05 BVI (authentication counts, pseudonymised), M16 ECC (alert for anomalously low turnout stations), M15 IAB.

**User acceptance:** IEBC operations manager: turnout forecast view UAT.

**Security acceptance:** No PII in any model input or output. Pseudonymised station counts only.

**Operational Acceptance:** Full election-day simulation; forecast accuracy within target.

---

### M45 — Party Registration Management

**Functional:** Political party registration application. Membership threshold verification. Financial compliance check (Campaign Financing Act). Certificate of registration issuance. Deregistration workflow. Public registry.

**Non-functional:** P99 application submission ≤ 2 s. 100 registered parties capacity (well above Kenya's current number). Public registry refresh ≤ 5 min.

**Integration:** M12 CRMS (party eligibility for nominations), M22 Campaign Finance (party financial compliance), M67 Party Symbol Registry.

**User acceptance:** Party registrar persona: application submission UAT. IEBC registrar: review and certificate issuance UAT.

**Security acceptance:** OIDC + MFA for party officials. Party data isolated by party (IDOR prevention). Public registry read-only anonymous.

**Regulatory:** Political Parties Act 2011 full compliance. ODPC DPIA. PPADA 2015.

**Operational Acceptance:** 100-party simulation.

---

### M46 — Party Primaries Oversight

**Functional:** Party primary result submission by party. IEBC oversight dashboard. Dispute workflow. Integration with CRMS to confirm primary-cleared candidates.

**Non-functional:** P99 submission ≤ 2 s. 15,000 candidate primary results supported.

**Integration:** M12 CRMS (candidate clearance), M22 Campaign Finance (primary campaign finance).

**User acceptance:** Party officials: primary result submission UAT. IEBC oversight officer: dispute handling UAT.

**Security acceptance:** Results signed by party authorized signatory. OIDC + MFA. Immutable after IEBC acceptance.

**Regulatory:** Political Parties Act 2011 (primary oversight). ODPC DPIA.

**Operational Acceptance:** All party primaries tracked and cleared before nomination deadline.

---

### M47 — Nomination Fee Payment

**Functional:** Candidate pays statutory nomination fee via M-Pesa or bank transfer. Receipt generation. Fee confirmation triggers CRMS nomination clearance. Refund workflow for invalid nominations.

**Non-functional:** Payment confirmation ≤ 60 s (M-Pesa real-time). P99 receipt generation ≤ 5 s. PCI DSS compliance for card payments.

**Integration:** M12 CRMS (fee confirmation), M54 Budget Expenditure (fee income recording).

**User acceptance:** Candidate persona: M-Pesa payment UAT. IEBC finance officer: fee reconciliation UAT.

**Security acceptance:** PCI DSS compliance for any card processing path. No card numbers stored. Payment token integration only. M-Pesa API certificate pinning.

**Regulatory:** Elections Act §24 (nomination fee). PPADA 2015. PCI DSS.

**Operational Acceptance:** 15,000 payment transactions in nomination window simulation.

---

### M48 — Recall Election Workflow

**Functional:** Manages recall petition (County Assembly petition for MP recall per Constitution Art. 104). Signature collection, threshold verification, IEBC determination, recall election trigger.

**Non-functional:** P99 petition submission ≤ 2 s. Signature threshold verification ≤ 10 min.

**Integration:** M35 CVR (voter roll for signature verification), M49 Referendum Mechanics (recall election uses referendum infrastructure where applicable), M23 Grievance.

**User acceptance:** Petition coordinator: signature submission UAT. IEBC determination officer: threshold verification UAT.

**Security acceptance:** Petition signatories' NID stored encrypted. Signature verification auditable. DPIA on file.

**Regulatory:** Constitution Art. 104 (recall). Elections Act §45 (recall elections).

**Operational Acceptance:** Recall workflow tested end-to-end with synthetic constituency.

---

### M49 — Referendum Mechanics

**Functional:** Referendum-specific workflows: Yes/No ballot design, ballot distribution, station operation (re-uses M05, M08 infrastructure with referendum configuration), results transmission (re-uses M10), and results publication (re-uses M11).

**Non-functional:** Same SLAs as M05, M08, M10, M11. Referendum configuration applied without code changes to underlying modules (configuration-driven).

**Integration:** M05, M08, M10, M11 (re-use), M51 (referendum result certification), M49 has its own referendum-specific configuration layer.

**User acceptance:** IEBC referendum coordinator: end-to-end referendum simulation UAT. M68 Referendum Explainer linked and live.

**Security acceptance:** Same as M05, M08, M10. Ballot secrecy verified for Yes/No choice (M61 referee test).

**Regulatory:** Constitution Art. 255-257 (referendum). Elections Act §85. DPA 2019.

**Operational Acceptance:** Full referendum simulation (synthetic) before any gazetted referendum.

---

### M50 — By-Election Operations

**Functional:** Streamlined deployment of full election stack for single-constituency by-elections. Configuration-driven: activates relevant modules for affected constituency only. Faster deployment timeline (target: 45-day by-election cycle).

**Non-functional:** By-election stack deployment from code to live: ≤ 5 days (from configuration to station kit provisioning). SLAs same as general election for activated modules.

**Integration:** All election modules (activated subset). M20 Inventory (partial kit deployment).

**User acceptance:** IEBC by-election coordinator: deployment checklist UAT.

**Security acceptance:** Same as general election for activated modules. Software freeze for non-affected modules maintained.

**Regulatory:** Elections Act (by-election provisions). Full certification for activated modules.

**Operational Acceptance:** By-election simulation in one constituency.

---

### M51 — Result Declaration Certification

**Functional:** Presiding officer (Form 34A) and Returning Officer (Form 34B) digital certification chain. National tally certification by IEBC Chair (Form 34C equivalent). Each certification level requires biometric sign-off + FIDO2. Certification events HSM-countersigned and Merkle-chained. Trigger for M11 public publication.

**Non-functional:** Certification P99 ≤ 10 s. HSM-countersigned within 60 s. Merkle root published within 60 s.

**Integration:** M08 (Form 34A), M09 (Form 34B), M10 (national aggregation), M11 (publication trigger), M14 (bundle inclusion), M21 RLA (RLA must pass before national certification), M40 (court evidence).

**User acceptance:** IEBC Chair persona: national certification UAT. RO persona: constituency certification UAT.

**Security acceptance:** Dual-custody for national certification. HSM-countersigning verified. Merkle root published publicly immediately. Insider tamper simulation: parallel-tally drift detected before certification.

**Regulatory:** Elections Act §83 (IEBC certification). Constitution Art. 138 (presidential result proclamation). ODPC DPIA (officer identity in certification).

**Operational Acceptance:** Full certification chain exercised in dress rehearsal T-60.

---

### M52 — Election Calendar and Gazette

**Functional:** Manages statutory election calendar milestones. Generates gazette-ready exports (for Kenya Gazette publication). Public calendar with notification subscription. Gazette publication confirmation tracking.

**Non-functional:** Calendar events published ≤ 1 hour of IEBC decision. Gazette export generated ≤ 10 min.

**Integration:** M12 CRMS (nomination deadlines), M35 CVR (registration cut-off), M39 Ballot Printing (print deadline).

**User acceptance:** IEBC secretary: calendar management UAT. Kenya Gazette Office: export format verification.

**Security acceptance:** Calendar events signed (ECDSA). No PII. Tampering raises SIEM alert.

**Regulatory:** Elections Act (statutory calendar). Laws of Kenya Gazette publication.

**Operational Acceptance:** Full calendar populated for 2027 election cycle.

---

### M53 — Voter ID Card

**Functional:** Voter ID card issuance workflow using biometric data from M01/M02. Card design: QR code (signed voter ID), name, photo, station assignment. Issuance tracking in M35 CVR. Lost card replacement workflow.

**Non-functional:** Card generation ≤ 5 s. Batch print job ≤ 4 hours for 30M cards (distributed print).

**Integration:** M01/M02 BVR (source biometric data), M35 CVR (issuance record), M20 Inventory (card stock management).

**User acceptance:** IEBC card issuance officer: card generation UAT. Voter persona: QR scan verification of card.

**Security acceptance:** Card QR signed with ECDSA; tampering detectable. Photo biometric not transmitted to card printer (print-ready image only, no raw biometric template).

**Regulatory:** Elections Act §35 (voter ID). DPA 2019 (photo is personal data; lawful basis s.30(1)(b)).

**Operational Acceptance:** Batch generation of 30M synthetic cards verified within SLA.

---

### M54 — Election Budget Expenditure

**Functional:** Expenditure tracking against approved budget. Purchase order workflow. Payment approval (dual for large amounts). Vendor invoice management. Budget vs actual reporting. PPADA 2015 compliance.

**Non-functional:** P99 entry submission ≤ 2 s. Real-time budget utilisation view. Audit trail queryable.

**Integration:** M47 Nomination Fee (income). M55 HR Payroll (staff costs). M59 Auditor General Interface (expenditure reports).

**User acceptance:** IEBC finance manager: expenditure entry and approval UAT. Auditor-General's office: expenditure report UAT.

**Security acceptance:** Dual approval for expenditures > KES 1M. No single approver can approve and receive. Audit trail signed.

**Regulatory:** PPADA 2015. PFMA 2012. ODPC DPIA (vendor and staff PII in expenditure records).

**Operational Acceptance:** Full election-cycle financial simulation.

---

### M55 — HR and Payroll (Temporary Staff)

**Functional:** Temporary election staff management (130,000 poll workers, field engineers, etc.). Recruitment, contract issuance, training tracking (M38), payroll (M-Pesa disbursement), end-of-duty certificate.

**Non-functional:** 130,000 staff records. Payroll run ≤ 4 hours for full staff. M-Pesa disbursement confirmation ≤ 60 s per payment.

**Integration:** M38 Training LMS (completion gates), M33 Personnel Dispatch (assignment records), M54 Budget Expenditure (payroll costs).

**User acceptance:** IEBC HR manager: recruitment and payroll UAT. Poll worker: payroll receipt verification.

**Security acceptance:** Bank/M-Pesa details field-encrypted with separate DEK. Staff NID encrypted. Payroll access dual-role approval. DPIA on file.

**Regulatory:** DPA 2019 (employee data). Employment Act (Kenya). ODPC DPIA.

**Operational Acceptance:** 130,000-staff simulation. Payroll run within SLA.

---

### M56 — County-National Coordination

**Functional:** Secure messaging and coordination between county election offices and national HQ. Incident escalation. Resource request workflow. Situation report (SITREP) tools.

**Non-functional:** Message delivery ≤ 5 s. 290 county simultaneous connections. Offline message queue.

**Integration:** M16 ECC (escalation), M33 Personnel Dispatch (resource requests), M60 Medical First Response (emergency coordination).

**User acceptance:** County RO persona: situation report UAT. National operations officer: aggregated county view UAT.

**Security acceptance:** OIDC + MFA. Messages encrypted in transit (mTLS) and at rest. County officer can only see their county.

**Regulatory:** DPA 2019 (officer identity in messages). Elections Act operational governance.

**Operational Acceptance:** 290-county simulation with concurrent SITREPs.

---

### M57 — International Observer Coordination

**Functional:** Observer briefing portal (accreditation-gated). Station assignment management. Real-time situation updates to observer group leaders. Observation report aggregation.

**Non-functional:** P99 briefing document load ≤ 2 s. 1,000 international observers supported.

**Integration:** M43 Observer Accreditation (credential validation), M34 Mobile Observer (mobile reports), M16 ECC (urgent observer escalation).

**User acceptance:** EU EOM lead observer persona: briefing access and report submission UAT. African Union observer: accreditation to report workflow.

**Security acceptance:** OIDC + MFA. Briefing documents signed. Observer identity verified. DPIA for international data transfer (ODPC authorisation for observer data to home countries).

**Regulatory:** Elections Act §73 (observer rights). DPA 2019 s.18 (international data transfer). Bilateral observer agreements.

**Operational Acceptance:** 1,000-observer simulation.

---

### M58 — Parliamentary and Judiciary Reporting

**Functional:** Formal result reporting to National Assembly, Senate, and Judiciary. Signed report generation. Transmittal record. Statutory deadlines enforced (Elections Act §83).

**Non-functional:** Report generation ≤ 30 min. Signed transmission confirmation ≤ 1 hour of certification.

**Integration:** M51 Result Certification (source data), M10 RTS Backend (detailed results), M59 Auditor General Interface (shared report format).

**User acceptance:** IEBC secretary: report generation and transmission UAT. Clerk of the National Assembly: receipt confirmation.

**Security acceptance:** Reports signed with ECDSA. Chain of custody for delivery. Transmission acknowledgement signed.

**Regulatory:** Elections Act §83. Constitution Art. 138. Parliamentary Standing Orders.

**Operational Acceptance:** Full reporting chain exercised in dress rehearsal.

---

### M59 — Auditor General Interface

**Functional:** Provides Auditor-General's office with read-only access to election financial records (M54), expenditure audit trails, IT system audit logs, and reproducibility bundle (M14). Structured evidence package delivery on request. Access to Information Act 2016 compliance.

**Non-functional:** Evidence package delivery ≤ 48 hours of request. Read-only; no write to any production system.

**Integration:** M54 Budget Expenditure, M14 Reproducibility Bundle, M10 RTS Backend (results audit), M69 Long-Term Archive.

**User acceptance:** Auditor-General's office: evidence package request and receipt UAT. IEBC finance officer: audit query response UAT.

**Security acceptance:** AG office certificate-authenticated access. Dual-custody for sensitive evidence packages. Audit log of all AG accesses.

**Regulatory:** Access to Information Act 2016. Public Audit Act 2015. PPADA 2015.

**Operational Acceptance:** AG access tested pre-election (T-30). Post-election report delivered within T+30.

---

### M60 — Medical First Response Coordination

**Functional:** Medical incident reporting from polling stations (clerk or supervisor). Dispatch of first-aid coordinator. Incident log for post-election review. Integration with M33 Personnel Dispatch.

**Non-functional:** Incident report submission ≤ 30 s. Dispatch notification ≤ 60 s.

**Integration:** M33 Personnel Dispatch, M16 ECC, M56 County Coordination.

**User acceptance:** Presiding officer persona: medical incident report UAT.

**Security acceptance:** Location data stored at station level only (not GPS coordinates). Data deleted T+7 (no evidentiary hold needed). Lawful basis s.30(1)(c) (vital interests).

**Regulatory:** DPA 2019 s.30(1)(c). Occupational Safety and Health Act.

**Operational Acceptance:** Incident simulation with dispatch response verified.

---

### M61 — Secrecy Assurance Testing

**Functional:** Red-team engagement lifecycle management. Finding tracker (RESTRICTED classification). Remediation workflow. IAB attestation record. Separate from main pen-test programme. Covers: RTS metadata analysis, BVI device secrecy, party-agent app leaks, transmission sequencing analysis.

**Non-functional:** Finding PoC details encrypted (dual-key); no live voter data in any pen-test activity. IAB attestation produced before T-30.

**Integration:** M08 (RTS secrecy), M26 Anomaly Detection (feeds findings as detection rules), M30 Mobile Party Agent (secrecy review), M28 AI Monitoring (clone-app detection results).

**User acceptance:** IAB Chair: attestation review and sign-off UAT. IEBC CISO: finding disposition review.

**Security acceptance:** M61 itself is subject to annual pen-test (separate from KIEMS programme). PoC blobs verifiably encrypted. No live voter data found in any pen-test artefact.

**Regulatory:** Elections Act §54 + §72 (secrecy obligation). DPA 2019 s.30(1)(b). ODPC DPIA for pen-test activities.

**Operational Acceptance:** IAB secrecy attestation signed before T-30. All P0 secrecy findings remediated.

---

### M62 — Transmission Legal Basis Audit

**Functional:** Per-transmission audit of legal basis for each data element in the RTS transmission stream. Flags any transmission element without documented lawful basis. Produces periodic audit report for IEBC DPO.

**Non-functional:** Audit run ≤ 24 hours for full election-day transmission log. No PII in audit tool logs.

**Integration:** M08, M09, M10 (transmission events), M10 (Merkle chain for completeness).

**User acceptance:** IEBC DPO: audit report review UAT. ODPC representative: methodology review.

**Security acceptance:** Audit tool has read-only access. No personal data in audit logs (hash-only references). Audit report signed.

**Regulatory:** DPA 2019 s.17 (lawful basis documentation requirement). ODPC audit obligation.

**Operational Acceptance:** Post-election audit report produced within T+14.

---

### M63 — Ballot Serial Chain

**Functional:** Issues unique serial numbers for all ballots (per constituency, per station, sequential). Maintains Merkle-chained serial register. Public verification: any observer can verify a ballot's serial is in the legitimate range for its station. Cross-links with M39 (printing) and M20 (distribution).

**Non-functional:** Serial issuance ≤ 1 s per batch of 10,000. Merkle root published immediately after each serial batch. Public verification ≤ 500 ms.

**Integration:** M39 Ballot Printing (serial range allocation), M20 Inventory (physical ballot tracking), M21 RLA Audit Tool (ballot manifest source), M40 Court Interface (serial chain as evidence).

**User acceptance:** IEBC print officer: serial batch issuance UAT. International observer: public serial verification UAT.

**Security acceptance:** No voter linkage possible from serial number alone. Serial chain Merkle-rooted and published to transparency log. Tampering with serial chain raises SIEM alert.

**Regulatory:** Elections Act §39 (ballot accountability). Access to Information Act (serial chain is public data).

**Operational Acceptance:** 60M serial numbers issued and Merkle-chained in staging test.

---

### M64 — Voter List Public Inspection

**Functional:** Provides public inspection period access to voter register (Elections Act §35). Redacted list: name, ward, polling station — no NID, no biometric. Search by name or polling station. Objection submission workflow (challenged entries).

**Non-functional:** P99 lookup ≤ 500 ms. Rate limit: 100 lookups/IP/hour. List published for statutory inspection period only.

**Integration:** M35 CVR (source), M36 Voter Deregistration Appeal (objection processing).

**User acceptance:** Civil society organisation: list inspection and objection submission UAT. IEBC registrar: objection review UAT.

**Security acceptance:** NID not exposed in public list. Rate limit prevents NID enumeration even from name search. CAPTCHA on suspicious patterns. Inspection period enforced (list unpublished outside statutory window).

**Regulatory:** Elections Act §35 (public inspection right). DPA 2019 (lawful basis s.30(1)(b)); redaction required to minimise personal data exposure.

**Operational Acceptance:** Full 30M-voter redacted list published in staging. Statutory inspection period enforcement verified.

---

### M65 — Diaspora Ballot Physical Logistics

**Functional:** Tracks physical ballot packs dispatched to diaspora voting centres (embassies and consulates). Ballot-by-ballot tracking using serial numbers (M63 integration). Return tracking. Reconciliation on arrival.

**Non-functional:** Tracking update ≤ 24 hours from postal/courier scan. 99% tracking coverage for dispatched ballots.

**Integration:** M63 Ballot Serial Chain (serial tracking), M24 Diaspora Voting (voter eligibility), M20 Inventory (physical stock).

**User acceptance:** IEBC diaspora coordinator: dispatch and return tracking UAT.

**Security acceptance:** No voter-ballot linkage in tracking system (serial used, not voter ID). Tracking data retained until T+6 months then deleted.

**Regulatory:** Elections Act §52. DPA 2019. International mail/courier agreements.

**Operational Acceptance:** Diaspora ballot logistics simulation with 50 embassy partners.

---

### M66 — Election Financial Forensics

**Functional:** Forensic investigation tool for campaign finance anomalies. Case management for IEBC enforcement. POCAMLA compliance (money laundering red flags). Evidence package for prosecution referral. Integration with M22 Campaign Finance and M54 Budget Expenditure.

**Non-functional:** Case management P99 ≤ 2 s. Evidence package generation ≤ 4 hours.

**Integration:** M22 Campaign Finance (anomaly escalation), M54 Budget (expenditure anomaly), M40 Court Interface (forensic evidence).

**User acceptance:** IEBC enforcement officer: case management UAT. ODPP (Director of Public Prosecutions) liaison: evidence package format verification.

**Security acceptance:** Dual-custody access to forensic case files. Chain-of-custody record per file access. No unauthorised access to case files. DPIA on file.

**Regulatory:** POCAMLA 2009. Election Campaign Financing Act 2013. ODPC DPIA.

**Operational Acceptance:** Case management tested with synthetic financial anomaly scenarios.

---

### M67 — Party Symbol and Colour Registry

**Functional:** Official registry of political party symbols and colours. Symbol uniqueness enforcement (no confusingly similar symbols). Public reference for ballot design. Colour palette export for M39 Ballot Printing.

**Non-functional:** P99 registry lookup ≤ 200 ms. Public registry always available (99.9%). Symbol similarity check ≤ 5 s (perceptual hash comparison).

**Integration:** M45 Party Registration (symbol approval), M39 Ballot Printing (symbol export), M12 CRMS (party symbol on nomination).

**User acceptance:** IEBC registrar: symbol submission and similarity check UAT. Party official: symbol registration UAT.

**Security acceptance:** No PII. Registry entries signed. Tampering raises SIEM alert.

**Regulatory:** Political Parties Act 2011 (symbol registration). Elections Act §34 (ballot design).

**Operational Acceptance:** All registered party symbols loaded and verified before nomination window opens.

---

### M68 — Referendum Explainer Platform

**Functional:** Citizen-facing educational platform explaining referendum questions in plain language, EN + SW + 8 additional Kenyan languages. Video, audio, infographic. Ballot simulation (practice). Accessible to feature phone via USSD. WCAG 2.2 AAA target.

**Non-functional:** P99 page load ≤ 2 s on 3G. USSD flow ≤ 5 steps. Support 10 Kenyan languages. 5M concurrent users on CDN. WCAG 2.2 AAA verified by independent accessibility auditor.

**Integration:** M37 Voter Education CMS (content sharing), M49 Referendum Mechanics (ballot simulation alignment).

**User acceptance:** Low-literacy voter persona (multiple language communities): usability test. Accessibility tester (blind + deaf + mobility-impaired) personas: full WCAG 2.2 AAA audit.

**Security acceptance:** No PII. CSP strict. HTTPS only. Content signed. No cookies unless strictly necessary (referendums are constitutionally sensitive).

**Regulatory:** Constitution Art. 255-257 (referendum public participation). Access to Information Act 2016. ODPC (no PII).

**Operational Acceptance:** Full language set published before referendum date. AAA audit passed. 5M-user load test.

---

### M69 — Election Data Long-Term Archive

**Functional:** Permanent immutable storage of all election artefacts: signed Form 34A/B images, Merkle chain, audit logs, reproducibility bundle, biometric template erasure certificates (after cycle closure), SBOM history, all signed events. WORM storage. SLH-DSA (FIPS 205) re-signing annually for post-quantum survivability of 15-year archive. Tiered storage: online ≤ 7 years, cold archive 7-15 years. Retrieval SLA: ≤ 4 hours for any archived item.

**Non-functional:** Ingest throughput: 100,000 events/s at election close. Annual re-signing: complete within 48 hours. Storage: ≥ 100 TB capacity at primary and escrow.

**Integration:** M10 (signed events), M06 (auth logs), M40 (court evidence retrieval), M14 (bundle archive), M59 (AG access), M21 (RLA audit records).

**User acceptance:** IEBC archivist: retrieval workflow UAT. Auditor-General's office: long-term access verification UAT.

**Security acceptance:** WORM enforcement verified (no deletion possible without dual-custody + court order). SLH-DSA re-signing workflow tested. HSM-sealed archive keys. Access logged per item. Annual re-signing event published to transparency log.

**Regulatory:** Public Archives and Documentation Service Act. Elections Act §39 (permanent record). DPA 2019 §41 (cryptographic erasure certificates stored here).

**Operational Acceptance:** 15-year archive lifecycle simulation (compressed to 30-day test). Re-signing verified. AG access tested.

**Rollout:** Archiving begins at election close (T+0).

---

## 4. Programme-Level Acceptance Criteria

### 4.1 Election-Day Success Criteria

The election-day acceptance is declared on 2027-08-10 (T-0) and confirmed at T+24 hours:

| Criterion | Target | Measurement |
|---|---|---|
| Station coverage | 100% of 55,393 polling stations open with functional KIEMS kit | M16 ECC real-time dashboard; M20 inventory return |
| Authentication availability | 99.99% kit uptime over election day (08:00-17:00) | Prometheus SLO dashboard |
| RTS results received | 100% of Form 34A from all stations within T+12 hours of polls closing | M10 backend tracking |
| National result published | National result published to M11 Public Portal within T+72 hours | Publication timestamp in transparency log |
| Risk-Limiting Audit | RLA confirms declared winner at α ≤ 0.05 before M51 national certification | M21 RLA report signed by auditors |
| Zero P0 security incidents | No undetected P0 security event during election window (T-30 to T+7) | M61 + SIEM post-election review |
| Anomaly detection | All statistical outliers (Benford, clustering, timing) flagged and dispositioned by M26 before certification | M26 alert log; M15 IAB acknowledgement |

### 4.2 Petition-Window Success Criteria

The petition window is T+0 (result proclaimed) to T+7 (petition filing deadline per Elections Act §75(1)):

| Criterion | Target |
|---|---|
| Evidence bundle delivery | Any evidence bundle requested by a petitioner's legal team delivered within 24 hours of request (M40) |
| SC IT Registrar dry-run | Supreme Court IT Registrar pre-election dry-run completed at T-14; acceptance confirmed in writing |
| Evidence integrity | Every artefact in every evidence bundle has unbroken chain-of-custody record from origin to court delivery |
| RLA report | Final RLA report signed and available as evidence by T+14 |
| Expert witness | Named expert witness designated and briefed by T+14 |

### 4.3 Post-Election Success Criteria

Final acceptance period: T+0 to T+90:

| Criterion | Target | Deadline |
|---|---|---|
| Reverse logistics | 100% of 65,287 kits accounted for in M20 (returned or declared lost with police report) | T+7 |
| Biometric template erasure | All biometric templates from election cycle cryptographically erased; erasure certificates filed in M69 | T+90 |
| Post-election review report | M41 report signed by IEBC and delivered to IAB | T+30 |
| Auditor-General report | M59 evidence package delivered; AG access to all financial records confirmed | T+30 |
| Bug-bounty reinstatement | Bug-bounty programme reinstated (suspended at T-30) | T+14 |
| Source-code escrow final deposit | All source code and SBOM for T+0 deployed versions escrowed | T+7 |
| Parliamentary reporting | M58 formal result report delivered to National Assembly Clerk | T+3 |

### 4.4 Three-Year Framework Success Criteria

These determine whether the programme has delivered long-term value by the 2030 local elections and 2032 general elections:

| Criterion | Target | Measurement |
|---|---|---|
| No repeat-cycle procurement risk | IEBC can operate and maintain the system without Datacraft by 2030 | Demonstrated by IEBC-run drill of full election workflow at T+18 months |
| IEBC capacity building | 23 HQ ICT + 47 County ICT staff can perform Tier 1 and 2 operations independently | Training completion records + T+18 month operational assessment |
| Self-sufficiency by 2032 | IEBC manages own election cycle (M35, M01, M05 provisioning, M20 inventory) without Datacraft support contract | 2032 readiness assessment by IAB |
| ISO 27001 ongoing | ISO 27001 surveillance audit maintained annually through 2032 | Annual audit certificates |
| No critical unpatched CVEs | Dependency scanning (Trivy + Snyk) shows zero critical unpatched CVEs in any production system continuously | Continuous Renovate + weekly scan cadence; IEBC ICT responsible post-T+18 |
| Archive integrity | M69 annual re-signing completed each year through 2032; all election artefacts retrievable | Annual re-signing event log on transparency log |

---

*Document version 2.0. Approved: Datacraft QA Director + IEBC Project Manager (at OAT). Effective: pre-award. Update trigger: module specification change, contractual amendment, or regulatory revision.*
