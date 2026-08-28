# Datacraft Software Build Scope — IEBC IEMS 2027
## Exhaustive Build-Ready Specification

**Contract:** IEBC/OIT/01/2026-2027
**IPR:** All Custom Software vests in IEBC per GCC 15.4; Standard Software licensed perpetually, fully paid-up, irrevocably per GCC 16.
**T-0 (election day):** 2027-08-10
**Version:** 3.0 — Exhaustive build specification (69-module scope)
**Status:** Pre-award bid document; converges to final specification at Preliminary Project Plan approval

---

## Section 0 — Executive Summary

### 0.1 Programme scope in one paragraph

The IEBC Integrated Elections Management System 2027 is Kenya's sovereign, end-to-end election technology stack, comprising 69 purpose-built software modules that collectively run biometric voter registration for approximately 30 million Kenyans; continuously maintain the register between elections; authenticate voters at approximately 55,393 polling stations on election day; transmit and tally results from up to 60,000 concurrent sessions across six simultaneous elective positions plus any referenda; expose public transparency portals that any citizen can independently verify on a feature phone; manage candidate nominations for approximately 15,000 candidates; manage party registration, primaries oversight, and symbol/colour uniqueness; operate six mobile apps for field officers (party agents, poll clerks, observers, field engineers, dispatch personnel, and citizens); detect and alert on election-day anomalies in real time; manage a physical inventory of approximately 65,287 KIEMS biometric kits; track individual ballot papers from printer to polling station; provide a cryptographically reproducible results bundle that any auditor can rerun on a standard laptop; manage special elections (referenda, by-elections, recall elections); coordinate with counties, international observers, Parliament, the Auditor-General, and courts; manage election staff HR and payroll; and preserve election data for 15 years in a standards-compliant archive. Every module is built as a new component with no legacy code and no incumbent platform assumptions other than explicitly named OSS and third-party products. All Custom Software IPR vests in IEBC at Operational Acceptance, and every module is designed to be buildable, deployable, and operated by IEBC without Datacraft cooperation.

### 0.2 Team size and duration

Peak engineering headcount: 420-500 people across programme management, architecture, security, data protection, domain engineering, platform, mobile, embedded, ML/data, QA, SRE, and field operations. Active engineering spans approximately T-720 to T-14 (roughly 2025-08 to 2027-07-27), with a support-only regime from T-14 through T+180 (petition window close, approximately 2028-02). The 69-module scope drives the increase from the 28-module baseline (240 peak): roughly 1.8x raw module count at diminishing marginal cost due to shared platform, common mobile codebase (Flutter), and reuse of M02/M10 core patterns across register maintenance, compliance audit, and special-election modules.

### 0.3 Cost of build (rough order of magnitude)

Total programme cost estimate (build + infrastructure + licensing + operations through T+3 years): KES 3.5 billion to KES 4.5 billion at a blended rate of KES 480,000 per person-month for senior Kenyan engineers and KES 1,100,000 per person-month for international specialists, plus infrastructure and third-party licensing. Datacraft software and services component: KES 1.35-1.45 billion; total programme including Mechrevo hardware: KES 3.5-4.5 billion. Per-subsystem detail in Section 9. This is a rough order of magnitude only; the fixed-price contract envelope is governed by the bid response.

### 0.4 Four architectural choices that dominate the design

1. **Async Python everywhere (back-end).** All server-side modules are Python 3.13 async with FastAPI + Starlette + SQLAlchemy 2.x async. This is the Datacraft standard per CLAUDE.md, maximises internal hiring depth, and guarantees consistent error-handling patterns across 28 modules.

2. **Event-sourced core.** Every state change is an immutable event appended to a hash-chained, Merkle-logged event store. State is derived by replaying events. No event is ever deleted or updated in place. This provides a complete tamper-evident audit trail and enables deterministic reproducibility of any result from first principles.

3. **Offline-first edge.** Every kit-side device client (M01 BVR, M05 BVI, M08 RTS, M27 Intent Preservation) stores, signs, and queues records locally. The kit operates fully for hours without network connectivity and submits on reconnect with idempotency guarantees. This prevents the "network cut, election stops" failure mode.

4. **Cryptographically anchored publication.** Results are not just published; they are Merkle-rooted, and the root hash is published simultaneously to the IEBC transparency log, GitHub, IPFS, and the Kenya Gazette. Any citizen with a station code can verify their station's tally against the national root in under 30 seconds. This design converts the entire electorate into independent auditors.

### 0.5 What "done" looks like on 2027-08-10

On election day, every one of 55,393 polling stations opens on time with a functional, certified KIEMS kit. The kit authenticates voters by fingerprint in under 3 seconds. Results transmission begins as polls close. The national aggregation (M10) processes 60,000 concurrent sessions at p99 sub-100 ms authorisation latency. The Public Results Portal (M11) serves up to 50 million unique visitors within 24 hours of publication, backed by a CDN-cached signed static snapshot that survives any DDoS attack on the origin. The Public Verifier App (M13) allows any Kenyan with a feature phone to verify their station's result. The Anomaly Detection Stack (M26) has flagged and escalated every statistical outlier to the Election Command Centre (M16) and the Independent Assurance Board (M15) in real time. The Reproducibility Bundle (M14) allows any observer to deterministically reproduce the national tally from signed source images on a standard laptop. Zero P1 security incidents are undetected. Every signed Form 34A is retrievable in under 5 minutes for any court petition.

---

## Section 1 — Scope Boundary

### 1.1 What Datacraft builds (Custom Software — new build)

All 69 modules listed in Section 3, their shared platform layers, their test harnesses, their runbooks, their deployment automation, their acceptance-test suites, the data-migration toolkit from the existing register, and the source-code + IPR transfer package. Every line of code, every schema, every CI pipeline, every API contract, and every operational runbook for these 69 modules is a Datacraft new build.

### 1.2 What is off-the-shelf third-party (integrated but not built)

| Component | Vendor options | Datacraft's obligation |
|---|---|---|
| ABIS biometric matching engine | IDEMIA MorphoSmart / Innovatrics ABIS / Thales Cogent | Integrate via SDK; build ingestion pipeline, adjudication workbench, SLA monitoring, all custom UI |
| HSM firmware and appliance | Thales Luna 7 / Utimaco CryptoServer / Entrust nShield | Integrate via PKCS#11 + vendor SDK; build signing service, key-ceremony tooling, audit hooks |
| PostgreSQL 17 | Open source | Operate, tune, HA cluster, backup; no modification to engine |
| Redis 7 | Open source | Operate; no modification |
| Apache Kafka + Confluent Schema Registry | Open source + Confluent | Operate; build all producers and consumers |
| MinIO | Open source | Operate; build all SDK wrappers |
| OpenSearch | Open source | Operate; build all index schemas and query services |
| Keycloak | Open source | Configure and extend; build all custom flows and extensions |
| Envoy / Linkerd 2 | Open source | Configure and operate as service mesh |
| Grafana + Loki + Prometheus + Tempo | Open source | Configure; build all dashboards and alert rules |
| Playwright | Open source | Write all test suites |
| React + Next.js | Open source | Build all voter-facing front-end |
| React Native | Open source | Build all mobile apps |
| HashiCorp Vault | BSL | Configure and operate; build all integration wrappers |
| ArgoCD | Open source | Configure and operate; write all Application manifests |
| ONNX Runtime + TFLite | Open source | Write all models and inference pipelines |
| k6 | Open source | Write all load-test scripts |
| Chaos Mesh | Open source | Write all chaos experiments |

### 1.3 What is IEBC-provided

- Primary and mirror data-centre converged infrastructure (servers, storage, networking) per IEBC Section V.
- Telco VPN infrastructure (physical circuits and commercial agreements with Safaricom, Airtel, Telkom).
- HSM hardware appliance (procured by IEBC from Thales/Utimaco; Datacraft integrates).
- Physical facilities: data centres, constituency stores, polling stations.
- KNPS security operations at polling stations and data centres.
- ~14,100 legacy Android KIEMS devices (existing fleet; Datacraft ports a compatibility-mode client).
- IEBC-employed poll workers, Returning Officers, County Returning Officers.
- Ballot printing (post-CRMS handoff; Datacraft generates ballot proofs, printing is separate contract).
- Kenya Gazette publication process (Datacraft generates gazette-ready exports; Gazette Office publishes).

### 1.4 What is Mechrevo-supplied

- Hardware only: new KIEMS kit assembly, sensors, battery, SIM hardware, accessories, firmware for sensor modules. Mechrevo delivers to specification; Datacraft does not take responsibility for hardware design or manufacturing defects.

### 1.5 What is explicitly out of scope

- Voter education content, civic outreach campaigns, broadcast media.
- Campaign material design or distribution.
- Judicial process, election petition adjudication workflow (Datacraft delivers evidence packs on demand; the court's own systems are not in scope).
- KNPS security operations and command.
- KRA tax verification beyond calling the KRA API adapter.
- Ballot printing execution and ink/paper supply chain.
- Commercial eSIM/SIM agreements (IEBC/Safaricom/Airtel/Telkom negotiates; Datacraft builds the management software).
- IEBC's internal HR and payroll systems for permanent IEBC employees.

---

## Section 2 — Reference Architecture

### 2.1 Layered view

```
LAYER 6: VERIFICATION
  Public verifier app (M13) · Observer portals · Court evidence API · Transparency log verifiers (civil society)

LAYER 5: PUBLICATION
  Merkle-anchored results (M14 reproducibility) · CDN signed snapshots · Kenya Gazette exports
  IPFS pinning · GitHub publication · Public Results Portal (M11)

LAYER 4: PROCESSING
  National aggregation (M10) · ABIS dedup (M03) · ICR/AI tally extraction · RLA audit (M21)
  Anomaly detection (M26) · Campaign finance (M22) · IAB dashboard (M15)

LAYER 3: INGRESS
  RTS results ingestion · Observation channels · Grievance intake (M23) · CRMS nominations
  Queue transparency (M19) · Diaspora voting (M24) · Special voter workflows (M25)

LAYER 2: TRANSPORT
  Telco VPN (Safaricom / Airtel / Telkom) · Satellite fallback (Starlink / VSAT)
  eSIM carrier rotation · SMS/USSD fallback · CDN edge

LAYER 1: EDGE
  KIEMS kits (M01 BVR, M05 BVI, M08 RTS, M27 Intent Preservation)
  Constituency tally tablet (M09) · County workstation · Diaspora voting terminal
```

### 2.2 Service topology: 69 modules into deployable services

The 69 modules compose into 18 deployable Kubernetes services. Grouping principle: modules that share the same data store, are always deployed together, and communicate synchronously in their happy path belong in the same service. Modules with meaningfully different SLOs, scaling profiles, or security boundaries are separated even if they share a domain.

| Deployable service | Modules | Grouping rationale |
|---|---|---|
| biometric-edge | M01, M05, M08, M27, M31 (kit-side APK/binary; not Kubernetes) | All run on the KIEMS device; share biometric SDK, offline queue, kit-cert signing |
| bvr-svc | M02, M35, M36, M53, M64 | All read/write the voter register; share PostgreSQL schema and DPA lawful-basis logic |
| abis-svc | M03 | Isolated for confidential compute; no shared schema with any other service |
| vprs-svc | M04, M07 | Public registration and register inspection; both are citizen-facing read-heavy web |
| bvi-svc | M06 | Polling-day kit auth; separate SLO (99.99% polling window); no shared write path |
| rts-svc | M09, M10, M51, M62 | Results ingestion, tally, formal declaration, and per-transmission legal-basis audit; same Kafka topics and PostgreSQL results schema |
| portal-svc | M11, M13, M29, M68 | Public-facing read portals and apps; CDN-first; no write paths; referendum explainer is same delivery pattern |
| crms-svc | M12, M22, M45, M46, M47, M67 | Candidate and party domain; shared party/candidate PostgreSQL schema; nomination fee payment same event chain |
| transparency-svc | M14, M15, M21, M57, M58, M59 | Audit, reproducibility, IAB, and external reporting; all read-only consumers of signed events; OAG / Parliament / international observer portals share the same read-model |
| ecc-svc | M16, M17, M44, M60 | Operational command, speech integrity, turnout forecasting, and medical coordination; all drive the ECC operator dashboard and share the same WebSocket push bus |
| boundary-svc | M18, M52 | Boundary and election-calendar data; both are spatial/temporal reference services consumed by all other modules |
| queue-svc | M19 | Separate: high-frequency polling-station telemetry; different scaling profile (100K updates/minute peak) |
| inventory-svc | M20, M39, M63, M65 | Physical asset tracking domain: kit lifecycle, ballot printing chain, individual ballot serial tracking, diaspora ballot physical logistics; share the Merkle chain-of-custody data model |
| governance-svc | M23, M24, M25, M40, M48, M49, M50 | Grievance, diaspora, special voters, election court IT interface, recall/referendum/by-election mechanics; all are episodic workflows sharing the governance PostgreSQL schema |
| ai-platform-svc | M26, M28, M66 | Anomaly detection, AI monitoring stack, and financial forensics; share ML model registry, ONNX Runtime workers, and analytics data lake |
| field-mobile-svc | M30, M32, M33, M34 | Field-officer mobile backends: party-agent, field-support, personnel-dispatch, observer; MDM-distributed Flutter apps; share field-ops PostgreSQL schema |
| training-svc | M37, M38, M41, M42, M43 | Content and accreditation: voter education CMS, poll-worker LMS, post-election review, media and observer accreditation; episodic and low-throughput; same CDN delivery pattern |
| support-admin-svc | M54, M55, M56, M61, M69 | Administrative backends: budget/expenditure, HR/payroll, county coordination, secrecy-assurance testing, long-term archive; share back-office PostgreSQL schema; infrequent writes |

### 2.3 Data-plane topology

```
Kit (offline-first SQLite encrypted store)
  --> [mTLS + signed batch] --> Kit Gateway (edge Kubernetes node per region)
  --> [Kafka topic: bvr.enrollments / bvi.events / rts.forms] --> Processing services
  --> [PostgreSQL primary] --> [WAL streaming replication] --> Mirror PostgreSQL
  --> [Debezium CDC] --> [Kafka outbox] --> Downstream consumers (ABIS, audit, BI)
  --> [MinIO: scanned Form 34A images + signed biometric archives]
  --> [Merkle builder] --> [Transparency log: IEBC + GitHub + IPFS]
  --> [CDN pre-bake job] --> [Edge CDN: public read, no auth]
```

National results flow specifically:

```
55,393 kit stations
  --> RTS back-end (M10) aggregates per polling station (Form 34A)
  --> 290 constituency tally (M09 Returning Officer) aggregates Form 34B
  --> 47 county tally aggregates Form 34C
  --> Presidential national tally (Form 34D)
  --> Merkle-builder (M14) hashes every form, builds national tree
  --> Root published to transparency log + M11 portal
  --> M13 verifier app fetches and cross-checks
```

### 2.4 Control-plane topology

- **Election Command Centre (M16):** unified situational view for all 28 modules. IEBC Commissioners, ICT ops team, and Datacraft SRE team all access ECC.
- **Independent Assurance Board Dashboard (M15):** read-only, hash-verified feed. IAB members access a separate deployment with no write paths.
- **Per-module admin consoles:** embedded in each service; access governed by Keycloak RBAC roles (Commissioner, Secretariat, ICT Admin, RO, PO, Party Agent, Auditor).
- **Observers:** dedicated observer portal with per-station, per-constituency, per-county drill-down, downloadable signed evidence packets.

### 2.5 Cross-cutting concerns

| Concern | Solution |
|---|---|
| Identity | Keycloak OIDC for human principals; SPIFFE/SPIRE for service-to-service; per-kit certificates from internal PKI |
| Authorisation | OPA policy engine; Cedar for fine-grained resource-level decisions |
| Observability | OpenTelemetry collector on every pod; Prometheus scrape; Loki log aggregation; Tempo traces |
| Config | Declarative Kubernetes manifests in git; ArgoCD GitOps; all changes signed |
| Secrets | HashiCorp Vault sealed with Thales Luna HSM; no plain-text secrets in git or CI/CD |
| Service mesh | Linkerd 2 with mTLS on every svc-to-svc call; per-namespace policy |
| API contracts | OpenAPI 3.1 spec-first; protobuf for gRPC event bus; CloudEvents 1.0 envelope |
| Error model | RFC 7807 Problem Details; correlation IDs propagated from kit through to CDN edge |

### 2.6 Deployment topology

- **Primary site:** Kubernetes 1.31 on IEBC-provided bare metal, primary data centre (Anniversary Towers area, Nairobi).
- **Mirror site:** identical Kubernetes cluster, IEBC DR site. Synchronous replication on write-critical paths (register, results, HSM audit); async for BI marts.
- **Air-gapped provisioning environment:** kit prep, PKI ceremony, key ceremonies; no internet. One-way data diode for signed image exports.
- **Edge nodes:** regional Kubernetes edge clusters (5 regional hubs) for kit gateway, aggregation cache, and fallback serving.
- **Kit-side:** Android APK (Kotlin + Jetpack Compose) on Snapdragon SM7325-class or RK3588-class SoC running Android 13+. Legacy Android port (feature matrix compatibility layer) for ~14,100 existing kits.
- **CDN:** Cloudflare or Akamai for public portal; signed static snapshots, no origin dependency for reads.

---

## Section 3 — Per-Subsystem Specification (M01–M69)

---

### M01 — BVR Device Client

**Purpose:** Capture biographic and biometric data for every voter at registration centres, fully offline-capable.

**Stack:** Kotlin + Jetpack Compose (Android 13+); Room DB (SQLite encrypted); SQLCipher; Bouncy Castle for on-device crypto; ISO 19794 SDK wrappers; native biometric sensor SDKs. Legacy Android compatibility layer in Java (feature matrix degradation for older devices).

**Deployment:** Kit-side Android APK signed by Datacraft code-signing cert. Deployed via MDM (M20 inventory / OTA). Not a Kubernetes workload.

**Team:** 1 mobile lead, 3 Android engineers, 1 embedded/biometric SDK specialist, 1 QA, 1 SRE (OTA pipeline). Total: 7.

**Build effort:** 18 person-months [LOW 14 / MID 18 / HIGH 24].

**Dependencies upstream:** PKI service (M04/P04) for per-kit cert; inventory (M20) for kit registration. **Downstream:** BVR backend (M02) for record submission; MDM for OTA config; geo-fence layer for polygon check.

**Three hardest problems:**
1. *Biometric SDK integration on two hardware generations (new Snapdragon + legacy Android fleet).* Approach: hardware abstraction layer over all sensor SDKs; automated compatibility matrix CI on real devices in a device farm.
2. *Offline-first durability with cryptographic integrity.* Approach: SQLCipher encrypted Room DB as local queue; every record signed with kit private key at creation; submission idempotency via per-record UUID7; WAL-mode journaling for crash safety.
3. *Fingerprint quality gating at FAP 50/60 under field conditions (sweat, dirt, elderly voters).* Approach: NIST NFIQ 2.0 quality scoring built in; retry prompts with finger guidance; fallback modality order (fingerprint → iris → facial → manual search).

**Third-party components:** NIST-certified ABIS SDK from chosen vendor (IDEMIA/Innovatrics/Thales) for on-device dedup cache; SQLCipher (MIT licence) for encrypted local store; Bouncy Castle for EC key pair + signing.

**Delivery milestone:** Functional prototype T-540 (2026-02-17); field-trial candidate T-270 (2026-11-13); regulatory build freeze T-60 (2027-06-11).

**Test strategy:** Unit tests (JUnit + Mockito, ≥80% coverage of business logic); integration tests on physical devices (5 device SKUs in CI farm); offline-sync stress test (5,000-record backlog submit); biometric accuracy test against IEBC-supplied ground-truth set; field trial in 3 Kenyan counties T-270.

**Compliance anchor:** Elections Act §6A (enrolment window enforcement); DPA 2019 §25 (biometric data processing with explicit consent); Elections (Technology) Regulations 2017 Reg 5 (functional test).

---

### M02 — BVR Backend (Voter Registration Central System)

**Purpose:** Central voter register store; ingestion from kits; ABIS trigger pipeline; RO back-office; register generation.

**Stack:** Python 3.13 async; FastAPI + Starlette; SQLAlchemy 2.x async; PostgreSQL 17 (primary + mirror, streaming replication); Redis 7 (session cache, rate limiting); Kafka (inbound enrolment events, outbound ABIS trigger); MinIO (biometric archive, ID scan PDFs); Alembic (migrations).

**Deployment:** Kubernetes, primary + mirror. Three replicas minimum; HPA up to 20 replicas under registration-window peak. PostgreSQL in HA cluster (Patroni + HAProxy). Redis Sentinel.

**Team:** 1 lead, 3 back-end engineers, 1 data engineer, 1 QA, 1 SRE. Total: 7.

**Build effort:** 22 person-months [LOW 18 / MID 22 / HIGH 28].

**Dependencies upstream:** M01 (kit enrolment events); PKI/IAM (M04) for kit auth. **Downstream:** M03 ABIS (trigger); M07 BVV (register read); M04 VPRS (pre-reg merge); BI fabric (P10).

**Three hardest problems:**
1. *Optimistic concurrency on a 30M-row voter register with continuous enrolment from 65,000+ kits.* Approach: per-record version counter; Kafka per-kit sharded topics to serialise per-kit writes; conflict resolution at ingestion layer with signed provenance.
2. *Gazetted cut-off with legally-timed record freeze.* Approach: election-calendar service publishes cut-off timestamps; register write path checks election-state machine; freeze is applied atomically in a PostgreSQL transaction with a counter-signed IEBC Commissioner event.
3. *Electoral-area mapping consistency (county → constituency → ward → polling centre → station).* Approach: immutable electoral-area table loaded from gazetted boundary data (M18 integration); foreign-key hierarchy enforced in DB; any area change requires a migration with dual approval.

**Key third-party:** Patroni + HAProxy (open source) for PostgreSQL HA; Debezium (open source) for CDC outbox; pg_partman for time-series partitioning on audit log tables.

**Delivery milestone:** Core register store T-600; full RO back-office T-480; register generation + ABIS pipeline T-420.

**Test strategy:** ≥80% unit coverage; integration tests with embedded PostgreSQL for CI; load test (k6) simulating 65,000 concurrent kit submissions; ABIS pipeline integration test against ABIS sandbox; field trial integration T-270.

**Compliance anchor:** Elections Act §44 (accurate register); DPA 2019 §26 (data minimisation, purpose limitation); Elections (Tech) Reg 2017 Reg 6.

---

### M03 — ABIS (Biometric Identification Integrator)

**Purpose:** Wrap a third-party biometric matching engine at ≥30M records; own the ingestion pipeline, adjudication workbench, SLA enforcement, and audit trail. Datacraft is the integrator, not the ABIS engine builder.

**Stack:** Python 3.13 async; FastAPI (adjudication API); SQLAlchemy async; PostgreSQL 17 (candidate + match result store); Kafka (ingest from M02); MinIO (template archive); SDK wrapper for chosen engine (IDEMIA/Innovatrics/Thales). Confidential compute enclave (AMD SEV-SNP or Intel TDX) for matching workers.

**Deployment:** Kubernetes with GPU-accelerated nodes for iris matching workers. Confidential-compute node pool for biometric template processing. Isolated network segment; no direct internet egress.

**Team:** 1 lead (biometric specialist), 2 back-end engineers, 1 ML/biometric engineer, 1 QA. Total: 5.

**Build effort:** 16 person-months [LOW 12 / MID 16 / HIGH 20] — effort is integration/orchestration, not engine build.

**Dependencies upstream:** M02 (template ingest triggers). **Downstream:** M02 (match results, dedup flags); M16 ECC (SLA alert); M15 IAB (audit feed).

**Three hardest problems:**
1. *1:N search against 30M records at p99 ≤ 8 seconds for single-modality query.* Approach: engine-vendor tuning; sharded matching pools; pre-computed candidate shortlists using blocking filters (county, DOB range); benchmark against IEBC-provided test set before go-live.
2. *Multi-modal fusion threshold calibration.* Approach: ROC curve analysis on ground-truth dataset; operating point selection at FAR ≤ 0.01%; adjustable per-election-cycle by IEBC with dual-commissioner approval.
3. *Adjudication queue SLA under peak (millions of potential duplicates during mass registration events).* Approach: priority queuing by risk score; investigator workbench with side-by-side comparison and minutiae overlay; SLA timers per case; escalation to senior investigator at 80% SLA consumed.

**Key third-party:** ABIS engine SDK (IDEMIA/Innovatrics/Thales — chosen at contract award); AMD SEV-SNP or Intel TDX for confidential compute; NIST NFIQ 2.0 (open source) for quality scoring.

**Delivery milestone:** Engine SDK integration + ingestion pipeline T-540; adjudication workbench T-480; accuracy certification T-300.

**Test strategy:** Accuracy benchmarking (FAR/FRR/throughput) against IEBC ground-truth set by independent lab; load test at 30M records; chaos test (engine pod kill, match-result replay); penetration test of confidential compute boundary.

**Compliance anchor:** FBI IAFIS IQS; ANSI/NIST-ITL 1-2011 Part 1 + ITL 2-2008; ISO 19794-2/5/6; DPA 2019 §25.

---

### M04 — VPRS (Voter Pre-Registration System)

**Purpose:** Public-facing portal for citizen-initiated voter pre-registration with IPRS identity verification, appointment booking, and QR handoff to the KIEMS device.

**Stack:** Python 3.13 async (back-end); FastAPI; PostgreSQL 17; Redis 7 (OTP store, session, rate limit); Next.js 14 (SSR/SSG, React) for web portal; React Native for companion mobile app; Twilio / Africa's Talking for SMS OTP; IPRS adapter (Askari integration reused).

**Deployment:** Kubernetes; auto-scales to 500K daily peak during registration windows. CDN-cached public pages; authenticated flows on origin. Rate-limited at Nginx ingress (100 req/min per IP for OTP endpoints).

**Team:** 1 lead, 2 back-end engineers, 2 front-end engineers, 1 mobile engineer, 1 QA. Total: 7.

**Build effort:** 14 person-months [LOW 10 / MID 14 / HIGH 18].

**Dependencies upstream:** IPRS (I01) for identity check; SMS gateway (I08). **Downstream:** M02 BVR backend (pre-reg merge); M01 BVR device (QR handoff).

**Three hardest problems:**
1. *IPRS reliability and rate limits.* Approach: resilient IPRS client with circuit breaker, retry with exponential backoff, graceful degradation to manual-entry flow when IPRS is unavailable; cache successful verifications with voter consent (24-hour TTL).
2. *QR handoff security (prevent session hijacking).* Approach: QR encodes a short-lived signed JWT (5-minute TTL, kit-bound claim); kit validates signature against IEBC intermediate CA before populating form.
3. *Accessibility across 11 language markets.* Approach: message catalogue with professional translation; Next.js i18n routing; screen-reader-first component design; field trials with disabled-voter advocacy groups.

**Key third-party:** IPRS (KNBS/NIIMS REST API, adapter reused from Datacraft Askari); Africa's Talking or Twilio for SMS OTP; next-i18next for localisation.

**Delivery milestone:** VPRS live (public access) T-600; appointment-booking module T-540; QR handoff tested end-to-end T-480.

**Test strategy:** Unit ≥80%; IPRS integration test against sandbox; WCAG 2.1 AA automated audit (axe-core); load test at 500K concurrent users (k6); pen test before public launch.

**Compliance anchor:** Elections Act §6A (pre-registration as pathway to enrolment); DPA 2019 §31 (consent management); Constitution Art 83(1) (citizen's right to register).

---

### M05 — BVI Device Client (Biometric Voter Identification, polling day)

**Purpose:** Authenticate each voter at their assigned polling station on election day; enforce one-kit-one-station and geo-fence; transmit poll-book events to back-end.

**Stack:** Kotlin + Jetpack Compose (Android 13+); same encrypted local store as M01; biometric SDK wrappers; kit-cert signed event queue. Legacy Android Java layer for older fleet.

**Deployment:** Kit-side APK; loaded at kit-prep time; station-QR bound; no network dependency for core auth workflow.

**Team:** 1 mobile lead, 3 Android engineers, 1 biometric SDK specialist, 1 QA, 1 SRE (OTA). Total: 7.

**Build effort:** 16 person-months [LOW 12 / MID 16 / HIGH 22]. Shared codebase primitives with M01 reduce incremental effort.

**Dependencies upstream:** M06 BVI backend (register bundle load at kit-prep time). **Downstream:** M06 (poll-book events in real time + deferred); M10 RTS backend (result-type trigger, not data flow — separate).

**Three hardest problems:**
1. *Register bundle distribution to 65,000+ kits within the final 24-hour pre-election prep window.* Approach: content-addressed signed bundles pre-computed per station; kit downloads delta from regional edge node; P2P gossip fallback between kits in same constituency if edge node fails.
2. *Geo-fence enforcement under urban-canyon GPS degradation.* Approach: multi-GNSS (GPS + Galileo + BeiDou + GLONASS); indoor fallback to cell-tower triangulation; if both degrade, human supervisor override with biometric counter-sign and audit flag.
3. *Voter-not-found handling at scale (wrong station, register error, late transfer).* Approach: encrypted "all-station" lookup fallback (read-only) via back-end API for in-day re-routing; Returning Officer authority to admit via override with audit chain.

**Third-party:** Same biometric SDK as M01/M03; SQLCipher; GNSS library (Android location APIs + explicit satellite enumeration).

**Delivery milestone:** Core auth flow T-480; full poll-book event suite T-360; dress-rehearsal ready T-90.

**Test strategy:** Biometric accuracy test (FAR ≤ 0.01% at operating threshold); offline-endurance test (8-hour continuous with 700-voter sequence); geo-fence field test in 10 Kenyan constituencies; 14-day pre-election SLA test (24-hour repair target).

**Compliance anchor:** Elections Act §44A (biometric voter identification); Elections (Tech) Reg 2017 Reg 8; DPA 2019 §25.

---

### M06 — BVI Backend

**Purpose:** Generate and distribute per-station register bundles; manage kit whitelist/blacklist; provide polling-day operations dashboard; receive poll-book events.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Redis 7; Kafka (poll-book event stream); MinIO (register bundles); gRPC (high-throughput event ingestion from kits).

**Deployment:** Kubernetes; 99.99% availability target during polling window. Regional edge nodes (5 hubs) cache register bundles.

**Team:** 1 lead, 3 back-end engineers, 1 data engineer, 1 QA, 1 SRE. Total: 7.

**Build effort:** 18 person-months [LOW 14 / MID 18 / HIGH 24].

**Dependencies upstream:** M02 BVR backend (register data); PKI (P04) for kit cert validation. **Downstream:** M01/M05 kits; M16 ECC (ops dashboard); M20 inventory (kit telemetry).

**Three hardest problems:**
1. *60,000 concurrent kit sessions at p99 ≤ 100 ms auth on election day.* Approach: stateless auth JWT validation; Redis session cache; pre-computed register bundle checksums; sharded Kafka per-constituency partitioning.
2. *Kit revocation propagation in ≤ 60 seconds.* Approach: OCSP must-staple + short-lived revocation CRL pushed to all kits via MDM event; kit validates revocation status at every session start.
3. *MDM-like emergency remote wipe of a compromised kit.* Approach: kit polls MDM channel on heartbeat; wipe command signed by IEBC emergency key (HSM-backed); local wipe executes on next heartbeat acknowledgment; remote attestation confirms wipe.

**Delivery milestone:** Register bundle pipeline T-420; full ops dashboard T-300; 60K session load test T-180.

**Compliance anchor:** Elections Act §44A; Elections (Tech) Reg 2017 Reg 4 (system integrity).

---

### M07 — BVV (Biometric Voter Verification)

**Purpose:** Public inspection of the register during the statutory 30-day period; citizen search, objection filing, and privacy-protected audit.

**Stack:** Python 3.13 async; FastAPI; Next.js 14 (SSR, read-heavy); PostgreSQL 17 read replicas; Redis (rate limiting, per-IP quotas); Africa's Talking (SMS notifications).

**Deployment:** Kubernetes; CDN-cached for static pages; authenticated search on origin with strict rate limits (10 lookups per IP per hour).

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Three hardest problems:**
1. *Privacy: prevent bulk export or scraping of the voter register.* Approach: per-IP + per-session rate limits enforced in Redis; bot detection (Turnstile/hCaptcha); no bulk API; response provides only the searched record; audit every lookup.
2. *Objection workflow with legal standing.* Approach: objection form captures objector biographic + reason + supporting document; case assigned to RO for decision within statutory window; petitioner notified by SMS; case audit log accessible to objector.
3. *Accessibility for voters who cannot use digital channels.* Approach: USSD short-code for feature-phone lookup; physical kiosk mode (accessible to assisted voters at constituency offices).

**Delivery milestone:** Public portal live T-600; objection workflow T-540; USSD channel T-480.

**Compliance anchor:** Elections Act §18 (register inspection); DPA 2019 §26; Constitution Art 83(3).

---

### M08 — RTS Device Client (Results Transmission at Kit)

**Purpose:** Scan Form 34A (and variants 35A–38A) on election day; embed per-station digital signature; transmit over VPN; operate fully offline with signed retry queue.

**Stack:** Kotlin + Jetpack Compose (Android 13+); camera-based document scanner (OpenCV + ML Kit for perspective correction); PDF/A generation; kit-cert signed event queue; VPN client integration. On-device ICR using TFLite for numerical field pre-fill (advisory, not authoritative).

**Deployment:** Kit-side APK. Loaded at kit-prep time alongside M05 BVI (separate app on same device).

**Team:** 1 mobile lead, 2 Android engineers, 1 ML/ICR engineer, 1 QA. Total: 5.

**Build effort:** 14 person-months [LOW 11 / MID 14 / HIGH 18].

**Three hardest problems:**
1. *Form scan quality under field conditions (poor lighting, angled, worn forms).* Approach: adaptive perspective correction with OpenCV; quality score gate before submission; retry prompt with guidance; augmented preview for PO to confirm before signing.
2. *Per-form digital signature binding image to tally numbers.* Approach: PDF/A produced locally; SHA-256 hash of PDF embedded in QR code on paper form at scan time; QR also signed by kit private key; back-end verifies signature on receipt.
3. *Thundering-herd at station-close when all 55,393 stations transmit within a narrow window.* Approach: per-kit randomised jitter (0–10 min) built into retry scheduler; sharded Kafka per-constituency on back-end; back-pressure visible to kit (retry with ETA shown to PO).

**Delivery milestone:** Core scan + sign + transmit T-480; full form variants (34A–38A) T-360; load tested T-180.

**Compliance anchor:** Elections Act §39(1C) (results transmission); Elections (Tech) Reg 2017 Reg 10.

---

### M09 — RTS Returning Officer (Constituency Tally Module)

**Purpose:** Vote count entry at constituency level; aggregation into Form 34B; party agent co-signing; verification before national submission.

**Stack:** Python 3.13 async (back-end API); React + Vite (ops front-end); PostgreSQL 17; Redis; PDF generation (reportlab / WeasyPrint); digital signature service (HSM via M10 back-end).

**Deployment:** Constituency workstation (web app, Chrome browser); back-end in Kubernetes. Offline-capable for core count entry (IndexedDB local draft); submits when connectivity restored.

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 14 person-months [LOW 11 / MID 14 / HIGH 18].

**Three hardest problems:**
1. *Party agent co-signing workflow (two-person integrity at constituency level).* Approach: multi-party signature ceremony; Form 34B locked after agent signatures; each signatory authenticates with FIDO2 key or biometric; any post-signature modification detectable via hash comparison.
2. *Derivation trail: Form 34B must be derivable from submitted Form 34A images.* Approach: 34B aggregation engine shows per-station subtotals; any manual override requires RO reason-entry + dual approval; override event written to Merkle log.
3. *Handling of disputed stations (Form 34A under objection).* Approach: station flagged as "provisional" in 34B; provisional total calculated with and without disputed station; both figures published with clear status.

**Delivery milestone:** Core tally entry T-420; co-signing workflow T-360; derivation trail verified T-300.

**Compliance anchor:** Elections Act §39(1C), §44; IEBC Returning Officer procedures.

---

### M10 — RTS Backend (National Aggregation)

**Purpose:** Receive, validate, and aggregate results from 55,393 polling stations through 290 constituencies and 47 counties to national level; 60,000 concurrent sessions; p99 ≤ 100 ms auth.

**Stack:** Python 3.13 async; FastAPI; gRPC (high-throughput kit ingestion); PostgreSQL 17 (HA, Patroni); Redis 7 (session cache, rate limit, dedup); Kafka (ingest fan-out to audit, anomaly detection, portal); MinIO (Form 34A signed image archive); ICR engine (Python + ONNX Runtime) for text extraction; HSM signing service integration.

**Deployment:** Kubernetes, primary + mirror; 20–60 replicas under election-day peak (HPA). Dedicated node pool for ICR workers. Connection pooling via PgBouncer.

**Team:** 1 lead, 4 back-end engineers, 2 ML/ICR engineers, 1 data engineer, 2 QA, 1 SRE. Total: 12.

**Build effort:** 30 person-months [LOW 24 / MID 30 / HIGH 38].

**Dependencies upstream:** M08 RTS device, M09 RTS-RO, PKI. **Downstream:** M11 portal, M14 reproducibility, M15 IAB, M16 ECC, M21 RLA, M26 anomaly detection.

**Three hardest problems:**
1. *60,000 concurrent sessions at p99 ≤ 100 ms auth.* Approach: stateless JWT validation in Redis; pre-issued session tokens per kit with sub-minute expiry; sharded Kafka (290 constituency partitions); async ingestion to avoid blocking session auth on DB write; horizontal scaling with Kubernetes HPA.
2. *ICR accuracy on Form 34A under real-world conditions (bad handwriting, ink bleed, fold marks).* Approach: ONNX Runtime ICR model trained on IEBC-provided historical forms; confidence thresholding — results below 0.90 confidence routed to human review queue; ICR output always labelled "auxiliary"; signed image is authoritative for any recount.
3. *Tamper-detection between kit and national total.* Approach: cryptographic chain — kit signs form at capture, back-end verifies signature on ingestion, Merkle hash of every form included in national tree; any substitution detectable without need for the original kit.

**Key third-party:** ONNX Runtime (open source, MIT) for ICR inference; PgBouncer for connection pooling; DDoS scrubbing at Cloudflare (or Akamai) upstream.

**Delivery milestone:** Ingestion API + auth T-420; ICR pipeline T-360; 60K load test T-180; full end-to-end T-120.

**Compliance anchor:** Elections Act §39(1C); Elections (Tech) Reg 2017 Reg 11 (stress test, load test, functional test); Constitution Art 138 (results declaration timeline).

---

### M11 — Public Results Portal

**Purpose:** Publish scanned Forms 34-38 with full digital provenance; live dashboards; 50M+ visitor capacity; DDoS-resilient via signed static snapshots.

**Stack:** Next.js 14 (SSG pre-bake for static snapshots; SSR for live API calls); React; Python 3.13 async (portal API back-end); PostgreSQL 17 (read replica only); Redis; CDN (Cloudflare or Akamai) for signed static asset serving. Multilingual: 11 languages.

**Deployment:** API back-end in Kubernetes; public pages pre-baked to signed static bundles pushed to CDN every 5 minutes during publication window. Origin DDoS causes no voter-facing outage; CDN snapshot serves stale-but-valid results until origin recovers.

**Team:** 1 lead, 2 back-end, 3 front-end, 1 QA, 1 SRE. Total: 8.

**Build effort:** 18 person-months [LOW 14 / MID 18 / HIGH 22].

**Three hardest problems:**
1. *50M unique visitors within 24 hours of first publication.* Approach: CDN-first architecture; static HTML pre-baked with embedded data; no client-side API call required to view result; JS enhancement only for live updates. Estimated CDN capacity 1 Tbps+.
2. *Merkle inclusion proof served to every visitor within 30 seconds.* Approach: inclusion proofs pre-computed per station; cached in CDN edge; served as static JSON; app verifies client-side against published root.
3. *Multilingual accessibility for 11 language communities.* Approach: professionalised translation (not machine-translation) for all result labels and UI copy; RTL support in CSS for potential Somali / Arabic communities; audio-read result option via Web Speech API.

**Delivery milestone:** Portal skeleton T-480; live results dashboard T-300; static-snapshot pipeline T-180; CDN load test T-120.

**Compliance anchor:** Elections Act §74 (public notification of results); Constitution Art 138(10) (results publication); DPA 2019 §26 (only public data, no voter PII).

---

### M12 — CRMS (Candidate Registration Management System)

**Purpose:** Party portal for online nominations; RO clearance workflow; ballot proof generation; gender/quota rule enforcement.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Redis; Next.js 14 (party portal); React + Vite (RO console); MinIO (document uploads); PDF ballot-proof generator (WeasyPrint).

**Deployment:** Kubernetes; usage is episodic (nomination windows), so autoscaling from 1 to 10 replicas. Document store on MinIO.

**Team:** 1 lead, 2 back-end, 2 front-end, 1 QA. Total: 6.

**Build effort:** 14 person-months [LOW 11 / MID 14 / HIGH 17].

**Three hardest problems:**
1. *Voter-register cross-check: verify each candidate is a registered voter without exposing the full register to the party portal.* Approach: single-record lookup API on M02 (returns boolean + candidate's station name, not full record); audit every cross-check.
2. *Gender rule enforcement (Elections Act §27 one-third gender rule at party list level).* Approach: real-time quota counter per party per elective position per county; submission blocked if quota breached; RO can grant exception with statutory reasoning.
3. *Ballot proof generation matching printers' colour-separation specs.* Approach: PDF/X-3 generation with CMYK colour profiles; printer-specific templates managed in configuration; proof requires dual-commissioner digital signature before dispatch.

**Delivery milestone:** Party portal live T-480; RO clearance workflow T-420; ballot proof generation T-360.

**Compliance anchor:** Elections Act §27 (gender rule); Elections Act §22 (nomination requirements); PPDA 2015 §§ relevant to procurement of ballot printing.

---

### M13 — Public Verifier App (Voter-Facing, Multi-Modal)

**Purpose:** Allow any Kenyan to verify their station's result via mobile app, PWA, USSD short-code, WhatsApp bot, or SMS. Converts 30M voters into independent auditors.

**Stack:** React Native (iOS + Android); Next.js 14 (PWA); Python 3.13 async (API back-end); Redis (rate limit, result cache); WhatsApp Business API (Twilio or Meta Cloud API); Africa's Talking USSD; Merkle verification library (Rust WASM module embedded in app for client-side proof verification). Multilingual: 11 languages (EN + SW + Kikuyu + Dholuo + Kalenjin + Kamba + Luhya + Somali + Meru + Giriama + Maasai).

**Deployment:** React Native app on app stores (Google Play + Apple App Store); PWA on portal domain; API in Kubernetes with CDN caching of result data. USSD via Africa's Talking. WhatsApp bot via webhook.

**Team:** 1 lead, 2 mobile (RN) engineers, 2 front-end (Next.js) engineers, 1 back-end, 1 WhatsApp/USSD integration engineer, 1 QA. Total: 8.

**Build effort:** 22 person-months [LOW 18 / MID 22 / HIGH 28].

**Three hardest problems:**
1. *Client-side Merkle proof verification without trusting any server.* Approach: Rust Merkle verification library compiled to WASM; embedded in the React Native app bundle; fetches root from IEBC transparency log + cross-checks against 4 independent mirrors (GitHub, IPFS, IAB, SNS); verification is fully local.
2. *Feature-phone USSD reach (2G/3G, no data plan, ~30% of Kenyan adult population).* Approach: USSD session encodes station code + contest selection; returns tally in a single USSD response (180 char budget); Africa's Talking aggregator with IEBC-assigned short-code; SMS fallback for results push after station closes.
3. *Scaling WhatsApp bot to millions of simultaneous queries at result publication time.* Approach: WhatsApp webhook processes queries asynchronously; results served from Redis cache (pre-warmed from M10 output); no back-end database hit for station lookup; rate-limit per WhatsApp number (10 queries per hour).

**Key third-party:** Rust (wasm-pack) for WASM Merkle library; Africa's Talking for USSD + SMS; Meta WhatsApp Cloud API or Twilio Conversations for WhatsApp; Google Play + Apple App Store distribution.

**Delivery milestone:** PWA + API T-360; React Native app T-300; USSD channel T-270; app store release T-120 (pre-election); results mode live T-0.

**Compliance anchor:** Elections Act §74; Strategy §PB-02 (public verifiability commitment); DPA 2019 §26 (no PII in public result query).

---

### M14 — Reproducibility Bundle

**Purpose:** A laptop-runnable `make reproduce` command that re-derives the national presidential tally from signed Form 34A images, deterministically, without IEBC servers, in under 4 hours on a commodity laptop.

**Stack:** Nix or Bazel with locked-hash inputs for hermetic build; Python 3.13 (tally derivation scripts); ONNX Runtime (ICR inference); PostgreSQL 17 dump (per-election signed snapshot); MinIO download of signed image archive; SHA-256 verification at every step; Merkle builder (Python + Rust extension).

**Deployment:** Distributed as a signed, versioned tarball on the IEBC public website, GitHub Releases, and IPFS. Not a Kubernetes workload. The bundle contains everything needed to run offline; the only external dependency is the signed image archive (downloadable from MinIO CDN URL embedded in bundle).

**Team:** 1 lead (build systems + crypto), 1 back-end engineer, 1 ML/ICR engineer, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Three hardest problems:**
1. *Hermetic reproducible build: same input must produce byte-identical output on any machine.* Approach: Nix flake with locked nixpkgs rev; all Python deps pinned with content hashes; ONNX model embedded in bundle; no runtime network calls during reproduction.
2. *Guiding a non-technical civil-society verifier through the process.* Approach: single-command `make reproduce`; progress bar with step descriptions; results printed to stdout as a human-readable tally table + machine-readable JSON; interactive guide document bundled.
3. *ICR divergence between bundle ICR and back-end ICR (different hardware, different ONNX runtime version).* Approach: bundle ships the identical ONNX model used in production (signed hash check); both paths produce identical results by construction; any divergence is a bug and generates a mismatch report rather than silent failure.

**Delivery milestone:** Bundle prototype T-300; full election-night bundle published T-0; post-election final bundle with all forms T+7.

**Compliance anchor:** Elections Act §74; Strategy §PB-02; Constitution Art 138 (verifiability of results).

---

### M15 — Independent Assurance Board Dashboard

**Purpose:** Read-only, hash-verified audit feed for the IAB; real-time visibility into all 28 modules' health, anomaly flags, and result integrity.

**Stack:** Python 3.13 async (read-only API gateway, no write paths); React + Vite (dashboard); PostgreSQL 17 read replica; Kafka consumer (IAB-dedicated topic, one-way from production event bus); Grafana (embedded for metrics panels); signature verification on every data item served.

**Deployment:** Separate Kubernetes namespace with no write paths to production. Read-only service account. All data items carry original production signatures; IAB dashboard re-verifies before displaying.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Three hardest problems:**
1. *Cryptographic isolation: IAB cannot be misled by a compromised production service serving false data.* Approach: IAB fetches Merkle roots from the public transparency log (not from the production API); compares against Kafka-streamed hashes; any divergence raises P0 alert to IAB members.
2. *Real-time anomaly context for non-technical IAB members.* Approach: anomaly feed from M26 pre-processed into natural-language summaries with severity ratings; IAB members can drill down to raw data; escalation channel direct to ECC (M16).
3. *IAB operational security — board members access from external networks.* Approach: IAB-specific Keycloak realm with FIDO2-only MFA; separate TLS termination; IAB namespace isolated from production namespace by Kubernetes NetworkPolicy and Linkerd AuthorizationPolicy.

**Delivery milestone:** Core dashboard T-360; live anomaly feed T-180; full election-day integration T-60.

**Compliance anchor:** Elections (Tech) Reg 2017 (independent verification requirement); Elections Act §44A; IEBC transparency commitment.

---

### M16 — Election Command Centre (E-SOC + COC + CAOC Unified)

**Purpose:** Unified operational picture for IEBC Commissioners, Secretariat, and Datacraft SRE during election day; real-time kit health, results flow, anomaly escalation, and incident management.

**Stack:** Python 3.13 async; FastAPI + WebSocket (real-time push); React + Vite (ops dashboard, map view); PostgreSQL 17; Redis Pub/Sub (real-time event fan-out); Kafka consumer; Grafana (embedded metrics); Mapbox GL JS (constituency map overlays); PagerDuty or equivalent (incident management).

**Deployment:** Kubernetes; air-gapped secondary deployment in IEBC physical war room (Anniversary Towers) with local mirror feed for network-resilience.

**Team:** 1 lead, 2 back-end, 2 front-end, 1 GIS engineer, 1 QA, 1 SRE. Total: 8.

**Build effort:** 20 person-months [LOW 16 / MID 20 / HIGH 25].

**Three hardest problems:**
1. *Single operational picture across 28 modules, 55,393 stations, and 65,000+ kits without cognitive overload.* Approach: progressive disclosure dashboard — national summary → county → constituency → station; colour-coded RAG status; drill-down on anomaly; map overlays with constituency boundaries (M18 integration).
2. *Incident management at scale (potential 1000+ simultaneous kit failures on election day).* Approach: auto-triage by severity and geography; bulk-action remediation (push config, revoke kit, escalate to field team); SLA timer per incident; integration with M23 grievance channel for citizen-reported incidents.
3. *War-room resilience if primary network fails.* Approach: air-gapped secondary ECC deployment in physical war room with 15-minute-max data lag from local Kafka mirror; war room operates autonomously during outage.

**Delivery milestone:** Dashboard MVP T-360; full module integration T-180; war-room secondary T-90; dress-rehearsal exercise T-60.

**Compliance anchor:** Elections (Tech) Reg 2017 Reg 4 (system integrity monitoring); IEBC's legal obligation to conduct a credible election.

---

### M17 — Speech Integrity Programme (Hate Speech + CIB + Deepfake Detection)

**Purpose:** Monitor public social-media and broadcast channels for election-related hate speech, coordinated inauthentic behaviour, and synthetic media (deepfakes) targeting the election; escalate to ECC and designated national authorities.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Kafka (alert events); OpenSearch (content index); ONNX Runtime + HuggingFace Transformers (multilingual hate-speech classifier, 11 Kenyan languages); MediaPipe or Sensity.ai SDK (deepfake detection); Twitter/X API, Meta API, YouTube Data API (ingestion); custom web crawler (Scrapy + Playwright for broadcast monitoring).

**Deployment:** Kubernetes with GPU node pool for ML inference. Isolated from election-data plane (no access to voter register or results during inference). Alert output only to M16 ECC and designated authorities.

**Team:** 1 lead (NLP/ML), 2 ML engineers, 2 back-end engineers, 1 QA. Total: 6.

**Build effort:** 20 person-months [LOW 16 / MID 20 / HIGH 26].

**Three hardest problems:**
1. *Multilingual hate-speech classification across 11 Kenyan language communities including mixed-language (Sheng, English-Kiswahili code-switching).* Approach: fine-tune a multilingual base model (mBERT or XLM-R) on a Kenya-specific labelled dataset; human annotation team for each language community; continuous active learning loop.
2. *Deepfake detection of audio-visual content in a politically contested environment where false positives can themselves cause harm.* Approach: ensemble of spatial + temporal artifact detectors; output is a confidence score, not a binary verdict; human reviewer mandatory before any public escalation; escalation path is to national authorities (NCA, NCIC), not to public.
3. *Legal and ethical boundary: election system must not become a censorship tool.* Approach: system only flags and escalates; no content removal; escalation recipients are designated statutory authorities (NCIC, NCA, DCI); M17 has no write access to any public channel; transparency report published weekly.

**Delivery milestone:** Classifier training complete T-360; CIB detection live T-270; deepfake pipeline T-180; election-day monitoring live T-30.

**Compliance anchor:** National Cohesion and Integration Act 2008; Kenya Information and Communications Act; NCIC mandate; Elections Act §6A; DPA 2019 §47 (data processing for public interest).

---

### M18 — Boundary Transparency (GeoJSON Explorer + Simulator + Fairness Metrics)

**Purpose:** Publish all electoral boundary data in open, machine-readable format; provide a browser-based boundary explorer and a population-fairness simulator.

**Stack:** Python 3.13 async (boundary API); FastAPI; PostgreSQL 17 + PostGIS (spatial data); Next.js 14 (boundary explorer); Mapbox GL JS or Leaflet (map rendering); Turf.js (client-side spatial analysis); R or Python (fairness metric computation); GeoJSON + TopoJSON formats; GDAL (spatial data pipeline).

**Deployment:** Kubernetes; read-heavy (public access); CDN-cached GeoJSON tiles; no auth required for public read.

**Team:** 1 lead (GIS specialist), 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Three hardest problems:**
1. *PostGIS performance for boundary intersection queries at national scale (290 constituencies, 1,450 wards, 55,393 stations).* Approach: pre-indexed GeoJSON tiles at multiple zoom levels; spatial indexes on all boundary tables; tile pre-bake job on boundary update.
2. *Fairness metric computation (population per elected seat, deviation from quota).* Approach: census population data joined to boundary data; Sainte-Laguë and largest-remainder computations; deviation from ideal seat population visualised as choropleth.
3. *Boundary change simulation: users want to test "what if ward X moved to constituency Y."* Approach: client-side simulation using Turf.js and locally cached GeoJSON; no write paths; simulation results are local, not persisted.

**Delivery milestone:** Boundary data loaded from IEBC gazette T-540; public explorer T-480; fairness metrics T-420.

**Compliance anchor:** Constitution Art 89 (boundary delimitation); IEBC Act §26; Elections Act §6A.

---

### M19 — Queue Transparency Programme

**Purpose:** Real-time queue status (wait time, turnout) at each polling station; citizen-facing via M13; operations-facing via M16 ECC.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Redis (per-station real-time counters); Kafka (poll-book events from M06); React + Vite (ops view); REST API consumed by M13.

**Deployment:** Kubernetes; lightweight; derives queue metrics from M06 poll-book event stream.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Three hardest problems:**
1. *Estimating wait time without requiring voters to check in before queuing.* Approach: rolling average voters-per-hour per station from poll-book events; queue length estimated from turnout rate and station capacity; updated every 5 minutes.
2. *Privacy: aggregate data only, no individual voter identification in public feed.* Approach: public API returns station-level aggregates only (count, average wait); no individual event data; per-station granularity is already public information.
3. *Station-specific anomaly: queue wait > 90 minutes triggers ECC alert.* Approach: threshold monitoring in Redis; alert published to M16 ECC and M26 anomaly detection via Kafka event.

**Delivery milestone:** Queue metrics pipeline T-300; citizen API T-270; ECC integration T-180.

**Compliance anchor:** Elections Act §55 (PO duty to manage station); IEBC operational standards.

---

### M20 — Inventory Management System (RFID + Geofence + Custody Chain)

**Purpose:** Full lifecycle tracking of approximately 65,287 KIEMS kits and all sub-components from OEM factory to polling station to post-petition disposition, with RFID, biometric handoffs, and QR-verifiable seal scans.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Redis; Kafka (telemetry stream); React + Vite (logistics console); React Native (field officer mobile app for custody handoffs); RFID reader integration (ISO 14443 + ISO 15693 via HTTP gateway); MinIO (seals, documentation); GPS tracking integration (kit telemetry from M05/M06).

**Deployment:** Kubernetes; field officer mobile app via React Native (APK + iOS IPA); RFID gateway integrations via hardware HTTP bridges.

**Team:** 1 lead, 2 back-end, 1 mobile engineer, 1 RFID/hardware integration specialist, 1 QA, 1 SRE. Total: 7.

**Build effort:** 20 person-months [LOW 16 / MID 20 / HIGH 26]. Cross-references INVENTORY-MANAGEMENT.md for detailed scope.

**Three hardest problems:**
1. *Full chain-of-custody for 65,000+ multi-component kits with biometric handoff enforcement.* Approach: every custody transfer requires biometric (fingerprint) verification of transferor and transferee via field officer app; transfer event signed and written to tamper-evident log; RFID scan confirms physical kit presence.
2. *Geofence enforcement: kit must not leave its assigned constituency store before distribution day.* Approach: kit GPS telemetry from M05/M06; geofence polygon per storage location; any out-of-polygon event triggers alert to M16 ECC and custody chain officer.
3. *Component-level tracking (fingerprint sensor, iris module, battery all have independent serials).* Approach: hierarchical asset model (kit → sub-component → serial); any sub-component substitution generates a new custody event requiring supervisor biometric counter-sign; divergence from OEM build manifest flagged.

**Delivery milestone:** Kit master registry T-600; distribution tracking live T-300; election-day real-time telemetry T-60.

**Compliance anchor:** PPDA 2015 (public asset management); Elections (Tech) Reg 2017 (kit integrity); IEBC procurement rules.

---

### M21 — RLA Audit Tool (SHANGRLA)

**Purpose:** Run a Risk-Limiting Audit using Philip Stark's SHANGRLA framework against the paper ballot manifest and Form 34A images after election day.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; React + Vite (auditor console); SHANGRLA Python library (Stark, UC Berkeley, open source); statistical sampling framework; MinIO (ballot manifest, Form 34A archive access read-only).

**Deployment:** Kubernetes; post-election-day workload; no election-day dependency. Auditor access via separate IAB-scoped Keycloak realm.

**Team:** 1 lead (statistical specialist), 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Three hardest problems:**
1. *SHANGRLA requires a ballot manifest (total ballots cast per station). Ensuring accuracy of the manifest.* Approach: manifest derived from signed Form 34A data (authoritative); cross-checked against poll-book event stream; any discrepancy requires RO explanation before RLA proceeds.
2. *Sampling audit-board composition and chain-of-custody for selected ballots.* Approach: audit-board assigned by IEBC per county; sampled ballots tracked by ballot-ID in M20 inventory; physical retrieval logged in M20 custody chain.
3. *Statistical stopping rule tuned for Kenyan election context (multi-contest, simultaneous positions).* Approach: SHANGRLA supports multi-contest audits natively; risk limit (alpha) configurable by IEBC statistician; default alpha = 0.05; escalation to full hand-count if RLA does not stop within budget.

**Delivery milestone:** RLA tool ready T-120; full integration with M10 manifest T-60; post-election RLA run T+7 through T+30.

**Compliance anchor:** Elections Act §74 (audit authority); Auditor-General Act; academic best practice (SHANGRLA, Stark 2020).

---

### M22 — Campaign Finance Transparency Portal

**Purpose:** Publish candidate and party campaign finance disclosures; flag statutory limits; enable public inspection.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; React + Next.js 14 (public portal); React + Vite (party filing console); Redis; MinIO (uploaded disclosures); OpenSearch (full-text search across disclosures).

**Deployment:** Kubernetes; public portal CDN-cached; party console requires FIDO2 MFA.

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Three hardest problems:**
1. *Statutory limit enforcement across multiple elective positions simultaneously (presidential, gubernatorial, MP each have different limits).* Approach: configurable limit table per position tier; running total per candidate per tier; hard block at 90% of limit with warning; final submission blocked at limit.
2. *Disclosure verification: ensuring uploaded documents are genuine.* Approach: document authenticity is out of scope (IEBC verifies physically); system stores, indexes, and publishes what is submitted; submission timestamp and submitter identity are auditable.
3. *Public accessibility of potentially complex financial data for non-specialist voters.* Approach: visual summary per candidate (bar chart vs limit); natural-language summary ("KES X raised, Y% of limit"); downloadable CSV for data journalists.

**Delivery milestone:** Filing portal T-480; public portal T-420; statutory-limit enforcement T-360.

**Compliance anchor:** Elections Act §§103-106 (campaign finance); Elections (General) Regulations 2017; PPDA 2015.

---

### M23 — Grievance Channels (Multi-Modal Intake + Triage)

**Purpose:** Multi-modal election-grievance intake (web, app, SMS, USSD, WhatsApp, phone transcription) with AI-assisted triage and tracked resolution.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Redis; Kafka (inbound grievance events); React + Vite (triage console); React Native (field officer app); Africa's Talking (SMS + USSD); WhatsApp Business API; whisper.cpp (on-premise speech-to-text for phone intake); spaCy + custom NLP (classification + entity extraction); Elasticsearch / OpenSearch (case search).

**Deployment:** Kubernetes; handles surge capacity on election day (estimated 100K+ grievances). Triage console for IEBC regional teams.

**Team:** 1 lead, 2 back-end, 1 NLP/ML engineer, 1 mobile engineer, 1 front-end, 1 QA. Total: 7.

**Build effort:** 18 person-months [LOW 14 / MID 18 / HIGH 23].

**Three hardest problems:**
1. *Channel unification: same grievance filed via SMS, WhatsApp, and web by different people about the same incident.* Approach: entity extraction (station code, candidate, incident type) used to cluster likely duplicates; duplicate candidates surfaced to triage officer for manual merge decision.
2. *AI triage without automating legal decisions.* Approach: NLP classifies grievance category (violence, procedural error, kit failure, voter intimidation, results tampering) and routes to the correct IEBC team; AI output is advisory; every routing decision is reviewable and overrideable.
3. *Surge capacity: election day could produce 100K+ simultaneous submissions.* Approach: async ingestion via Kafka; acknowledgement to citizen is immediate (ticket ID); actual triage queue processes behind; triage console shows queue depth to IEBC supervisors.

**Delivery milestone:** Core intake + triage T-360; all channels live T-270; election-day load tested T-90.

**Compliance anchor:** Elections Act §75 (dispute resolution); IEBC Act §4(c); DPA 2019 (complainant PII protection).

---

### M24 — Diaspora Voting Platform

**Purpose:** Secure remote voting for Kenyan diaspora in eligible jurisdictions; end-to-end encrypted ballot; voter authentication via biometric match against ABIS.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Redis; Next.js 14 (voter-facing portal); React Native (companion app); biometric capture SDK (WebRTC-based facial liveness + fingerprint via USB reader at diaspora voting centre); end-to-end encrypted ballot transmission; HSM-backed ballot encryption and decryption ceremony; homomorphic encryption candidate (Zama TFHE or Microsoft SEAL) for privacy-preserving tally.

**Deployment:** Kubernetes; diaspora voting centres in targeted jurisdictions; internet-connected but with HSM-backed encryption. Kit-grade security for ballot encryption.

**Team:** 1 lead (cryptography specialist), 2 back-end, 1 front-end, 1 cryptographic engineer, 1 QA. Total: 6.

**Build effort:** 22 person-months [LOW 18 / MID 22 / HIGH 28] — homomorphic tally is a high-risk R&D item.

**Three hardest problems:**
1. *Voter authentication without physical ID document in foreign jurisdiction.* Approach: biometric match (facial liveness via WebRTC) against ABIS; backup: facial + ID document upload for manual adjudication at diaspora centre; no vote recorded until match confirmed.
2. *End-to-end ballot secrecy: nobody, including IEBC, can link a ballot to a voter.* Approach: threshold encryption scheme (3-of-5 IEBC commissioner keys); ballot encrypted at client; decryption requires quorum; mix-net (Verificatum open source) for unlinkability before tally.
3. *Network resilience for diaspora voting centres in jurisdictions with poor connectivity.* Approach: embassy/consulate venues as voting centres; encrypted ballot queued locally (USB key backup) if network fails; ballot submitted on reconnect within voting window.

**Delivery milestone:** Architecture validated T-480; prototype T-360; security audit of crypto design T-270; regulatory clearance T-180; pilot (selected cities) T-120.

**Compliance anchor:** Elections Act §44 (as amended to permit diaspora voting); Constitution Art 83; IEBC diaspora registration roll.

---

### M25 — Special-Voter Workflows (Prisoners + Hospitals + KDF + Refugees)

**Purpose:** Configure and manage polling-station-level workflows for special voter categories requiring non-standard procedures: prisoners, hospitalised voters, Kenya Defence Forces personnel, and registered refugees.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; React + Vite (admin console for RO special-category management); integration with M05 BVI device (special mode flag); integration with M02 register (special-category flag propagation).

**Deployment:** Kubernetes; small-scale service but high regulatory sensitivity. Separate Keycloak realm for special-category officers.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Three hardest problems:**
1. *Privacy: prisoner and patient records must not cross-contaminate with the general register in any query response.* Approach: special-category flag on voter record; query APIs for special categories require elevated RBAC role; special-category voter data in separate PostgreSQL schema with row-level security.
2. *KDF and refugee voters may be in remote or insecure locations; kit connectivity is unreliable.* Approach: extended offline queue on M05 kit; special-mode extends the offline submission window from 4 hours to 24 hours for designated special-category stations.
3. *Maintaining chain of custody for assisted voting (voter cannot operate kit unaided).* Approach: assisted-voting flag on poll-book event; two-officer witness requirement logged with biometrics; paper record countersigned.

**Delivery milestone:** Special-category register extraction T-480; BVI special mode T-360; RO console T-300.

**Compliance anchor:** Elections Act §50 (special voters); Prison Act; KDF Act; Refugees Act; DPA 2019 §26 (sensitive personal data).

---

### M26 — Anomaly and Fraud Detection Stack (Seven Layers)

**Purpose:** Real-time detection of statistical anomalies, potential fraud patterns, and system-integrity violations across results, kit behaviour, biometric patterns, network events, financial disclosures, social media, and physical incidents.

**Stack:** Python 3.13 async; FastAPI; Kafka (all seven layer feeds); Apache Flink or Kafka Streams (real-time stream processing); PostgreSQL 17; Redis (alert dedup); ONNX Runtime (ML anomaly models); React + Vite (analyst console in M16 ECC); PagerDuty (escalation).

Seven layers:
1. Statistical results anomaly (Benford's Law, turnout outliers, late-reporting station patterns).
2. Kit-behaviour anomaly (unusual session patterns, repeated failed auth, off-schedule submission).
3. Biometric anomaly (same template submitted from multiple kits, velocity attacks).
4. Network anomaly (unusual traffic patterns, DDoS precursors, VPN tunnelling anomalies).
5. Campaign finance anomaly (spending velocity spikes, large-cash-equivalent transfers near election).
6. Social media signal (M17 feed integration: coordinated spikes correlating with on-ground incidents).
7. Physical incident correlation (M23 grievance + M16 ECC field reports spatially and temporally clustered).

**Deployment:** Kubernetes with dedicated Kafka Streams or Flink cluster. GPU node pool for ML inference. Outputs to M16 ECC via Kafka alert topic.

**Team:** 1 lead (data science / fraud detection), 3 data engineers, 2 ML engineers, 1 back-end, 1 QA. Total: 8.

**Build effort:** 24 person-months [LOW 20 / MID 24 / HIGH 30].

**Three hardest problems:**
1. *False-positive rate: an anomaly flag during a live election can itself cause harm (panic, premature media reports).* Approach: graduated alert levels (observe → investigate → escalate → public); only Level 3 escalations are visible outside IEBC; all Level 1/2 alerts are internal only; false-positive rate target ≤ 5% at Level 3.
2. *Benford's Law misapplication: it is not valid for small precinct-level counts.* Approach: Benford test applied only at constituency level and above (sufficient N); precinct-level uses turnout-rate comparison against historical and demographic expectations; all statistical tests documented with their assumptions and limitations.
3. *Cross-layer correlation: the most dangerous fraud involves coordinated anomalies across multiple layers simultaneously.* Approach: correlation engine joins alerts from all seven layers with a temporal window; compound alert score = weighted sum across layers; compound score > threshold triggers IAB notification.

**Delivery milestone:** Layer 1-2 (statistical + kit) T-360; Layers 3-5 T-270; Layers 6-7 (social + physical) T-180; full correlation engine T-120.

**Compliance anchor:** Elections Act §44 (system integrity); NCA mandate; IEBC legal obligation to conduct free and fair election.

---

### M27 — Kit-Side Intent Preservation (Pre-Mark Validation + Adjudication)

**Purpose:** On the BVI/RTS kit, validate and preserve voter intent during the ballot-marking phase; flag ambiguous marks for presiding-officer adjudication before the ballot is counted.

**Stack:** Kotlin + Jetpack Compose (Android); TFLite (on-device ballot-mark classification model); shared library with M05 BVI and M08 RTS kits; adjudication UI overlay.

**Deployment:** Kit-side module embedded in M08 RTS APK; activated at Form 34A scan review step.

**Team:** 1 lead, 1 ML engineer, 1 Android engineer, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Three hardest problems:**
1. *Distinguishing voter intent from stray marks on a hand-counted ballot form under field conditions.* Approach: TFLite mark-classification model trained on labelled Kenya ballot images (including historical ambiguous cases); confidence threshold — high-confidence marks are auto-accepted; low-confidence marks are flagged for PO adjudication.
2. *PO adjudication UI must be simple and non-leading.* Approach: shows ballot image crop with mark highlighted; PO chooses from "valid mark for [candidate]", "spoilt", or "ambiguous — refer to RO"; choice recorded with PO biometric and timestamp.
3. *Auditability: adjudication decisions must be reviewable in a petition.* Approach: adjudication event written to kit local queue, signed, and submitted to M10 back-end; stored in Merkle log; retrievable in evidence pack within 5 minutes.

**Delivery milestone:** TFLite model trained T-360; adjudication UI T-300; field-trial tested T-120.

**Compliance anchor:** Elections Act §39(1C) (valid ballot determination); Elections (General) Regulations 2017 Rule 59 (spoilt ballots).

---

### M28 — AI Monitoring Stack (On-Device + Central with Sovereignty)

**Purpose:** Monitor all AI/ML model behaviour across the IEMS in real time; detect model drift, adversarial inputs, and unexpected outputs; enforce data-sovereignty constraints on all ML workloads.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Kafka (model inference event stream); MLflow (model registry, versioning); Evidently AI (drift detection, open source); ONNX Runtime (central inference audit); React + Vite (ML ops console); custom sovereignty enforcement layer (all inference requests audited for data classification compliance).

**Deployment:** Kubernetes central component; on-device component is a lightweight Python daemon embedded in kit APK's companion service layer (minimal footprint, ≤ 5 MB RAM).

**Team:** 1 lead (MLOps), 2 ML engineers, 1 back-end, 1 QA. Total: 5.

**Build effort:** 14 person-months [LOW 11 / MID 14 / HIGH 18].

**Three hardest problems:**
1. *On-device model-drift detection with constrained compute.* Approach: lightweight statistical test (population stability index) computed on rolling inference distribution; divergence from training distribution above threshold triggers alert to central M28 service; heavy drift analysis happens centrally.
2. *Data sovereignty: biometric templates must never pass through any ML inference path that has external data egress.* Approach: sovereignty enforcement layer intercepts all inference requests; any request containing biometric data (classified by data-type tag) that routes to a model with an external endpoint is blocked and logged as a P0 security event.
3. *Adversarial input detection on facial recognition and ballot-mark classification models.* Approach: input validation pipeline checks for known adversarial perturbation patterns; anomalous inputs quarantined and routed to human review; no adversarial input is counted without human sign-off.

**Delivery milestone:** Model registry T-480; central drift monitoring T-360; on-device monitoring T-270; sovereignty enforcement T-180.

**Compliance anchor:** DPA 2019 §47 (automated decision-making rights); AI governance principles; IEBC data sovereignty obligation.

---

### M29 — Mobile Citizen Verify (IEBC Verify App)

**Purpose:** Public-facing Flutter app for any Kenyan: register lookup, polling-station finder, Merkle-proof result verification, candidate browser, civic information, and suspected-issue reporting. Flagship transparency expression of IEMS.

**Stack:** Flutter 3.x (Dart) for iOS 14+ / Android 8+ / PWA; Rust WASM module for client-side Merkle verification (same library as M13); Python 3.13 async FastAPI back-end; Redis 7 (rate limit, result cache); PostgreSQL 17 read replica; CDN for static assets; Africa's Talking USSD fallback.

**Deployment:** App stores (Google Play + Apple App Store) + signed APK from iebc.or.ke; back-end in Kubernetes, CDN-cached result data. Not MDM-managed; citizen-owned devices.

**Team:** 1 mobile lead, 2 Flutter engineers, 1 back-end, 1 Rust/WASM engineer, 1 QA. Total: 6.

**Build effort:** 18 person-months [LOW 14 / MID 18 / HIGH 24].

**Dependencies upstream:** M10 (result data), M02 (register lookup), M14 (Merkle roots), M11 (transparency log). **Downstream:** M11, M23 (issue reports), M64 (voter-list inspection link).

**Three hardest problems:**
1. *Merkle proof verification without trusting any server.* Approach: Rust WASM library fetches root from IEBC transparency log + cross-checks against 4 independent mirrors; all verification local.
2. *App store delays risking pre-election release.* Approach: PWA as always-available fallback; USSD shortcode for feature phones; staged Play Store review request at T-180.
3. *Accessibility for low-literacy users.* Approach: audio-read workflow for result display; icon-first navigation; WCAG 2.2 AA; TalkBack/VoiceOver support.

**Key third-party:** Rust wasm-pack; Africa's Talking USSD; Google Play + Apple App Store.

**Delivery milestone:** Beta at T-300; app-store release T-120; results mode live T-0.

**Test strategy:** Unit ≥80%; Merkle verification test against known test vectors; USSD channel integration test; WCAG 2.2 AA automated + manual; load test at 10M concurrent sessions.

**Compliance anchor:** DPA 2019 §26 (no PII in public result query); Elections Act §74; Constitution Art 138.

---

### M30 — Mobile Party Agent App

**Purpose:** Accredited party-agent tool: station authentication, independent Form 34A verification, cryptographic counter-signing, and structured incident reporting.

**Stack:** Flutter 3.x (Android 10+ / iOS 15+); distributed via IEBC MDM (Google Play Managed + Apple Business Manager); Python 3.13 async FastAPI back-end; PostgreSQL 17; Kafka (incident events); Redis.

**Deployment:** MDM-managed on party-supplied or IEBC-provisioned devices. Back-end in Kubernetes under field-mobile-svc.

**Team:** 1 mobile lead, 2 Flutter engineers, 1 back-end, 1 QA. Total: 5.

**Build effort:** 14 person-months [LOW 11 / MID 14 / HIGH 18].

**Dependencies upstream:** M10 (Form 34A data), M09 (constituency tally), M43 (agent credential). **Downstream:** M23 (incidents), M15 (IAB anomaly feed).

**Three hardest problems:**
1. *Cryptographic counter-signing on a citizen device.* Approach: agent private key generated and stored in device secure enclave at MDM provisioning; signing is local; public key registered with IEBC PKI at accreditation.
2. *Form 34A verification against physical form in hand.* Approach: QR-code scan of Form 34A links to signed PDF in M10; agent app verifies hash match locally; any mismatch is an immediate P1 incident.
3. *MDM revocation of compromised agent credentials.* Approach: agent credential has 24-hour TTL; MDM can push immediate revocation event; app blocks signing on revoked credential.

**Delivery milestone:** Core verification flow T-360; counter-signing T-270; MDM deployment T-120.

**Test strategy:** Unit ≥80%; end-to-end Form 34A verification test; revocation propagation test; MDM integration test.

**Compliance anchor:** Elections Act §83A (party agent rights); DPA 2019 §26.

---

### M31 — Mobile Poll Clerk Companion

**Purpose:** Deputy PO and polling-clerk companion running on the same KIEMS device (separate app): reference materials, incident logging, voter-lookup backup when device is single-user busy, paper-backup workflow guidance, kiosk mode for voter-list inspection.

**Stack:** Flutter 3.x (Android 10+ / iOS 15+); kit-side APK alongside M05/M08; local SQLite for reference data; Python 3.13 async FastAPI back-end.

**Deployment:** Kit-side APK via MDM at kit-prep time. Back-end in Kubernetes (biometric-edge service, non-K8s APK).

**Team:** 1 mobile lead, 2 Flutter engineers, 1 QA. Total: 4.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Dependencies upstream:** M06 (register bundle), M05 (station state). **Downstream:** M23 (incidents), M19 (queue updates).

**Three hardest problems:**
1. *Coexistence with M05/M08 on same device without resource conflicts.* Approach: separate Android user profile per app; explicit CPU/memory quotas via Android WorkManager; M31 yields to M05 biometric capture.
2. *Offline reference material freshness.* Approach: reference bundle versioned and distributed at kit-prep time; delta-update via regional edge node at last connectivity window.
3. *Paper-backup workflow under stress.* Approach: guided form wizard with required fields; barcode scan of pre-printed station code; signed locally for later merge.

**Delivery milestone:** Core reference + incident T-360; kiosk mode T-270; full kit integration T-120.

**Test strategy:** Coexistence test with M05/M08 on KIEMS hardware; offline endurance test (8 hours); WCAG 2.2 AA audit.

**Compliance anchor:** Elections Act §44A; IEBC Presiding Officer procedures.

---

### M32 — Mobile Field Support App

**Purpose:** Datacraft field-engineer mobile console: support-ticket queue, remote diagnostics against kits, spare-parts lookup in M20, KEBS/warranty claim initiation, escalation routing, end-of-day reporting.

**Stack:** Flutter 3.x (Android 10+ / iOS 15+); MDM-managed; Python 3.13 async FastAPI back-end; PostgreSQL 17; Kafka (ticket events); Redis; M20 inventory API integration.

**Deployment:** MDM-managed on ~200 Datacraft field-engineer devices. Back-end in Kubernetes under field-mobile-svc.

**Team:** 1 mobile lead, 1 Flutter engineer, 1 back-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M20 (spare parts), M16 (ticket assignments), M06 (kit diagnostics). **Downstream:** M20 (spare consumption), M16 (escalation events).

**Three hardest problems:**
1. *Remote diagnostics on a kit over limited telco VPN bandwidth.* Approach: kit exposes diagnostics endpoint (local network when engineer is on-site) returning signed health JSON; M32 renders structured diagnostic report without needing cloud round-trip.
2. *Spare-parts lookup with stale M20 data on mobile.* Approach: M20 regional cache updated every 5 minutes; M32 shows cache-freshness timestamp; critical shortages push via FCM.
3. *Warranty-claim evidence capture in field.* Approach: photo-capture + EXIF geotag + kit UID binding; signed evidence bundle uploaded to M20 on connectivity; claim lodged against Mechrevo OEM API.

**Delivery milestone:** Core ticket + diagnostics T-360; M20 integration T-270; warranty workflow T-180.

**Test strategy:** M20 API contract test; offline diagnostics test on real KIEMS hardware; 200-engineer concurrent-session load test.

**Compliance anchor:** PPDA 2015 (warranty claims); Datacraft SLA obligations in contract.

---

### M33 — Mobile Personnel Dispatch App

**Purpose:** Dispatch and attendance for ~250K polling-day workers: push assignment, GPS-verify check-in, status pings, incident reporting, GPS-verified check-out, payroll trigger to M55.

**Stack:** Flutter 3.x (Android 8+ / iOS 14+ / PWA for BYOD); MDM + signed APK; Python 3.13 async FastAPI back-end; PostgreSQL 17; Kafka; Redis; Google Maps + OpenStreetMap geofence; M55 payroll API integration.

**Deployment:** MDM for IEBC-issued devices; signed APK for casual-worker BYOD. Back-end in Kubernetes under field-mobile-svc with 250K concurrent session headroom.

**Team:** 1 mobile lead, 2 Flutter engineers, 1 back-end, 1 GIS engineer, 1 QA, 1 SRE. Total: 7.

**Build effort:** 14 person-months [LOW 11 / MID 14 / HIGH 18].

**Dependencies upstream:** M16 (assignments), M20 (kit-to-station mapping). **Downstream:** M55 (payroll trigger), M16 (attendance dashboard).

**Three hardest problems:**
1. *GPS spoofing of check-in location.* Approach: multi-GNSS validation; device attestation (SafetyNet / Play Integrity); anomaly flag when check-in GPS diverges from known station coordinates by >200m; M16 ECC alert.
2. *250K concurrent sessions at station-open time.* Approach: stateless JWT auth; pre-issued session tokens; Kafka fan-out per county partition; HPA to 80 replicas; load tested at T-180.
3. *BYOD device diversity (Android 8–14, iOS 14+, KaiOS).* Approach: PWA as fallback for KaiOS; minimum-viable USSD channel for check-in on feature phones; full app only on Android/iOS.

**Delivery milestone:** Core check-in/check-out T-360; payroll integration T-270; 250K load test T-180.

**Test strategy:** 250K concurrent session load test (k6); GPS spoofing penetration test; M55 payroll integration test; BYOD compatibility matrix CI.

**Compliance anchor:** Labour Act (employment records); DPA 2019 §26 (GPS data purpose limitation); Elections Act (polling personnel accountability).

---

### M34 — Mobile Observer App

**Purpose:** Accredited independent and international observer tool: structured incident reporting, cryptographic result verification at any polling station visited, observer findings feed to transparency layer.

**Stack:** Flutter 3.x (Android 10+ / iOS 15+); MDM-managed; Python 3.13 async FastAPI back-end; PostgreSQL 17; Redis; Merkle client library.

**Deployment:** MDM on IEBC-provisioned observer devices (~500 accredited observers). Back-end under field-mobile-svc.

**Team:** 1 mobile lead, 1 Flutter engineer, 1 back-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M43 (credential), M10 (result data), M14 (Merkle roots), M15 (IAB feed). **Downstream:** M23 (incidents), M15 (observer findings feed).

**Three hardest problems:**
1. *Data-access tiering: observers see public data only, not PII.* Approach: observer role in RBAC has read-only access to Form 34A images and tallies; no biometric or voter PII endpoints visible.
2. *Multilingual interface for international observers (EU EOM, Carter Center, AU).* Approach: EN, FR, AR, SW; language selected at MDM provisioning; terminology aligned with IEBC observation mission briefing.
3. *Signed observer reports as court-admissible evidence.* Approach: report signed with observer device key at time of submission; timestamp from HSM TSA; report immutable once submitted.

**Delivery milestone:** Core incident + verification T-360; multilingual T-270; MDM deployment T-120.

**Test strategy:** Role-isolation test (observer cannot access PII endpoints); Merkle verification test; multilingual UI review; 500-user concurrent session test.

**Compliance anchor:** Elections Act §86 (observer rights); IEBC observer accreditation framework.

---

### M35 — CVR: Continuous Voter Register Maintenance

**Purpose:** Inter-election continuous maintenance of the voter register: life-event ingestion from Civil Registration Service and NRB (births, deaths, name changes, address changes), ward-transfer workflow, accuracy-metrics dashboard, and change-audit trail.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (same bvr-svc schema); Kafka (life-event ingest); SQLAlchemy 2.x async; Alembic; MinIO (CRS/NRB batch archives).

**Deployment:** Kubernetes under bvr-svc; separate namespace from registration-window M02 to enforce data-flow governance. Active year-round.

**Team:** 1 lead, 2 back-end engineers, 1 data engineer, 1 QA. Total: 5.

**Build effort:** 18 person-months [LOW 14 / MID 18 / HIGH 22].

**Dependencies upstream:** CRS (Civil Registration Service), NRB (National Registration Bureau), M02 (voter register master). **Downstream:** M36 (deregistration events), M64 (public inspection), M53 (ID-card trigger).

**Three hardest problems:**
1. *CRS/NRB data quality and lateness.* Approach: each batch validated against known schema; anomaly check on unexpected change rates (>0.5% of register in a single batch triggers hold for human review); manual exception queue with 5-day SLA.
2. *Dead-voter removal without disenfranchising the living with similar names.* Approach: ABIS-quality matching (M03 API) used for death-record matching, not just name/DOB; dual-approval for any removal; affected voter notified by SMS via last-known number.
3. *Ward-transfer workflow with election-calendar blackout periods.* Approach: M52 election-calendar API gates transfers; blackout enforcement is a policy engine (OPA) rule, not ad-hoc code; override requires Commissioner dual-approval.

**Delivery milestone:** CRS/NRB ingestion pipeline T-540; ward-transfer workflow T-420; accuracy dashboard T-300; full year-round operation T-240.

**Test strategy:** CRS/NRB mock-batch integration test; death-removal accuracy test against synthetic ground truth; election-calendar blackout enforcement test; 6-month synthetic continuous-run simulation.

**Compliance anchor:** Elections Act §6A (continuous register maintenance); DPA 2019 §26 (purpose limitation for CRS/NRB data); Constitution Art 83 (right to be on register).

---

### M36 — Voter Deregistration Appeal

**Purpose:** Statutory appeal workflow for voters administratively removed from the register: case-file intake, hearing scheduling, adjudicator decision workflow, statutory deadline enforcement, automatic Elections Court referral on breach.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (bvr-svc schema); React + Next.js 14 (petitioner portal); MinIO (evidence documents); Kafka (case events); Africa's Talking SMS notifications.

**Deployment:** Kubernetes under bvr-svc; episodic load (registration windows and CVR maintenance batches).

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 16].

**Dependencies upstream:** M35 (deregistration events), M02 (voter record). **Downstream:** M40 (Elections Court referral on deadline breach), M64 (inspection-window objection merge).

**Three hardest problems:**
1. *Statutory 14-day hearing deadline enforcement.* Approach: SLA timer set at case creation; automated escalation to supervisor at 80% consumed; automatic court-referral event at T+14 if no decision; all timers logged in tamper-evident audit trail.
2. *Evidence integrity for court proceedings.* Approach: all petitioner-uploaded evidence hashed at upload and stored immutably in MinIO; adjudicator decision includes hash commitment to evidence set; hash publishable in transparency log.
3. *Adjudicator access to summary data without raw CRS/NRB biographic data.* Approach: adjudicator view shows only the case-officer recommendation, M35-derived summary, and voter's own data; raw CRS/NRB data never exposed outside M35 service.

**Delivery milestone:** Appeal intake + case file T-420; adjudicator workflow T-360; court-referral automation T-300.

**Test strategy:** SLA timer accuracy test; court-referral trigger test; evidence hash verification; role-isolation test (adjudicator cannot see raw NRB data).

**Compliance anchor:** Elections Act §6A(4) (automatic court referral); DPA 2019 §26; Administrative Justice Act 2015.

---

### M37 — Voter Education CMS

**Purpose:** Multi-format, multi-language voter education content management, campaign scheduling, and distribution to all IEMS channels (M11, M13, M29, M68, SMS, social media export).

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; Next.js 14 (CMS admin UI); MinIO (media assets); Kafka (publication events); Africa's Talking (SMS push); CDN for content delivery.

**Deployment:** Kubernetes under training-svc; low throughput (content management, not citizen-facing serving — citizen-facing content is delivered via M11/M13 CDN).

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** IEBC Communications team (content owners). **Downstream:** M13, M11, M29, M68, M38 (content distribution).

**Three hardest problems:**
1. *Multi-language content workflow (EN, SW, + 9 regional languages).* Approach: translation-management integration (Lokalise or Phrase); professional translation SLA tracked in CMS; community translation pipeline for regional languages with editorial review gate.
2. *Content scheduling aligned with election calendar.* Approach: M52 election-calendar API drives campaign schedules; content goes live automatically at scheduled time; embargo enforced by OPA policy.
3. *Accessibility for multi-format output (audio, video, print, SMS).* Approach: CMS stores format-agnostic master; renderers for each output channel; audio track required for all video content; SMS versions capped at 160 chars with link.

**Delivery milestone:** CMS admin live T-480; channel integrations T-360; full multilingual content live T-270.

**Test strategy:** Translation completeness check; publication-timing accuracy test; SMS delivery integration test; CDN cache-invalidation test.

**Compliance anchor:** Elections Act §44A (voter education mandate); DPA 2019 §26 (no PII in public content).

---

### M38 — Poll Worker Training LMS

**Purpose:** Training, assessment, and certification LMS for approximately 290K poll workers; offline-capable tablet app; module-completion gating for election-day access credentials.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; React + Next.js 14 (LMS web); Flutter (offline tablet app); MinIO (video assets); Kafka (completion events to M33); Africa's Talking SMS (completion notification).

**Deployment:** Kubernetes under training-svc. Tablet APK distributed via MDM at training venues. Peak load during training windows (T-180 to T-90).

**Team:** 1 lead, 2 back-end, 1 mobile (Flutter), 1 front-end, 1 QA. Total: 6.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Dependencies upstream:** M37 (training content), M33 (worker registry). **Downstream:** M33 (certification gating for dispatch), M16 (ECC: certified-worker counts dashboard).

**Three hardest problems:**
1. *Offline video content delivery to rural training venues.* Approach: tablet app caches course content at venue preparation time; content signed and versioned; delta-update on next connectivity; streaming is never required for core training flow.
2. *Certification fraud prevention.* Approach: assessment randomises questions from a bank; time-limited attempt; biometric ID check at assessment start (fingerprint via tablet peripheral or camera facial match); completion event signed by device key.
3. *290K workers completing assessments in the T-180 to T-90 window.* Approach: multi-server HPA; venue-distributed tablet sessions reduce central server load; async completion-event pipeline decoupled from assessment API.

**Delivery milestone:** LMS web live T-420; tablet offline app T-360; 290K-session load test T-180.

**Test strategy:** 290K concurrent session load test; offline sync test (7-day offline); certification-fraud simulation; M33 integration test (completion event triggers dispatch eligibility).

**Compliance anchor:** Elections Act (poll worker qualifications); DPA 2019 §26 (worker PII); PPADA 2015 (training records).

---

### M39 — Ballot Printing Chain of Custody

**Purpose:** Printer-to-warehouse custody tracking: print-run records from Al Ghurair or equivalent printer MES, RFID chip manifest ingestion, quality-control scan records, transfer manifests, and warehouse arrival confirmation. Distinct from M63 (warehouse-to-polling-station).

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (bvr-svc-adjacent, separate inventory-svc schema); Kafka; MinIO (QC scan images); RFID ISO 15693 reader integration; printer MES webhook adapter.

**Deployment:** Kubernetes under inventory-svc. Air-gapped printer-side component for QC scan uploads via one-way data diode.

**Team:** 1 lead, 2 back-end, 1 integration engineer, 1 QA. Total: 5.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Dependencies upstream:** Printer MES (Al Ghurair webhook); M20 (warehouse receiving). **Downstream:** M63 (ballot UID handoff at warehouse), M20 (inventory record creation).

**Three hardest problems:**
1. *Printer MES API is proprietary and may change.* Approach: adapter pattern with versioned client; contract test against printer MES sandbox; fallback to manual manifest CSV upload.
2. *RFID chip defect rate in print run (estimated 0.1–0.3%).* Approach: QC scan at printer captures defect rate per roll; defective UIDs quarantined and never assigned to stations; defect report to M20 for replacement ordering.
3. *One-way data flow from air-gapped printer environment.* Approach: data diode hardware (e.g., Owl Cyber Defense) for manifest files; cryptographic signature on every manifest; receiving system verifies signature before import.

**Delivery milestone:** Printer MES adapter T-480; RFID manifest ingestion T-360; QC scan pipeline T-270; full integration with M63 T-180.

**Test strategy:** Printer MES contract test; RFID chip-defect simulation; data-diode transfer integrity test; M63 handoff test.

**Compliance anchor:** Elections Act §54 (ballot security); PPADA 2015 (printing contract governance); ISO 9001 (QC records).

---

### M40 — Election Court IT Interface

**Purpose:** Evidence-package system for election petitions: signed Form 34A/B/C retrieval, audit-log extraction, chain-of-custody export, M20/M63 custody records, and secure package delivery to the Supreme Court IT Registrar.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (read-only across rts-svc + bvr-svc + inventory-svc schemas); MinIO (form images); Kafka consumer (audit log replay); PDF/A evidence bundle generator (WeasyPrint); HSM-signed delivery tokens.

**Deployment:** Kubernetes under governance-svc; episodic (petition periods). Strict read-only role; no write access to any production schema.

**Team:** 1 lead, 2 back-end, 1 QA. Total: 4.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 16].

**Dependencies upstream:** M10, M09, M20, M63, P05 (audit log). **Downstream:** Supreme Court IT Registrar (HTTP delivery + secure portal link).

**Three hardest problems:**
1. *24-hour SLA for complete evidence pack after petition filing.* Approach: pre-computed evidence indexes updated continuously from M10/M20 events; bundle assembly parallelised by constituency; target 1-hour assembly, 24-hour SLA as backstop.
2. *Evidence package integrity for adversarial court scrutiny.* Approach: every included document carries original production signature; bundle manifest hashed and counter-signed by IEBC Commissioner before delivery; hash published to transparency log.
3. *Selective disclosure (petitioner sees only their constituency or station).* Approach: scope-limited API keys issued per petition by IEBC legal team; key encodes constituency and case ID; server enforces scope on every query.

**Delivery milestone:** Core evidence bundle generation T-360; Supreme Court IT Registrar integration T-270; 24-hour SLA test T-180.

**Test strategy:** 24-hour bundle assembly test with realistic data volume; signature chain verification; scope-isolation test; penetration test of the evidence-delivery endpoint.

**Compliance anchor:** Elections Act §75 (petition rights); Constitution Art 87 (election dispute resolution); Evidence Act 2011.

---

### M41 — Post-Election Review Platform

**Purpose:** Lessons-learned and recommendation tracking: finding registry, responsible-party assignment, closure verification, cross-cycle trend analysis, and input to next-cycle design review.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; React + Next.js 14; MinIO (review documents).

**Deployment:** Kubernetes under training-svc; very low throughput; active primarily T+30 to T+180.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M16, M21, M59 (findings feeds). **Downstream:** Next election-cycle PRD inputs; M69 archive.

**Three hardest problems:**
1. *Tracing findings across election cycles.* Approach: finding IDs carry cycle tag; cross-cycle trend view groups by root-cause category; each finding links to the module it implicates.
2. *Accountability without blame culture.* Approach: findings are assigned to process owners, not individuals; individual-attribution data DPA-compliant, access restricted to Programme Director.
3. *Capturing field observations from 47 counties.* Approach: M16 ECC exports structured incident log post-election; M34 observer reports merged; county ICT officer submits structured post-election debrief via portal.

**Delivery milestone:** Portal live T+30; full integration with M16/M21 export T+45; cross-cycle analytics T+180.

**Test strategy:** Finding lifecycle unit tests; cross-cycle trend query accuracy; role-access isolation test.

**Compliance anchor:** PPADA 2015 (contract performance review); ISO 27001 Annex A.16 (incident management improvement).

---

### M42 — Media Accreditation Portal

**Purpose:** Journalist and media-house accreditation: application intake, credential review, badge issuance, access-zone management, and renewal. Every accreditation action audit-logged.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17; React + Next.js 14; MinIO (credential documents); Africa's Talking SMS (notification).

**Deployment:** Kubernetes under training-svc; episodic load (pre-election accreditation window).

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** IEBC Communications team (approval authority). **Downstream:** M16 (access-zone enforcement), M57 (international press coordination).

**Three hardest problems:**
1. *Verifying media-house credentials (registration with Media Council of Kenya).* Approach: MCK registration number validated against MCK API; fallback to manual review with MCK certificate upload.
2. *Zone-based access control on election day.* Approach: QR-coded credential badge; M16 ECC scans badge at NTC/CTC entry; revocation propagates in <60 seconds.
3. *Audit trail for all accreditation decisions.* Approach: every action (approve, reject, revoke) logged with actor, timestamp, reason code, and IP; immutable append-only log.

**Delivery milestone:** Portal live T-300; badge issuance T-180; zone-access integration T-90.

**Test strategy:** MCK API integration test; badge revocation propagation test; audit-log completeness test.

**Compliance anchor:** Elections Act §86 (media access); Kenya Information and Communications Act 2013; DPA 2019 §26.

---

### M43 — Observer Accreditation Portal

**Purpose:** Civil-society and international observer accreditation: organisation vetting, individual credential, station-assignment, credential issuance, and reporting-access grant.

**Stack:** Identical pattern to M42 (Python 3.13 async, FastAPI, PostgreSQL 17, Next.js 14, MinIO, SMS); separate database schema.

**Deployment:** Kubernetes under training-svc, separate deployment from M42.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** IEBC Legal team; M57 (international mission coordination). **Downstream:** M34 (MDM provisioning trigger), M15 (observer read-access grant).

**Three hardest problems:**
1. *Vetting international observer organisations (AU, EU, Carter Center, ELOG).* Approach: pre-vetted list of recognised international election-observer bodies; new body applications require IEBC Commissioner approval; bilateral accreditation agreements loaded at T-300.
2. *Station-assignment at scale (500+ observers across 46K stations).* Approach: assignment algorithm optimises coverage; manual override by IEBC; assignment published to observer via M34 app.
3. *Credential revocation for misconduct.* Approach: instant revocation event to M34 MDM; badge QR code blacklisted in M16.

**Delivery milestone:** Portal live T-300; international missions on-boarded T-180; station assignments published T-90.

**Test strategy:** Revocation propagation test; station-assignment coverage optimiser test; international-organisation vetting workflow test.

**Compliance anchor:** Elections Act §86; Vienna Convention on Diplomatic Relations (diplomatic observers); DPA 2019.

---

### M44 — Turnout Forecasting and Analytics

**Purpose:** Real-time and predictive turnout analytics: live BVI event aggregation, historical model calibrated on 2013-2022 elections, constituency heat-map, ECC feed, and public dashboard.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (read replica of BVI events); ClickHouse (time-series aggregation); Kafka consumer; Grafana (embedded public dashboard); scikit-learn + statsmodels (forecasting models); Redis (ECC WebSocket push).

**Deployment:** Kubernetes under ecc-svc; active from station-open through last publication.

**Team:** 1 data-science lead, 1 back-end, 1 data engineer, 1 QA. Total: 4.

**Build effort:** 14 person-months [LOW 11 / MID 14 / HIGH 18].

**Dependencies upstream:** M06 (BVI events), M02 (station-registered-voter counts). **Downstream:** M16 ECC (real-time feed), M19 (queue coordination), public dashboard.

**Three hardest problems:**
1. *Model accuracy when turnout patterns shift from historical baselines.* Approach: Bayesian update as election-day data arrives; credible-interval bands displayed rather than point estimates; model is advisory only — no automated decisions triggered by forecast.
2. *Latency from BVI event to dashboard update at scale.* Approach: ClickHouse materialised views refreshed every 60 seconds; WebSocket push to ECC on material change (>2% constituency change); public dashboard refreshes every 5 minutes.
3. *Constituency-level forecast for 290 constituencies simultaneously.* Approach: parallelised Kafka consumer per region; ClickHouse partitioned by constituency; pre-aggregated county summaries in Redis.

**Delivery milestone:** Historical model calibrated T-360; live BVI integration T-180; public dashboard T-90.

**Test strategy:** Model accuracy back-test against 2022 election data; 60K BVI event/minute throughput test; ECC WebSocket latency test.

**Compliance anchor:** DPA 2019 §26 (aggregate data only; no individual voter tracking); Elections Act §74.

---

### M45 — Party Registration and Management

**Purpose:** Registrar of Political Parties workflow: party constitution submission, membership threshold verification, office-bearer records, governance compliance, and deregistration process.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (crms-svc schema); React + Next.js 14; MinIO (constitution documents, membership lists); Kafka.

**Deployment:** Kubernetes under crms-svc. Episodic (party registration windows, annual compliance).

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Dependencies upstream:** KEBS (membership threshold data), M67 (symbol/colour uniqueness check). **Downstream:** M46 (party status for primaries), M12 (party status for nominations), M47 (fee status).

**Three hardest problems:**
1. *Membership threshold verification (Political Parties Act §14: 1,000 registered voters in at least 24 counties).* Approach: M02 voter register cross-check API verifies each submitted member ID; batch processing with rate limiting; exception queue for non-matching IDs.
2. *Office-bearer changes between elections.* Approach: change workflow requires current office-bearer authentication + incoming acceptance; IEBC Registrar countersigns; audit trail immutable.
3. *Deregistration process with natural-justice requirements.* Approach: show-cause notice workflow with 21-day response window; SLA enforcement; automatic gazette notice publication via M52 on deregistration.

**Delivery milestone:** Core party registration T-480; membership verification T-420; compliance dashboard T-360.

**Test strategy:** Membership threshold verification test against synthetic voter-register subset; deregistration workflow SLA test; M67 symbol-check integration test.

**Compliance anchor:** Political Parties Act 2011 §§7-14; Elections Act §22; DPA 2019 §26.

---

### M46 — Party Primaries Oversight

**Purpose:** IEBC oversight of party primary elections: nomination schedule filing, results upload from party, dispute intake, gender-rule compliance check, and outcomes feed to M12.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (crms-svc schema); React + Next.js 14; MinIO (results documents); Kafka.

**Deployment:** Kubernetes under crms-svc. Episodic (nomination seasons T-365 to T-270 typically).

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M45 (party status), M12 (nomination downstream). **Downstream:** M12 (approved primaries outcomes), M23 (dispute routing).

**Three hardest problems:**
1. *Party-uploaded results integrity (no cryptographic signing infrastructure at party level).* Approach: party uploads signed PDF + CSV; IEBC officer verifies PDF against CSV; discrepancy triggers manual review; no automated acceptance of party-provided tallies.
2. *Gender-rule compliance at primary level.* Approach: real-time quota counter per party per elective position; portal blocks submission of non-compliant list; IEBC can override with statutory justification and audit log.
3. *Dispute intake with statutory timelines.* Approach: dispute filed within 48 hours of primaries result; SLA timer enforced; decision required within 7 days; automatic M12 block on disputed seat until resolved.

**Delivery milestone:** Schedule filing + results upload T-480; dispute workflow T-420; gender-rule enforcement T-360.

**Test strategy:** Gender-rule edge-case tests; dispute SLA enforcement test; M12 integration (disputed-seat block) test.

**Compliance anchor:** Political Parties Act 2011 §25 (primaries); Elections Act §27 (gender rule).

---

### M47 — Nomination Fee Payment

**Purpose:** Candidate nomination fee collection via M-Pesa and bank transfer, receipt generation, fee-waiver workflow for gender-rule candidates, and refund tracking.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (crms-svc schema); React + Next.js 14; M-Pesa Daraja API; Pesapal or DPO Group for bank-card fallback; MinIO (receipts).

**Deployment:** Kubernetes under crms-svc. Episodic (nomination window).

**Team:** 1 lead, 1 back-end, 1 payment-integration engineer, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M12 (nomination eligibility), M45 (candidate's party status). **Downstream:** M12 (fee-paid status gate).

**Three hardest problems:**
1. *M-Pesa STK push timing out when candidate does not respond promptly.* Approach: 90-second STK window; auto-retry with user notification; bank-transfer fallback for amounts above M-Pesa daily limit.
2. *Fee-waiver workflow for gender-affirmative candidates.* Approach: waiver application with Elections Act §27 justification; IEBC Secretariat approval in portal; waiver signed by Commissioner; M12 notified of waiver status.
3. *Refund on disqualification before close of nominations.* Approach: automated refund trigger on M12 disqualification event; M-Pesa reversal or bank-transfer refund within 5 business days; refund audit log.

**Delivery milestone:** M-Pesa integration T-420; bank-transfer fallback T-360; fee-waiver workflow T-300.

**Test strategy:** M-Pesa STK mock integration test; refund trigger test; payment reconciliation test.

**Compliance anchor:** Elections Act §22 (nomination fees); DPA 2019 §26 (payment data minimisation).

---

### M48 — Recall Election Workflow

**Purpose:** End-to-end workflow for constituency member recall under Constitution Art 104: petition intake, signature verification against voter register, threshold calculation, IEBC gazette trigger, and ballot generation.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (governance-svc schema); React + Next.js 14; M02 (voter register cross-check); M52 (gazette trigger); M12 (ballot generation trigger).

**Deployment:** Kubernetes under governance-svc. Rare event (expected 0-5 per cycle); low throughput.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M02 (voter register), M52 (election calendar), M12 (ballot). **Downstream:** Full IEMS election pipeline (recall triggers a by-election-like flow via M50).

**Three hardest problems:**
1. *Signature verification at scale (30% of constituency registered voters required).* Approach: each signature batch cross-checked against M02 voter register API; duplicate-signature detection via M03 ABIS-quality name/DOB matching; anomaly flag on bulk-submitted signatures.
2. *Constitution Art 104 threshold calculation across multiple recall attempts in same constituency.* Approach: recall history table tracks all attempts per constituency per member per term; threshold re-calculated on each new petition; court ruling on previous petition is a blocking dependency.
3. *Gazette timeline enforcement (Constitution Art 104(3): 90 days from petition certification).* Approach: M52 event scheduling enforces 90-day outer bound; ECC alert at 60 days if not yet gazette-ready.

**Delivery milestone:** Petition intake + signature verification T-480; threshold calculation T-420; gazette integration T-360.

**Test strategy:** Signature verification batch test; threshold calculation edge-case test; 90-day timer enforcement test; M02 integration test (voter eligibility cross-check).

**Compliance anchor:** Constitution Art 104; Elections Act §45; DPA 2019 §26.

---

### M49 — Referendum Mechanics

**Purpose:** KIEMS mode-switch and back-end configuration for constitutional referenda: question registry, YES/NO tally logic, constitutional-threshold computation (50%+1 or two-thirds), and result declaration.

**Stack:** Mode-switch configuration layer over M08 (device) and M10 (backend); Python 3.13 async FastAPI referendum-configuration service; PostgreSQL 17 (governance-svc schema, separate from general election schema); React + Next.js 14 admin UI.

**Deployment:** Kubernetes under governance-svc. The configuration service is always-on; it activates the referendum mode flag 48 hours before referendum day.

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 16 person-months [LOW 13 / MID 16 / HIGH 20].

**Dependencies upstream:** M08 (device mode flag), M10 (results backend), M52 (referendum calendar). **Downstream:** M11 (results publication), M51 (declaration), M68 (voter explainer platform), M49 feeds Attorney General gazette notification.

**Three hardest problems:**
1. *Mode-switching 65,287 kits from general-election to referendum mode safely.* Approach: mode flag delivered as signed MDM configuration push; kit validates flag signature before switch; mode switch logged as HSM-signed audit event.
2. *Multiple questions on a single referendum ballot.* Approach: question registry supports 1-N questions; M08 device shows one question per screen with forced navigation; M10 tallies per question in separate columns; constitutional threshold computed per question.
3. *Contested threshold calculation (50%+1 vs two-thirds depending on question type).* Approach: threshold type is a parameter in the question registry, not hard-coded; Commissioner sets threshold at question registration with dual-approval; threshold derivation is part of the M14 reproducibility bundle.

**Delivery milestone:** Question registry + configuration service T-480; mode-switch integration T-360; threshold computation T-270; constitutional threshold verified T-180.

**Test strategy:** Mode-switch test across 100 simulated kits; multi-question tally accuracy test; threshold edge-case tests (50%+1, two-thirds, near-threshold scenarios); M14 reproducibility test for referendum tally.

**Compliance anchor:** Constitution Art 255-257 (referendum thresholds); Elections Act §49-55; DPA 2019 §26.

---

### M50 — By-Election Operations

**Purpose:** Reduced-scope election operations for a single constituency: partial kit deployment, constrained nomination window, and all cryptographic and audit guarantees of the general election in a single-constituency footprint.

**Stack:** Configuration layer over core IEMS services (M01-M12, M10, M16); Python 3.13 async FastAPI by-election-configuration service; PostgreSQL 17 (governance-svc schema).

**Deployment:** Kubernetes under governance-svc. Low throughput; activates episodically on gazette of a by-election.

**Team:** 1 lead, 1 back-end, 1 QA. Total: 3.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Dependencies upstream:** M12, M10, M16, M52 (calendar). **Downstream:** Full IEMS result pipeline scoped to one constituency.

**Three hardest problems:**
1. *Activating the full election pipeline for a single constituency without disrupting live system state from the previous general election.* Approach: by-election creates a separate election-cycle context in the database; all results tables are partitioned by cycle; previous-cycle data is read-only.
2. *Kit allocation for a single constituency from the main fleet.* Approach: M20 allocation workflow restricts by-election kits by constituency geofence; allocated kits are flagged; deallocation after by-election is an explicit M20 workflow step.
3. *Nomination window compressed to statutory minimum.* Approach: M52 enforces nomination window; M12 hard-blocks submission outside window; M47 fee payment window aligned to M12 window.

**Delivery milestone:** By-election configuration service T-480; full pipeline end-to-end test for single constituency T-360.

**Test strategy:** Full election pipeline test scoped to 1 constituency; kit-allocation isolation test; multi-by-election concurrency test (up to 3 simultaneous by-elections in different constituencies).

**Compliance anchor:** Elections Act §45 (by-election timing); Constitution Art 101.

---

### M51 — Result Declaration and Certification

**Purpose:** Formal result declaration workflow from constituency to national: Form 34-38 certification pipeline, IEBC Commissioner digital signature, gazette-ready export, and Supreme Court notification.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (rts-svc schema); React + Next.js 14 (Commissioner portal); HSM signing service; WeasyPrint (gazette-ready PDF/A); M52 (gazette trigger); M40 (court notification).

**Deployment:** Kubernetes under rts-svc. Active T-0 through T+7.

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 15].

**Dependencies upstream:** M10 (aggregated tallies), M09 (constituency Forms 34B). **Downstream:** M11 (publication trigger), M40 (court evidence), M52 (gazette notice).

**Three hardest problems:**
1. *Commissioner signature ceremony under time pressure.* Approach: pre-staged signature request at form-assembly time; Commissioner signs with HSM-backed FIDO2 key; dual-Commissioner requirement for presidential declaration; signature latency p99 ≤ 2 seconds.
2. *Provisional vs. final declaration states.* Approach: four declaration states (provisional, gazette-ready, gazzetted, final); each state transition requires Commissioner dual-approval; state machine enforced by OPA policy; no backward transition.
3. *gazette-ready PDF/A with embedded signatures for legal permanence.* Approach: PDF/A-3 with embedded XMP signature metadata; SHA-256 hash of PDF registered in transparency log at generation time.

**Delivery milestone:** Declaration state machine T-360; Commissioner portal T-270; HSM signing integration T-180; gazette export T-120.

**Test strategy:** Declaration state-machine coverage tests; Commissioner dual-approval enforcement test; PDF/A conformance test (ISO 19005-3); gazette export accuracy test.

**Compliance anchor:** Elections Act §83 (declaration of results); Constitution Art 138(10); Statutory Instruments Act.

---

### M52 — Election Calendar and Gazette Notice Management

**Purpose:** Authoritative election-calendar store with gazetted dates, notice generation, Government Printer integration, and cross-module date-authority API consumed by all 69 modules.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (boundary-svc schema); React + Next.js 14 admin UI; Government Printer HTTP API integration; MinIO (gazette PDFs).

**Deployment:** Kubernetes under boundary-svc. Always-on; very low throughput. High read-cache (Redis); all modules read election-calendar dates from this service.

**Team:** 1 lead, 1 back-end, 1 QA. Total: 3.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** IEBC Commissioners (calendar authority); Government Printer (gazette publication). **Downstream:** Every other module (read-only date queries).

**Three hardest problems:**
1. *Calendar authority: any date wrong here causes cascade errors across 69 modules.* Approach: dual-Commissioner approval for every calendar event; calendar events are immutable once gazetted; amendment requires a new gazette notice and version entry.
2. *Government Printer API reliability.* Approach: retry with exponential backoff; fallback to manual PDF upload with IEBC counter-sign; gazette notice is published when PDF is signed regardless of Printer API status.
3. *Calendar change propagation to all modules.* Approach: calendar change event published to Kafka topic `iebc.calendar.events`; all modules subscribe and invalidate local caches; M52 is the single source of truth — no module embeds hardcoded dates.

**Delivery milestone:** Core calendar service T-540; Government Printer integration T-420; full module integration T-300.

**Test strategy:** Dual-approval enforcement test; Government Printer mock integration test; Kafka propagation test (all 18 services receive calendar change within 10 seconds).

**Compliance anchor:** Statutory Instruments Act §5 (gazette publication); Elections Act §6 (election-date notice).

---

### M53 — Voter ID Card Production

**Purpose:** ID-card order management from M35 CVR or M02 registration events: NRB integration for biographic data, card-manufacturing partner API (NHIF card model or NIIMS card model), QC scan, distribution tracking, and replacement workflow.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (bvr-svc schema); React + Next.js 14 (production dashboard); MinIO (card design proofs, batch manifests); NRB API integration; card-partner API adapter.

**Deployment:** Kubernetes under bvr-svc. Episodic load (post-registration batch production runs).

**Team:** 1 lead, 1 back-end, 1 integration engineer, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M02, M35 (voter record triggers), M64 (inspection-window corrections trigger re-print). **Downstream:** Distribution tracking feeds M33 (logistics coordination).

**Three hardest problems:**
1. *NRB biographic data latency (NRB may be days behind registration).* Approach: card production queued; NRB check retried daily; production triggered only when NRB confirms; voter notified of delay by SMS.
2. *Card security feature verification (hologram, chip, UV ink).* Approach: QC scan protocol captures UV + visible-light images; ML classifier checks security features; defective cards quarantined; defect rate reported to card manufacturer.
3. *Card distribution tracking to constituency level.* Approach: card batches bar-coded by constituency; delivery confirmed by M33-style GPS check-in at constituency store; uncollected cards flagged after 90 days for return.

**Delivery milestone:** NRB integration T-480; card-partner API T-360; QC scan classifier T-270; distribution tracking T-180.

**Test strategy:** NRB API integration test; QC classifier accuracy test on synthetic card images; batch-manifest reconciliation test.

**Compliance anchor:** Elections Act §6A (ID card as voter credential); NRB Act; DPA 2019 §25 (biometric data on card).

---

### M54 — Election Budget and Expenditure Portal

**Purpose:** IEBC budget tracking and public expenditure transparency: budget-line management, expenditure upload with supporting documents, public dashboard, and OAG export.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (support-admin-svc schema); React + Next.js 14; MinIO (supporting documents); Africa's Talking SMS (approval notifications).

**Deployment:** Kubernetes under support-admin-svc. Always-on for Finance team; public dashboard on CDN.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** IEBC Finance directorate (budget authority). **Downstream:** M59 (OAG export), public dashboard.

**Three hardest problems:**
1. *Budget-line vs. actuals reconciliation across 47 county sub-budgets.* Approach: county-level expenditure uploaded by County ICT officer via M56; consolidated at national level; variance flag triggers Finance review.
2. *Public dashboard with appropriate redaction.* Approach: published figures are at programme-level granularity; sensitive procurement line items (HSM vendor, ABIS vendor pricing) redacted per PPADA §66; redaction level set by Commissioner order.
3. *Audit trail for expenditure authorisations.* Approach: every expenditure approval carries digital signature of authorising officer; approval event written to tamper-evident audit log; OAG can replay all approvals.

**Delivery milestone:** Budget management live T-480; expenditure upload T-360; public dashboard T-270.

**Test strategy:** Reconciliation accuracy test; redaction logic test; OAG export completeness test.

**Compliance anchor:** Public Finance Management Act 2012; PPADA 2015; Constitution Art 226 (accountability of public funds).

---

### M55 — HR and Payroll for Temporary Election Staff

**Purpose:** 290K temporary-staff lifecycle: engagement workflow, M-Pesa bulk disbursement, tax withholding, dispute resolution, NHIF/NSSF integration, and final payroll reconciliation report.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (support-admin-svc schema); React + Next.js 14 (HR admin portal); M-Pesa Daraja bulk disbursement; KRA iTax API (tax withholding); NHIF/NSSF APIs; Kafka (M33 attendance events trigger).

**Deployment:** Kubernetes under support-admin-svc. Peak at T-1 (290K disbursements in a single window).

**Team:** 1 lead, 2 back-end, 1 integration engineer, 1 QA, 1 SRE (bulk disbursement run). Total: 6.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 16].

**Dependencies upstream:** M33 (attendance events = payroll trigger), M38 (certification = eligibility gate). **Downstream:** KRA, NHIF, NSSF (statutory deductions); M59 (OAG payroll report).

**Three hardest problems:**
1. *290K M-Pesa disbursements in a single batch without SLA breach.* Approach: M-Pesa Daraja bulk API supports batches of up to 10,000; orchestrated in 30 parallel batches; idempotency keys prevent double payment; completion status polled; fallback to bank transfer for M-Pesa failures.
2. *Tax withholding accuracy for casual-worker tax bracket.* Approach: KRA iTax API confirms each worker's PIN and WHT rate at engagement; deductions computed at disbursement time; P9A forms generated for all workers.
3. *Dispute resolution for non-payment claims.* Approach: worker submits dispute via M33 app or USSD; HR admin reviews M33 attendance log and M-Pesa disbursement receipt; decision within 5 business days; escalation to Programme Director at 10 days.

**Delivery milestone:** Engagement workflow T-360; M-Pesa bulk disbursement T-270; 290K disbursement load test T-180.

**Test strategy:** 290K disbursement batch test against M-Pesa sandbox; tax-withholding accuracy test; dispute SLA enforcement test.

**Compliance anchor:** Labour Act (employment records, WHT); KRA WHT regulations; NHIF Act; NSSF Act.

---

### M56 — County and National Government Coordination Interface

**Purpose:** Secure data-sharing interface with county governments, Interior Ministry, and security agencies: pre-election logistical data exchange, polling-day situation feeds, and DPA-lawful-basis logging for every data item shared.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (support-admin-svc schema); React + Next.js 14 (coordination portal); Kafka; M62 (lawful-basis audit integration); mTLS-authenticated API endpoints for county portals.

**Deployment:** Kubernetes under support-admin-svc. Active from T-180 through T+30.

**Team:** 1 lead, 2 back-end, 1 QA. Total: 4.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M16 (operational feeds), M18 (boundary data). **Downstream:** County governments, Interior Ministry, security agencies (all consumers only — no write-back to IEMS).

**Three hardest problems:**
1. *Lawful-basis logging for every data item exchanged.* Approach: M62 integration; every outbound data item carries DPA 2019 §30(1)(d) (public task) purpose code, recipient authority, timestamp, and category; logged immutably.
2. *County government IT capability varies widely.* Approach: API-first but with CSV export fallback for counties without API integration; self-service portal for data download; no requirement on county IT system.
3. *Security agency access to sensitive operational data during election day.* Approach: separate RBAC roles for security agencies; data scoped to their operational need (station locations, not voter data); dual-Commissioner approval for any ad-hoc access beyond pre-approved scope.

**Delivery milestone:** Core coordination portal T-360; M62 lawful-basis integration T-270; county API endpoints T-180.

**Test strategy:** Lawful-basis log completeness test; role-scope isolation test (security agency cannot query voter data); county CSV export accuracy test.

**Compliance anchor:** DPA 2019 §30(1)(d); Elections Act §89 (cooperation with national security); National Government Coordination Act.

---

### M57 — International Observer Coordination Portal

**Purpose:** Multi-mission coordination for EU EOM, Carter Center, ELOG, IFES, and AU Observer Mission: mission briefing data, IEMS read-only feeds, diplomatic communication log, and mission-specific reporting.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (transparency-svc schema); React + Next.js 14; MinIO (mission briefing documents); Kafka consumer (read-only from production events).

**Deployment:** Kubernetes under transparency-svc. Active from T-180 through T+90.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M43 (mission accreditation), M15 (IAB data feeds), M10 (results feeds). **Downstream:** M34 (observer app data access grants).

**Three hardest problems:**
1. *Data access scoping per mission (EU EOM gets different data set than AU).* Approach: mission-scoped API keys; Rego policy per mission organisation type; scope defined at accreditation in M43.
2. *Diplomatic communication log integrity.* Approach: every IEBC communication to a mission logged with timestamp, author, and content hash; communication is immutable record once sent.
3. *Mission report aggregation across multiple observers.* Approach: M34 observer reports tagged with mission ID; aggregated mission report view in portal; export as signed PDF for mission's own report.

**Delivery milestone:** Portal live T-270; mission-scoped feeds T-180; aggregated report export T-90.

**Test strategy:** Mission-scope isolation test; diplomatic log integrity test; M34 integration test (mission tag propagation).

**Compliance anchor:** Vienna Convention on Diplomatic Relations; Elections Act §86; DPA 2019 §26.

---

### M58 — Parliamentary and Judiciary Reporting Platform

**Purpose:** Structured reporting interface for National Assembly's Departmental Committee on Justice and Legal Affairs, Senate, and Judiciary: elections-committee data packs, statutory returns, and audit reports.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (transparency-svc schema); React + Next.js 14; MinIO (report PDFs); WeasyPrint (statutory return PDF generation).

**Deployment:** Kubernetes under transparency-svc. Low throughput; episodic (committee hearings, statutory return windows).

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M10, M20, M54, M59. **Downstream:** National Assembly committee secretariat; Senate; Supreme Court Registrar.

**Three hardest problems:**
1. *Statutory return formats differ between National Assembly, Senate, and Judiciary.* Approach: format templates per recipient body; parameterised PDF generator; templates maintained in configuration, not code.
2. *Data accuracy for committee hearings (live data vs. statutory returns).* Approach: statutory returns are point-in-time signed snapshots; live data views are labelled "provisional" with freshness timestamp.
3. *Privilege and confidentiality for some committee materials.* Approach: document classification (public / restricted / confidential); confidential documents accessible only via password-protected link with audit log.

**Delivery milestone:** Core report generation T-360; recipient-specific templates T-270; statutory return automation T-180.

**Test strategy:** Format accuracy test per recipient body; confidentiality enforcement test; WeasyPrint PDF/A conformance test.

**Compliance anchor:** National Assembly Standing Orders; Senate Standing Orders; Elections Act §75.

---

### M59 — Auditor-General Interface

**Purpose:** OAG-scoped read-only data portal: financial audit extracts, kit-disposition reports, DPA-compliant access logging, and Auditor-General signed attestations of data completeness.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (read-only cross-service views); React + Next.js 14; MinIO; audit-log streaming from P05; HSM signing for attestation documents.

**Deployment:** Kubernetes under transparency-svc. Always-on for OAG access; low throughput.

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M20, M54, M55, M10, P05. **Downstream:** OAG audit workflow (no write-back).

**Three hardest problems:**
1. *OAG access scope is broad but must be DPA-compliant.* Approach: every OAG access event logged with data category, record count, extraction timestamp, OAG user, and scope reference; DPA 2019 §26 purpose code attached to every log entry.
2. *Data completeness attestation.* Approach: Auditor-General can request a signed attestation stating that the data extract covers a specified scope completely; attestation generated by HSM-signed certificate; IEBC Chief Accountant counter-signs.
3. *Large financial-data extracts for performance audit.* Approach: async extract job for large requests; OAG notified via email when ready; download link valid for 72 hours; extraction logged.

**Delivery milestone:** Core read-only portal T-360; access logging T-270; attestation workflow T-180.

**Test strategy:** Access-log completeness test; attestation signature verification; large-extract async job test.

**Compliance anchor:** Public Audit Act 2015; DPA 2019 §26; Constitution Art 229 (Auditor-General mandate).

---

### M60 — Medical and First-Response Coordination

**Purpose:** Polling-day medical and security incident coordination: structured incident reports from M33, ambulance-dispatch integration, hospital status feeds, and county health authority portal.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (ecc-svc schema); React + Vite (ECC embedded panel); Kafka consumer (M33 incident events); SMS gateway (Africa's Talking) for ambulance dispatch notifications; Redis (real-time status).

**Deployment:** Kubernetes under ecc-svc. Active on election day and 48-hour window around it.

**Team:** 1 lead, 1 back-end, 1 QA. Total: 3.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M33 (incident reports), M16 (ECC situational view). **Downstream:** County Health departments (outbound SMS/API notification); Kenya Red Cross API (if available).

**Three hardest problems:**
1. *Real-time triage of medical incident reports across 46K stations.* Approach: incident severity auto-classified by M28 AI stack based on keywords; P1 (life-threatening) incidents trigger immediate ECC alert and SMS to county ambulance dispatcher.
2. *Ambulance dispatch coordination without reliable APIs from county health systems.* Approach: SMS as primary dispatch channel; county health officer phone numbers configured per county; fallback to voice call runbook if SMS fails.
3. *Privacy of patient information in incident reports.* Approach: medical incident reports stored in a separate database table with access restricted to medical coordinator role; no patient PII in ECC public dashboard.

**Delivery milestone:** Incident intake + triage T-270; ambulance SMS dispatch T-180; county health portal T-120.

**Test strategy:** Severity classification accuracy test; SMS dispatch latency test (target <60 seconds from incident report to SMS); PII isolation test.

**Compliance anchor:** Health Act 2017; DPA 2019 §25 (health data as special category); Elections Act (duty of care to personnel).

---

### M61 — Ballot Secrecy Assurance and Testing Platform

**Purpose:** Automated ballot-secrecy assurance: anonymity-set analysis across BVI events, cross-module data-flow audits, and DPA 2019 purpose-limitation verification that no combination of published data allows voter-identification.

**Stack:** Python 3.13 async; FastAPI (audit API); PostgreSQL 17 (read-only cross-service); ClickHouse (anonymity-set analytics); React + Vite (internal dashboard); privacy-analysis toolchain (ARX data anonymisation library via JVM bridge).

**Deployment:** Kubernetes under support-admin-svc. Runs periodic scheduled analyses; also callable on-demand pre-election.

**Team:** 1 lead (privacy engineering), 1 data engineer, 1 QA. Total: 3.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M06 (BVI event log), M10 (result data), M20 (kit telemetry), M19 (queue data). **Downstream:** IEBC DPO (findings report), M15 IAB (secrecy assurance attestation).

**Three hardest problems:**
1. *Defining the anonymity set for a small polling station (<50 voters).* Approach: k-anonymity analysis with k=5 threshold; any published data combination that permits identification of <5 voters triggers data-generalisation recommendation; flagged to DPO.
2. *Cross-module data linkage attacks.* Approach: automated pipeline joins published data from M11, M13, M19, M44 against voter register attributes; any attribute combination with linkage probability >10% is flagged.
3. *Differential privacy for aggregate statistics.* Approach: Laplace mechanism noise injection on published aggregate statistics at station level where station size <100 voters; noise magnitude calibrated to epsilon=1.0 (standard practice per Apple/Google benchmarks).

**Delivery milestone:** Anonymity-set analysis T-360; cross-module linkage test T-270; differential-privacy injection T-180; pre-election assurance report T-60.

**Test strategy:** Known-linkage attack tests on synthetic data; anonymity-set threshold enforcement test; ARX library integration test.

**Compliance anchor:** DPA 2019 §25 (special-category data); GDPR Art 89 (research/public-task safeguards as benchmark); Elections Act §44A (ballot secrecy).

---

### M62 — Transmission Legal-Basis Audit

**Purpose:** Per-transmission lawful-basis audit log for every cross-system data movement: Elections Act citation, DPA 2019 purpose code, actor, timestamp, data category, and recipient — for all 69 modules.

**Stack:** Python 3.13 async; FastAPI (audit API); Kafka consumer (one-way, append-only); ClickHouse (audit warehouse); React + Vite (IEBC DPO dashboard); OPA policy engine (validates purpose codes at transmission time).

**Deployment:** Kubernetes under rts-svc (co-located with results transmission, the highest-volume data-sharing path). Separate namespace with read-only query interface.

**Team:** 1 lead, 1 back-end, 1 data engineer, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** Every module that transmits data (all 69 emit a `iebc.transmission.audit` CloudEvents envelope). **Downstream:** IEBC DPO (compliance dashboard), M59 (OAG access-log export), M15 (IAB audit feed).

**Three hardest problems:**
1. *Audit overhead must not add latency to transmission paths.* Approach: fire-and-forget Kafka publish for audit events; audit consumer is fully async; transmission is not gated on audit acknowledgment.
2. *Ensuring all 69 modules emit audit events for every data movement.* Approach: shared SDK library (pip package `iebc-audit-sdk`) imported by all modules; SDK wraps all outbound API calls; CI lint rule enforces SDK usage; audit-coverage report generated weekly.
3. *Purpose-code taxonomy must be agreed by IEBC DPO before go-live.* Approach: purpose-code registry built in the system; DPO reviews and signs off at T-480; any new purpose code requires DPO approval (OPA policy blocks transmission with unknown code).

**Delivery milestone:** Shared SDK T-480; ClickHouse audit store T-360; DPO dashboard T-270; 100% module coverage T-180.

**Test strategy:** Audit-event completeness test (all 69 modules emit events in integration environment); purpose-code validation test; ClickHouse query performance test (1B row audit table).

**Compliance anchor:** DPA 2019 §26 (purpose limitation); Constitution Art 31 (privacy); Elections Act §39(1C) (transmission obligations).

---

### M63 — Ballot Serial Chain of Custody

**Purpose:** Tracks every individual ballot paper from warehouse inclusion to station assignment, spoilt/replaced/reconciled disposition, and post-election reconciliation. Begins where M39 (Ballot Printing Chain) ends.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (inventory-svc schema); React + Next.js 14 (IEBC station-ops portal); RFID ISO 14443A / ISO 15693 reader integration; Kafka; MinIO (reconciliation PDFs).

**Deployment:** Kubernetes under inventory-svc. Heaviest load during kit-prep (T-30) and post-election reconciliation (T+1 to T+30).

**Team:** 1 lead, 2 back-end, 1 integration engineer, 1 QA. Total: 5.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M39 (ballot UID handoff), M20 (station-kit mapping). **Downstream:** M40 (evidence for petition), M21 (RLA ballot-level audit).

**Three hardest problems:**
1. *Tracking ~46M ballot papers (1,000 per station × 46K stations) to individual disposition.* Approach: serial-range assignment rather than individual RFID scan for most ballots; individual tracking for spoilt/replaced/returned ballots via RFID or barcode; range-level Merkle commitments published for range integrity.
2. *Reconciliation within 24 hours of station close.* Approach: PO submits reconciliation report via M08 at station close; M63 auto-validates against serial ranges assigned; mismatch triggers M40 alert.
3. *Post-election petition: proving a specific ballot paper was or was not in a given box.* Approach: serial-range commitments published in transparency log at kit-prep time; any individual ballot within a range can be included or excluded with a range-membership proof.

**Delivery milestone:** Serial-range assignment T-360; RFID integration T-270; reconciliation API T-180; petition-proof generation T-120.

**Test strategy:** 46M serial-range simulation; RFID reader integration test; reconciliation accuracy test; range-membership proof cryptographic test.

**Compliance anchor:** Elections Act §54 (ballot security); Elections (Technology) Regulations 2017 Reg 12; DPA 2019 §26.

---

### M64 — Voter List Public Inspection Portal

**Purpose:** Statutory T-90 to T-60 public-inspection regime: per-station voter roll download redacted per DPA 2019 §26, objection intake with structured form, objection resolution tracking, and automatic Elections Court referral for unresolved objections at T-60.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (bvr-svc read replica, redacted view); Next.js 14 (SSG + SSR); Redis (rate limiting, download tokens); MinIO (roll PDFs); Africa's Talking SMS (objection status notifications); Kafka (objection events to M36).

**Deployment:** Kubernetes under bvr-svc. High load during inspection window (T-90 to T-60); effectively zero outside that window.

**Team:** 1 lead, 2 back-end, 1 front-end, 1 QA. Total: 5.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M02 (voter register), M35 (CVR updates). **Downstream:** M36 (objection intake), M40 (unresolved objection court referral).

**Three hardest problems:**
1. *DPA-compliant roll publication: name and station visible; ID number and biometric reference suppressed.* Approach: database view with column-level encryption for suppressed fields; redaction enforced at the SQL view layer, not in application code; independent security test verifies suppressed fields are unreachable via any API endpoint.
2. *Objection deadline enforcement with statutory consequences.* Approach: objection window hard-closes at T-60 in OPA policy; unresolved objections at T-60 automatically create M40 court-referral events (Elections Act §6A(4) requirement); no manual override.
3. *High-volume roll downloads without exposing the voter register to bulk scraping.* Approach: per-station download tokens (rate-limited, one-time use, signed); token requires authenticated citizen IPRS lookup to obtain; token audit trail; bulk-download detection alert.

**Delivery milestone:** Roll publication API T-360; download-token system T-270; objection intake T-240; court-referral automation T-180.

**Test strategy:** Redaction enforcement test (suppressed fields absent from every API response); objection deadline enforcement test; bulk-scraping detection simulation.

**Compliance anchor:** Elections Act §6A; DPA 2019 §26; Constitution Art 31 (right to privacy).

---

### M65 — Diaspora Ballot Physical Logistics

**Purpose:** Physical return leg for diaspora ballots: tamper-evident container packing at embassies, customs pre-clearance coordination, diplomatic-pouch routing where applicable, hub-routing, chain of custody at each handoff, and physical arrival at Nairobi tally centre.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (inventory-svc schema); React + Next.js 14 (embassy portal + IEBC logistics portal); MinIO (manifest documents); Kafka; Africa's Talking SMS (status notifications to embassy electoral officer).

**Deployment:** Kubernetes under inventory-svc. Active T-7 through T+5 (packing to tally-centre arrival).

**Team:** 1 lead, 1 back-end, 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M24 (diaspora voting stations that generated the ballots), M20 (container UID from inventory). **Downstream:** M10 (tally-centre arrival triggers diaspora-result merge), M40 (chain-of-custody export for petition).

**Three hardest problems:**
1. *Diplomatic pouch routing is outside IEBC control.* Approach: diplomatic-pouch option available for high-risk countries; all other embassies use commercial courier with real-time tracking API integration (DHL/Aramex); ETA monitoring with M16 ECC alert if container is 24 hours late.
2. *Customs pre-clearance varies by country.* Approach: pre-clearance letter template generated by M65 with container manifest hash embedded; IEBC Legal team pre-negotiates clearance with KAA and customs for all 56 mission countries.
3. *Chain-of-custody handoff integrity at each transit hub.* Approach: each handoff requires biometric confirmation (fingerprint or facial photo) from the receiving officer via mobile app; signature hash recorded; breaks in custody chain trigger M16 ECC alert.

**Delivery milestone:** Embassy portal + packing manifest T-300; courier API integration T-240; customs pre-clearance workflow T-180; tally-centre arrival integration T-90.

**Test strategy:** Courier API mock integration test; handoff chain-of-custody verification test; diplomatic-pouch workflow simulation.

**Compliance anchor:** Constitution Art 82(1)(e) (diaspora voting); Vienna Convention (diplomatic pouch); Customs Act (pre-clearance).

---

### M66 — Election Financial Forensics

**Purpose:** Post-election published analytics for cost-per-vote conversion ratios by candidate, constituency, and electoral cycle using econometric methods; open API for researchers; ODPP and FRC referral for vote-buying patterns.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (read-only analytics-svc schema); ClickHouse (analytical store); pandas + statsmodels + linearmodels (regression discontinuity, IV, difference-in-differences); React + Vite (published analytics dashboard); Kafka consumer.

**Deployment:** Kubernetes under ai-platform-svc. Active T+30 onwards; annual report cycle.

**Team:** 1 econometrics lead, 1 data engineer, 1 back-end, 1 QA. Total: 4.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 16].

**Dependencies upstream:** M22 (campaign finance disclosures), M45 (party data), M47 (nomination fee data), M10 (results data). **Downstream:** ODPP referral API, FRC referral API, published open dataset.

**Three hardest problems:**
1. *Causal identification of vote-buying from observational data.* Approach: regression discontinuity where candidates are near spending limits; instrumental variables (distance to party HQ as IV for local spending); DID across election cycles. All models published with methodology and code in M14 reproducibility bundle.
2. *Endogeneity of campaign spending (winning candidates spend more, not more-spending candidates win).* Approach: IV approach with pre-registration limit as instrument; results reported with standard errors and 95% confidence intervals; no causal claims beyond what IV identifies.
3. *Legal exposure if referral naming is wrong.* Approach: referral threshold requires two independent econometric methods agreeing at p<0.01; referral is advisory to ODPP (not a finding of guilt); referral includes full methodology so ODPP can independently assess.

**Delivery milestone:** ClickHouse analytics store T-300; econometric model suite T-270; open dashboard T+30; annual report T+90.

**Test strategy:** Model accuracy back-test on 2017 and 2022 election data; referral threshold simulation; open-dataset integrity test.

**Compliance anchor:** Election Campaign Financing Act §24; ODPP mandate; DPA 2019 §26 (aggregate data only, no voter PII in published analytics).

---

### M67 — Party Symbol and Colour Registry

**Purpose:** Trademark-style registry for political party visual identity: symbol (graphical mark), colour (Pantone + RGB + CMYK), and logotype. Enforces Tanimoto similarity check for shapes, perceptual hash deduplication for images, and colour exclusivity per election cycle. Provides ballot-design export API for M12.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (crms-svc schema); React + Next.js 14 (party portal + IEBC back-office); MinIO (symbol images); OpenCV + imagehash (perceptual hash); shapely (Tanimoto coefficient for vectorised symbols); Kafka.

**Deployment:** Kubernetes under crms-svc. Low throughput (party registration season); very low on election day.

**Team:** 1 lead, 1 back-end (with computer vision skills), 1 front-end, 1 QA. Total: 4.

**Build effort:** 8 person-months [LOW 6 / MID 8 / HIGH 10].

**Dependencies upstream:** M45 (party registration triggers symbol submission). **Downstream:** M12 (ballot-design export), M45 (symbol approval status).

**Three hardest problems:**
1. *Similarity threshold calibration: too strict blocks legitimate symbols; too lenient allows confusion.* Approach: Tanimoto coefficient threshold 0.85 for shape similarity; perceptual hash Hamming distance ≤ 10 for image similarity; both thresholds configurable by IEBC Registrar with audit log; disputed similarity resolved by panel review.
2. *Colour exclusivity enforcement across ~90 registered parties.* Approach: colour space mapped to 256 Pantone equivalents; each election cycle each Pantone can be exclusive to one party; first-come-first-served within cycle; contested colours resolved by application date.
3. *Rebranding workflow: party changes symbol mid-cycle.* Approach: rebranding requires full similarity check against all existing symbols; mandatory differentiation test report; Commissioner approval; old symbol retained in registry as "retired" for historical reference.

**Delivery milestone:** Symbol registry + similarity checks T-480; colour-exclusivity enforcement T-420; ballot-design export API T-360.

**Test strategy:** Tanimoto coefficient accuracy test on known-similar symbol pairs; colour-exclusivity enforcement test; M12 export API integration test.

**Compliance anchor:** Political Parties Act 2011 §§7-9; Trade Marks Act 2002 (by analogy for similarity test methodology).

---

### M68 — Referendum Explainer Platform

**Purpose:** Voter-facing multi-language, multi-format public explainers for each referendum question: constitutional context, what Yes/No means, fact-check partner review status, and distribution across all channels. WCAG 2.2 AAA target (elevated from AA).

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (governance-svc schema); Next.js 14 (SSG/SSR, CDN-first); MinIO (media assets); M37 CMS integration (content source); Africa's Talking (SMS distribution); community translation pipeline (Crowdin or Transifex integration).

**Deployment:** Kubernetes under portal-svc. CDN-first; origin DDoS cannot prevent referendum explainer access.

**Team:** 1 lead, 1 back-end, 2 front-end, 1 QA. Total: 5.

**Build effort:** 10 person-months [LOW 8 / MID 10 / HIGH 13].

**Dependencies upstream:** M49 (question registry), M37 (content management). **Downstream:** M13, M11, M29 (distribution channels).

**Three hardest problems:**
1. *Legal neutrality of explainer content (IEBC cannot be seen to advocate Yes or No).* Approach: every explainer reviewed by IEBC Legal + independent constitutional law scholar; fact-check partner (e.g., PesaCheck) signs off on factual accuracy; fact-check status displayed prominently.
2. *Community translation for regional languages under time pressure.* Approach: Crowdin community translation platform with editorial review gate; professional translators for EN and SW; community volunteers for 9 regional languages with mandatory editorial review before publish; translated content not published until reviewed.
3. *WCAG 2.2 AAA for referendum content (elevated standard).* Approach: audio read-aloud for all text content; video with BSL and KSL subtitles; simplified language level (Flesch-Kincaid reading ease ≥ 60 for EN content); contrast ratio ≥ 7:1.

**Delivery milestone:** Platform live with question registry T-360 (if referendum called); full multilingual content T-270; AAA accessibility audit T-180.

**Test strategy:** Legal-neutrality review process test; WCAG 2.2 AAA automated (axe-core Level AAA) + manual; community translation completeness test; CDN static-snapshot test.

**Compliance anchor:** Constitution Art 255 (referendum process); DPA 2019 §26; WCAG 2.2 (AAA elevated for constitutional significance).

---

### M69 — Election Data Long-Term Archive

**Purpose:** 15-year retention archive: transition of election data from operational databases into long-term preservation formats, KNADS integration, format-migration pipelines, signed preservation attestations, and public-access API for researchers and historians.

**Stack:** Python 3.13 async; FastAPI; PostgreSQL 17 (support-admin-svc schema for archive metadata); MinIO (operational data export); BagIt (Library of Congress archival packaging standard); OAIS (ISO 14721 reference model for archive management); KNADS API integration; OpenSearch (researcher query interface).

**Deployment:** Kubernetes under support-admin-svc. Active T+180 onwards; low throughput but high storage volume. Data migrated from operational stores to object storage in BagIt packages.

**Team:** 1 lead (digital preservation specialist), 1 back-end, 1 data engineer, 1 QA. Total: 4.

**Build effort:** 12 person-months [LOW 9 / MID 12 / HIGH 16].

**Dependencies upstream:** M10, M20, M14, M59 (data sources for archival). **Downstream:** KNADS (physical archive deposit), public researcher API.

**Three hardest problems:**
1. *Format migration as storage formats age over 15 years.* Approach: BagIt packages include original format + immediate migration to BagIt-validated open formats (PDF/A-3, CSV, JSON-LD, TIFF for images); migration pipeline re-executable from source; each migration produces a new signed package version.
2. *Cryptographic signature validity over 15 years (algorithm sunset risk).* Approach: post-quantum hybrid signatures (ML-DSA + ECDSA P-384) for archive attestations; re-attestation planned at 5-year intervals using current best-practice algorithms; original signatures retained alongside re-attestations.
3. *Public researcher access with appropriate DPA redaction.* Approach: researcher-facing API returns aggregate data and anonymised records; voter PII fields redacted at query time using same redaction logic as M64; access gated on academic institution verification; usage logged for DPA compliance.

**Delivery milestone:** Archive ingestion pipeline T+180; KNADS integration T+270; public researcher API T+360; first format-migration test T+5 years.

**Test strategy:** BagIt package integrity verification; OAIS compliance check; KNADS API integration test; DPA redaction test on researcher-API queries; format-migration round-trip test.

**Compliance anchor:** Public Archives and Documentation Service Act; Kenya National Archives Act; DPA 2019 §32 (retention and archiving); Elections Act (permanent archive of results).

---

## Section 4 — Cross-Cutting Foundations

### 4.1 Identity

Every principal — human, service, or device — has a unique attributable identity.

- **Human identity:** Keycloak 23+ as OIDC/OAuth2 identity provider. FIDO2 hardware keys mandatory for all privileged accounts (Commissioner, ICT Admin, RO). SMS MFA prohibited on admin accounts. Short-lived access tokens (≤ 15 min interactive, ≤ 5 min service-to-service). Roles: Commissioner, Secretariat, ICT Admin, RO, PO, Enrolment Clerk, Party Agent, Auditor, Observer, ODPC Officer, Datacraft SRE.
- **Service identity:** SPIFFE/SPIRE. Every pod has a SPIFFE ID and a short-lived SVID (X.509 cert, 1-hour TTL, auto-rotated). Linkerd 2 service mesh validates SVID on every inbound request. No shared service credentials.
- **Device identity:** Per-kit X.509 certificate issued at kit-prep time from the internal PKI intermediate CA for devices. Certificate bound to kit TPM/secure element. Revocation propagates in ≤ 60 seconds via OCSP must-staple + CRL.

### 4.2 Cryptography

- **HSM:** Thales Luna 7 (or Utimaco CryptoServer) FIPS 140-3 Level 3+. Root CA keys generated and stored in HSM, never exported. HSM cluster: 3× redundancy, capacity 3× peak signing rate.
- **Long-lived signatures (results, certificates):** ML-DSA (CRYSTALS-Dilithium, NIST FIPS 204) + ECDSA P-384 hybrid, providing post-quantum resistance for long-lived form signatures that must remain verifiable in 10+ years.
- **Real-time signatures (kit events, API tokens):** ECDSA P-384 (fast, audited).
- **Symmetric encryption at rest:** AES-256-GCM with envelope encryption; data-encryption keys wrapped by HSM-stored key-encryption keys. Biometric templates use field-level encryption with separate template keys.
- **Transport:** TLS 1.3 everywhere; TLS 1.2 disabled; mTLS for all service-to-service; QUIC for kit-to-gateway on congested networks.
- **Post-quantum hybrid for key exchange:** X25519 + Kyber-768 (ML-KEM, NIST FIPS 203) hybrid KEM for new session establishment on sensitive channels.

### 4.3 Observability

- **Tracing:** OpenTelemetry SDK in every service (Python: opentelemetry-sdk, Kotlin: opentelemetry-android). Trace context propagated via W3C TraceContext headers. Traces exported to Tempo.
- **Metrics:** Prometheus scrape from every pod via opentelemetry-collector. Custom metrics per module (kit-auth latency, form-ingestion rate, ABIS queue depth, anomaly alert rate). SLO error-budget dashboards in Grafana.
- **Logs:** Structured JSON logs from every service; Loki aggregation; Grafana log explorer. Correlation IDs from kit event through CDN edge response.
- **Dashboards:** Grafana with module-specific dashboards; ECC war-room view (M16) aggregates top-level SLO status for all 28 modules.
- **Retention:** Metrics 15 years immutable (regulatory requirement for election data). Log archive: 90-day online + 15-year cold. Traces: 30-day online.
- **Alerting:** PagerDuty (or equivalent) for P0/P1; Grafana alerting for P2/P3; multi-channel (SMS, WhatsApp, email, dashboard) per severity.

### 4.4 Audit Logging

- Append-only, hash-chained, per-service audit log. Every audit entry: `(event_id UUID7, who, what, which_entity, from_where, when_utc, previous_hash, signature)`.
- Per-module Merkle trees built hourly; hourly roots aggregated into daily national Merkle tree.
- Daily root published to the Kenya Gazette URL + GitHub + IPFS.
- Independent escrow mirror: real-time replication of the Merkle tree to the Auditor-General's designated custodian (university or academic institution; specified at contract award).
- Break-glass: any use of break-glass admin credentials generates a public transparency-log entry visible to IAB and observers within 60 seconds.

### 4.5 Secrets Management

- HashiCorp Vault (BSL licence) sealed with Thales Luna HSM auto-unseal. No plain-text secrets in git, CI/CD pipelines, container images, or environment variables in configuration files.
- All secrets injected at pod startup via Vault Agent sidecar or CSI Secrets Store driver.
- Secret rotation: service credentials rotated every 30 days; certificate renewal automated (Vault PKI engine); emergency rotation procedure takes ≤ 15 minutes.
- SBOM published quarterly; dependency freshness reviewed monthly; no secrets in SBOM (SBOM is public).

### 4.6 Configuration and GitOps

- All Kubernetes manifests and Helm values in a dedicated private git repository (IEBC-owned; Datacraft has write access during contract).
- Every commit signed with Datacraft's GPG key; merge requires dual approval (Datacraft lead + IEBC ICT Admin).
- ArgoCD (open source) watches the config repo; applies changes to cluster; drift detection alerts if cluster state diverges from git.
- Software freeze: ArgoCD auto-sync disabled 30 days before any gazetted election event; re-enabled only on dual-commissioner approval for critical security patches.
- Terraform (IaC) for any infrastructure outside Kubernetes; Ansible for node-level hardening; all IaC in the same signed config repo.

### 4.7 Service Mesh

- Linkerd 2 (CNCF graduated project, open source). mTLS on every service-to-service call enforced by Linkerd sidecar proxy. No plaintext service communication inside the cluster is possible.
- Linkerd AuthorizationPolicy: explicit allow-list per service pair; default deny. ABIS management plane is explicitly isolated from RTS ingestion plane at policy level.
- Service-to-service observability: Linkerd Golden Metrics (latency, throughput, success rate) per service pair; exported to Prometheus.

### 4.8 Data Protection

- Kenya-jurisdiction hosting: production voter data, biometric templates, results, and audit logs never leave Kenya. IEBC-owned data centres; Datacraft has no offshore backup copy.
- Offshore remote support: permitted only via IEBC-controlled bastion (CyberArk PAM or Teleport) with session recording, dual-person approval, no production write access, no bulk data access.
- ODPC registration: Datacraft registers as data processor before go-live; IEBC registers as data controller.
- DPIA conducted before each new data processing activity that involves biometric or sensitive personal data.

### 4.9 API Standards

- REST APIs: OpenAPI 3.1 spec-first. Spec committed to git before code. Generated clients from spec for all back-end-to-back-end calls. JSON Schema for all request/response bodies.
- Async APIs: CloudEvents 1.0 envelope; Kafka topic schemas in Confluent Schema Registry (Avro). Event schema versioning with backward/forward compatibility enforced by schema registry.
- gRPC: protobuf 3 for high-throughput kit ingestion paths; `.proto` files in config repo; breaking changes require semver major bump.
- Error model: RFC 7807 Problem Details for every 4xx/5xx HTTP response. `type` URI, `title`, `status`, `detail`, `instance`, correlation `trace_id`.

---

## Section 5 — Technology Stack Decisions

| Layer | Choice | Rationale | Alternative considered |
|---|---|---|---|
| Language (back-end) | Python 3.13 async | Datacraft CLAUDE.md standard; deep hiring pipeline; ecosystem maturity (FastAPI, SQLAlchemy, Pydantic v2) | Go — rejected: reduces internal hiring depth; TypeScript — rejected: weaker async DB ecosystem |
| Web framework | FastAPI + Starlette | ASGI-native, OpenAPI generation, dependency injection, performance comparable to Go for I/O-bound workloads | Django REST Framework — rejected: sync-by-default, heavier; Flask — rejected: too minimal |
| Data validation | Pydantic v2 | CLAUDE.md standard; fastest Python validator; strict mode; JSON Schema export | Marshmallow — rejected: slower; attrs — rejected: less ergonomic for REST |
| ORM | SQLAlchemy 2.x async | Mature, async-native 2.x, migration tooling (Alembic), supports PostgreSQL extensions | Django ORM — rejected: tightly coupled to Django; Tortoise ORM — rejected: less mature |
| Database | PostgreSQL 17 | Row-level security, PostGIS, logical replication, Patroni HA, proven at scale | MySQL — rejected: weaker RLS + PostGIS; CockroachDB — rejected: adds distributed-system complexity |
| Cache / session | Redis 7 | Standard; Redis Sentinel for HA; RedisJSON for structured session data | Memcached — rejected: no persistence, no Lua scripting |
| Queue / streaming | Apache Kafka + Debezium | Durable, partitioned, replay-capable; Debezium for CDC outbox pattern | RabbitMQ — rejected: no built-in log replay; NATS — rejected: less mature for CDC |
| Object storage | MinIO (S3-compatible) | S3 API compatibility; open source; on-premise; no data-residency risk | AWS S3 — rejected: data residency; Ceph — rejected: operational complexity |
| Search | OpenSearch | Fork of Elasticsearch without BSL concerns; managed within Kubernetes | Elasticsearch — rejected: licence risk; Meilisearch — rejected: less mature for analytics |
| Frontend (voter-facing) | React + Next.js 14 | SSR + SSG in one framework; progressive enhancement; i18n; WCAG tooling | SvelteKit — rejected: smaller ecosystem; Remix — rejected: less mature for SSG |
| Frontend (ops) | React + Vite | Fast DX; SPA appropriate for ops consoles with complex state | Next.js — unnecessary SSR for ops; Angular — rejected: steeper onboarding for team |
| Mobile (citizen) | React Native (Expo) | Code sharing with web React; strong Kenya Android market; iOS for diaspora | Flutter — rejected: different language (Dart) from rest of stack; Kotlin Multiplatform — immature for shared UI |
| Kit-side (new kits) | Kotlin + Jetpack Compose (Android 13+) | Native Android; hardware SDK access; Compose for declarative UI | Java — rejected: Kotlin superset; React Native — rejected: insufficient biometric SDK depth |
| Kit-side (legacy Android) | Java + compatibility shim | Must run on older Android versions in the existing 14,100-unit fleet | No alternative — required by contract |
| On-device ML | ONNX Runtime + TFLite | ONNX for cross-platform central inference; TFLite for on-device (smaller binary) | PyTorch Mobile — rejected: larger binary; CoreML — iOS only |
| Container runtime | containerd + Kubernetes 1.31 | CNCF standard; CIS Benchmark available; IEBC likely familiar | Docker Swarm — rejected: less mature for production; Nomad — rejected: less ecosystem |
| CI/CD | GitHub Actions + ArgoCD | Actions for build/test/image; ArgoCD for GitOps deploy; both open source | Jenkins — rejected: operational burden; Flux — valid alternative to ArgoCD; chose ArgoCD for UI |
| IaC | Terraform + Ansible | Terraform for infra provisioning; Ansible for node-level hardening; proven combination | Pulumi — valid but smaller adoption in Kenya hiring market |
| Service mesh | Linkerd 2 | CNCF graduated; lightweight sidecar; strong mTLS; lower resource overhead than Istio | Istio — rejected: higher resource overhead; Consul Connect — rejected: adds Consul dependency |
| Observability | OpenTelemetry collector → Prometheus + Loki + Tempo | OTEL is vendor-neutral; Grafana stack is cohesive; all open source | Datadog — rejected: data residency risk; New Relic — rejected: same concern |
| Identity | Keycloak + SPIFFE/SPIRE | Keycloak for human OIDC; SPIRE for workload identity; both open source and CNCF | Okta — rejected: SaaS residency concern; Auth0 — rejected: same |
| Secrets | HashiCorp Vault + Thales Luna HSM | Vault is battle-tested; Luna is FIPS 140-3 Level 3+; BSL is acceptable for internal use | AWS Secrets Manager — residency concern; Azure Key Vault — residency concern |
| PKI | HashiCorp Vault PKI engine (+ SmallStep CA) | Vault PKI handles intermediate CA automation; SmallStep for ACME-compatible cert issuance | EJBCA — open source but heavier; Dogtag — complex |
| Feature flags | GrowthBook (open source) | Self-hosted, no data egress; supports percentage rollouts per election event | Unleash — valid alternative; LaunchDarkly — SaaS data-residency concern |
| Chaos engineering | Chaos Mesh | CNCF project; Kubernetes-native; pod kill, network partition, IO fault injection | Litmus — valid alternative; Netflix Chaos Monkey — less Kubernetes-native |
| E2E test | Playwright | Cross-browser; Python + TypeScript bindings; visual regression; network interception | Cypress — JS-only; Selenium — slower and flaky |
| Load test | k6 | JS-scriptable; Prometheus output; cloud execution option; CI-friendly | Locust — Python but slower; JMeter — XML config, difficult to version |
| Reproducibility build | Nix flakes | Hermetic; locked-hash inputs; byte-identical output across machines; the gold standard for reproducible builds | Bazel — complex ramp-up; Docker — not byte-identical across OS versions |

---

## Section 6 — Data Model Foundation

### 6.1 Entity taxonomy

| Domain | Core entities |
|---|---|
| Voter | Voter, BiometricTemplate, IDDocumentScan, VoterTransfer, PreRegistration |
| Electoral area | Country, County (47), Constituency (290), Ward (~1,450), PollingCentre, PollingStation (~55,393) |
| Kit | Kit, KitSubComponent, KitCustodyEvent, KitTelemetryEvent |
| Election | Election, ElectivePosition, ElectoralContest, Candidate, Party, Ballot |
| Results | ResultForm34A, ResultForm34B, ResultForm34C, ResultForm34D, TallyEvent, AdjudicationEvent |
| Audit | AuditEvent, MerkleBlock, MerkleTree, TransparencyLogEntry |
| Inventory | Asset, AssetCustodyChain, SealEvent, RFIDScanEvent |
| Governance | GrievanceCase, AnomalyAlert, CampaignDisclosure |

### 6.2 Event-sourced core

Every domain entity's state is derived by replaying its event stream. Entities are never updated in place for mutation; only for read-model snapshots. Event stream structure:

```
Event {
  event_id:       UUID7          -- time-sortable, globally unique
  stream_id:      UUID7          -- entity ID (e.g., voter_id, kit_id)
  stream_type:    str            -- "voter" | "kit" | "result_form" | …
  sequence:       int            -- monotonic per stream_id
  event_type:     str            -- "VoterEnrolled" | "KitDeployed" | …
  payload:        JSON           -- event-type-specific fields
  actor_id:       UUID7          -- who caused this event
  actor_role:     str            -- their role at the time
  kit_id:         UUID7 | null   -- if event originated on a kit
  station_code:   str | null     -- if election-day event
  occurred_at:    timestamp UTC  -- when the event occurred (device clock)
  received_at:    timestamp UTC  -- when back-end received it
  previous_hash:  bytes(32)      -- SHA-256 of previous event in stream
  signature:      bytes          -- ECDSA P-384 signature by originating principal
  hsm_countersig: bytes          -- HSM countersignature on ingestion
}
```

All events are append-only, stored in PostgreSQL `events` partitioned table (partitioned by `stream_type` + `occurred_at`). No event is ever deleted or updated.

### 6.3 Snapshot strategy

Read models (projections) are materialised as PostgreSQL tables or Redis hashes, derived by replaying events asynchronously via Kafka consumer. Snapshots are taken at configurable intervals (default every 1,000 events per stream) and stored in the `snapshots` table to avoid full-stream replay on startup. Snapshots are verified by re-deriving the same result from the event stream before being trusted.

### 6.4 ID architecture

- All IDs: UUID7 (time-sortable, k-sortable, PostgreSQL `uuid` column type). UUID7 from the `uuid6` PyPI package; Kotlin: `com.github.f4b6a3:uuid-creator`.
- Station code hierarchy: `{county_code:2}{constituency_code:3}{ward_code:4}{centre_code:3}{station_code:2}` = 14-character string; human-readable; used in all public-facing references.
- No auto-increment integers for domain IDs in the public API (information disclosure risk).

### 6.5 Referential integrity in an event-sourced system

- Foreign-key validation happens at command time (before event is written): the command handler verifies that referenced entities exist in their current read model.
- The event store itself has no FK constraints (events are immutable facts about the past; the referenced entity may have changed since).
- Cross-module references use entity IDs that are stable (UUID7 + entity type); referencing modules subscribe to entity lifecycle events to maintain their own read model of referenced entities.

---

## Section 7 — Delivery Timeline

All milestones anchored to T-0 = 2027-08-10 (election day).

| Phase | Window | Key deliverables |
|---|---|---|
| Mobilisation | T-720 to T-690 (2025-08 to 2025-09) | Contract signed; team hired; IEBC co-dev onboarding; PKI air-gapped ceremony; Kubernetes clusters provisioned (primary + mirror); CI/CD pipelines live; Vault unsealed; ArgoCD watching config repo; IPRS sandbox access obtained |
| Environment baseline | T-690 to T-660 (2025-09 to 2025-10) | All platform layers operational (Keycloak, SPIRE, Linkerd, Prometheus, Loki, Tempo, MinIO, Kafka, OpenSearch); canary deploy pipeline tested; sandbox environment live |
| Architecture baseline + all 69 module specs | T-660 to T-540 (2025-10 to 2026-02) | All 69 module specs approved; OpenAPI specs committed; 18 service schema migrations written; M01-M06 prototypes; data-migration ETL prototype; M52 election-calendar service live |
| Data migration (six-week deadline from CED) | T-720 + 6 weeks | Existing voter register migrated to new schema; reconciliation report; rollback plan signed off; gazetted cut-off data loaded |
| M01-M06 + M35 + M52 implementation | T-540 to T-420 (2026-02 to 2026-06) | BVR device + backend + ABIS + VPRS + BVI device + BVI backend + CVR pipeline + election calendar: feature-complete, unit tested, integration tested; M53 voter ID card integration begins |
| M07-M12 + M13-M20 + M45-M47 + M67 design and implementation | T-480 to T-360 (2026-04 to 2026-08) | All core modules in active development; M07-M12 feature-complete; BVV, RTS device, RTS-RO, RTS backend, Public Portal, CRMS; party registration + primaries + nomination fee + symbol registry feature-complete |
| M13-M28 + M29-M34 + M36-M44 + M48-M51 implementation | T-420 to T-270 (2026-06 to 2026-11) | All citizen-facing apps (M13, M29, M30, M31, M32, M33, M34); core transparency suite (M14, M15, M21); ECC (M16) + speech integrity (M17); queue (M19) + inventory (M20); voter education (M37) + LMS (M38); ballot chain (M39, M63); observer and media accreditation (M42, M43); referendum (M49, M68) + by-election (M50) + recall (M48); turnout analytics (M44); result declaration (M51) |
| M53-M69 implementation | T-360 to T-240 (2026-08 to 2026-12) | Remaining modules: voter ID card (M53), election budget (M54), HR/payroll (M55), county coordination (M56), international observers (M57), parliamentary reporting (M58), OAG interface (M59), medical coordination (M60), secrecy assurance (M61), transmission audit (M62), voter list inspection (M64), diaspora logistics (M65), financial forensics (M66), long-term archive (M69) |
| Integration and system test | T-300 to T-180 (2026-10 to 2027-02) | All 69 modules integrated across 18 services; end-to-end Playwright test suite passing; 60K-session load test passing; ABIS accuracy certification; M62 transmission-audit SDK coverage 100% |
| UAT with IEBC | T-180 to T-120 (2027-02 to 2027-04) | IEBC operators conduct full UAT per module across all 69 modules; defects fixed; IEBC sign-off on each module |
| Security assessment + code audit | T-180 to T-120 (2027-02 to 2027-04) | ISO 27001:2022 certification; independent penetration test (full 69-module scope); code audit; source-code escrow package prepared; M61 secrecy-assurance report issued to DPO |
| Full-scale dress rehearsal | T-90 to T-60 (2027-05-12 to 2027-06-11) | Mock election on sandbox with IEBC staff; all 18 services exercised; all 6 mobile apps deployed; simulated 55,393 stations; real connectivity test in 5 counties; M44 turnout model live; M56 county coordination active |
| Regulatory certification | T-90 to T-60 (2027-05-12 to 2027-06-11) | Elections (Technology) Regulations 2017: stress test, load test, pen test, functional test — certified by independent KEBS/CA-accredited firm; all 69 modules in scope |
| Voter list public inspection | T-90 to T-60 (2027-05-12 to 2027-06-11) | M64 portal live; objection intake open; M36 deregistration appeals active |
| Pilot deployment | T-60 to T-30 (2027-06-11 to 2027-07-11) | 2% sample stations (approx 1,100 stations across 10 constituencies); all mobile apps in-field; pilot feedback incorporated |
| National deployment | T-30 to T-14 (2027-07-11 to 2027-07-27) | All 55,393 KIEMS kits provisioned; all ~250K workers engaged via M55 + dispatched via M33; all 69 module hashes in Kenya Gazette |
| Readiness rehearsals | T-14 to T-1 (2027-07-27 to 2027-08-09) | Daily war-room rehearsal; M40 Supreme Court dry run (evidence pack extraction); M60 medical coordination drill; helpdesk team at stations |
| Election day | T-0 (2027-08-10) | All 69 modules live; ECC (M16) staffed 24/7; SRE on-call; 200 field engineers via M32; M60 medical coordination active |
| Post-election operations | T+1 to T+30 (2027-08-11 to 2027-09-09) | Progressive Forms 34-38 publication (M51); RLA audit (M21); petition evidence packs on demand (M40); reverse logistics via M65 (diaspora); M66 financial forensics begins; source-code + IPR transfer package for all 69 modules delivered |
| Post-election review | T+30 to T+60 | M41 findings registered; M69 archive ingestion begins; M66 cost-per-vote analytics published |
| Petition window support | T+30 to T+180 (2027-09-09 to 2028-02-05) | Evidence packs on demand; expert witness support for court proceedings; M21 RLA final report; M63 ballot-serial chain reconciliation for any disputed station |
| Framework-contract steady-state | T+180 to T+3 years (2028-02 onwards) | Quarterly security reviews; annual pen test; dependency freshness; by-election support via M50; SBOM publication; M69 archive format-migration pipeline verified; M55 HR process ready for by-election workers |

---

## Section 8 — Team Structure and Role Definition

### 8.1 Programme leadership

| Role | Count | Responsibility |
|---|---|---|
| Programme Director | 1 | Contract performance, IEBC relationship, risk governance |
| Deputy Programme Director | 1 | Day-to-day operations, milestone tracking |
| PMO (Project Managers + Coordinators) | 4 | Per-module tracking, RAID log, milestone reporting |

**Subtotal: 6**

### 8.2 Architecture

| Role | Count |
|---|---|
| Chief Architect | 1 |
| Platform Architect (infrastructure + mesh + observability) | 1 |
| Security Architect | 1 |
| Data Architect (event sourcing, data model, BI) | 1 |

**Subtotal: 4**

### 8.3 Security and data protection

| Role | Count |
|---|---|
| Security Lead | 1 |
| Application Security Engineers | 3 |
| Security Operations (SOC during election period) | 2 |
| Data Protection Officer | 1 |
| DPA Compliance Analyst | 1 |

**Subtotal: 8**

### 8.4 Domain engineering teams (per major module group)

Each team: 1 lead + 2-4 back-end engineers + 1 QA = 5-7 per team. 69-module scope adds 7 teams to the 28-module baseline. The 41 new modules cluster into manageable teams because: (a) 6 mobile apps share a Flutter codebase and platform patterns from M01/M05/M13; (b) register-maintenance modules (M35, M36, M53, M64) extend M02 with shared schema and patterns; (c) party/candidate modules (M45-M47, M67) extend M12 with shared domain; (d) most remaining modules are lower-complexity workflow systems (8-12 pm each) that a 4-person team can deliver in parallel.

| Team | Modules | People |
|---|---|---|
| BVR/ABIS team | M01, M02, M03 | 8 |
| VPRS/BVV team | M04, M07 | 6 |
| Register maintenance team | M35, M36, M53, M64 | 8 |
| BVI/Inventory team | M05, M06, M20 | 9 |
| Ballot chain team | M39, M63, M65 | 6 |
| RTS team | M08, M09, M10, M51, M62 | 14 |
| Portal/Verifier team | M11, M13, M29, M68 | 10 |
| CRMS/Campaign Finance team | M12, M22 | 6 |
| Party/Candidate team | M45, M46, M47, M67 | 8 |
| Transparency/RLA team | M14, M15, M21, M57, M58, M59 | 10 |
| ECC/Speech Integrity team | M16, M17, M44, M60 | 12 |
| Boundary/Queue/Calendar team | M18, M19, M52 | 6 |
| Governance team | M23, M24, M25, M40, M48, M49, M50 | 12 |
| Mobile field apps team | M30, M31, M32, M33, M34 | 10 |
| Anomaly/AI/Forensics team | M26, M27, M28, M66 | 11 |
| Training/Accreditation team | M37, M38, M41, M42, M43 | 10 |
| Support/Admin team | M54, M55, M56, M61, M69 | 10 |

**Subtotal: 156**

### 8.5 Platform team (SRE, DevOps, observability)

| Role | Count |
|---|---|
| SRE Lead | 1 |
| SREs | 4 |
| DevOps / Build Engineers | 3 |
| Observability Engineer | 2 |

**Subtotal: 10**

### 8.6 Data and ML team

| Role | Count |
|---|---|
| Data Science Lead | 1 |
| Data Engineers | 3 |
| ML Engineers (NLP, CV, statistical) | 4 |

**Subtotal: 8**

### 8.7 QA and test engineering

| Role | Count |
|---|---|
| QA Lead | 1 |
| QA Engineers (functional + regression) | 6 |
| Performance / Load Test Engineers | 2 |
| Biometric Accuracy Test Specialist | 1 |
| Security QA (SAST/DAST pipeline) | 2 |

**Subtotal: 12**

### 8.8 Mobile and front-end

| Role | Count |
|---|---|
| Mobile Lead (React Native) | 1 |
| React Native Engineers | 4 |
| Front-End Engineers (React/Next.js) | 5 |

**Subtotal: 10**

### 8.9 Kit-side embedded engineering

| Role | Count |
|---|---|
| Embedded Lead (Android Kotlin) | 1 |
| Kotlin / Jetpack Compose Engineers | 4 |
| Legacy Android (Java) Engineer | 1 |
| Biometric SDK Integration Specialist | 2 |

**Subtotal: 8**

### 8.10 Field operations and support

| Role | Count |
|---|---|
| Field Operations Lead | 1 |
| County ICT Support Engineers (47 counties) | 47 |
| Helpdesk Agents (election-day surge) | 30 |
| Kit Logistics Coordinators | 10 |

**Subtotal: 88**

### 8.5 Platform team (SRE, DevOps, observability)

| Role | Count |
|---|---|
| SRE Lead | 1 |
| SREs | 6 |
| DevOps / Build Engineers | 4 |
| Observability Engineer | 3 |

**Subtotal: 14**

### 8.6 Data and ML team

| Role | Count |
|---|---|
| Data Science Lead | 1 |
| Data Engineers | 5 |
| ML Engineers (NLP, CV, statistical, econometrics) | 6 |

**Subtotal: 12**

### 8.7 QA and test engineering

| Role | Count |
|---|---|
| QA Lead | 1 |
| QA Engineers (functional + regression) | 10 |
| Performance / Load Test Engineers | 3 |
| Biometric Accuracy Test Specialist | 1 |
| Security QA (SAST/DAST pipeline) | 3 |

**Subtotal: 18**

### 8.8 Mobile and front-end

| Role | Count |
|---|---|
| Mobile Lead (Flutter) | 1 |
| Flutter Engineers | 8 |
| Front-End Engineers (React/Next.js) | 8 |

**Subtotal: 17**

### 8.9 Kit-side embedded engineering

| Role | Count |
|---|---|
| Embedded Lead (Android Kotlin) | 1 |
| Kotlin / Jetpack Compose Engineers | 4 |
| Legacy Android (Java) Engineer | 1 |
| Biometric SDK Integration Specialist | 2 |

**Subtotal: 8**

### 8.10 Field operations and support

| Role | Count |
|---|---|
| Field Operations Lead | 1 |
| County ICT Support Engineers (47 counties) | 47 |
| Helpdesk Agents (election-day surge) | 40 |
| Kit Logistics Coordinators | 10 |
| Mobile-app deployment specialists (MDM) | 5 |

**Subtotal: 103**

### 8.11 Headcount summary

| Group | Peak headcount |
|---|---|
| Programme leadership + PMO | 6 |
| Architecture | 6 |
| Security + DPO | 10 |
| Domain engineering | 156 |
| Platform + SRE | 14 |
| Data + ML | 12 |
| QA + test | 18 |
| Mobile + front-end | 17 |
| Kit-side embedded | 8 |
| Field operations + support | 103 |
| **Total** | **350** |

**Why 350 (not 240)?** The 41 new modules add approximately 440 person-months of engineering work — roughly the same as M01-M28 combined. However, marginal headcount scales more slowly than marginal work because: (a) platform, SRE, architecture, security, and QA overhead is shared across 69 modules (not 41 new independent teams); (b) 14 of the 41 new modules (M29-M34 mobile apps, M37, M38, M42, M43, M52, M54, M58, M61) are lower-complexity (8-12 pm each) and can be staffed with 3-4 engineers per team; (c) code reuse between modules that share schemas (bvr-svc, crms-svc) reduces net-new engineering. Peak headcount of 350 occurs at T-270 to T-120. Field support (103) is election-day concentrated. Engineering peak is T-480 to T-180.

**Industry benchmark:** The EU eID large-scale project (2019-2023, 28 member states, 42 services) peaked at ~320 engineers. Kenya IEMS 69 modules with comparable security requirements and a tighter timeline justifies 350. The comparable 2022 IEBC programme (pre-contract) had ~180 staff across hardware and software; software-only scope at 69 modules and full cryptographic transparency is a materially larger build.

### 8.12 Kenyan vs. international mix

Target ≥ 70% Kenyan nationals across all non-field roles to meet PPADA local-content requirement (Form 8). International specialists engaged for ABIS engine integration, HSM/PKI ceremony, distributed-systems architecture review, and digital-preservation expertise for M69 (estimated ≤ 40 person-months international, up from 30 for 28-module scope).

---

## Section 9 — Build Effort and Cost

### 9.1 Per-subsystem effort summary

| Module | Person-months MID | Team type |
|---|---|---|
| M01 BVR Device | 18 | Mobile/embedded |
| M02 BVR Backend | 22 | Back-end |
| M03 ABIS | 16 | Integration + ML |
| M04 VPRS | 14 | Full-stack |
| M05 BVI Device | 16 | Mobile/embedded |
| M06 BVI Backend | 18 | Back-end |
| M07 BVV | 10 | Full-stack |
| M08 RTS Device | 14 | Mobile/embedded + ML |
| M09 RTS Returning Officer | 14 | Full-stack |
| M10 RTS Backend | 30 | Back-end + ML |
| M11 Public Portal | 18 | Full-stack |
| M12 CRMS | 14 | Full-stack |
| M13 Public Verifier App | 22 | Mobile + full-stack |
| M14 Reproducibility Bundle | 10 | Platform + ML |
| M15 IAB Dashboard | 8 | Full-stack |
| M16 Election Command Centre | 20 | Full-stack + GIS |
| M17 Speech Integrity | 20 | ML + back-end |
| M18 Boundary Transparency | 10 | GIS + full-stack |
| M19 Queue Transparency | 8 | Back-end |
| M20 Inventory Management | 20 | Full-stack + mobile |
| M21 RLA Audit Tool | 12 | Statistical + full-stack |
| M22 Campaign Finance | 12 | Full-stack |
| M23 Grievance Channels | 18 | Full-stack + ML |
| M24 Diaspora Voting | 22 | Full-stack + crypto |
| M25 Special Voter Workflows | 10 | Full-stack |
| M26 Anomaly Detection | 24 | Data + ML |
| M27 Intent Preservation | 10 | Mobile + ML |
| M28 AI Monitoring | 14 | MLOps |
| M29 Mobile Citizen Verify | 18 | Mobile + full-stack |
| M30 Mobile Party Agent | 14 | Mobile + back-end |
| M31 Mobile Poll Clerk | 12 | Mobile |
| M32 Mobile Field Support | 10 | Mobile + integration |
| M33 Mobile Personnel Dispatch | 14 | Mobile + back-end |
| M34 Mobile Observer | 10 | Mobile |
| M35 CVR Continuous Register | 18 | Back-end + data |
| M36 Voter Deregistration Appeal | 12 | Full-stack |
| M37 Voter Education CMS | 10 | Full-stack |
| M38 Poll Worker Training LMS | 12 | Full-stack + mobile |
| M39 Ballot Printing Chain | 12 | Back-end + integration |
| M40 Election Court IT Interface | 12 | Back-end |
| M41 Post-Election Review | 8 | Full-stack |
| M42 Media Accreditation | 8 | Full-stack |
| M43 Observer Accreditation | 8 | Full-stack |
| M44 Turnout Forecasting Analytics | 14 | Data + back-end |
| M45 Party Registration Management | 12 | Full-stack |
| M46 Party Primaries Oversight | 10 | Full-stack |
| M47 Nomination Fee Payment | 8 | Back-end + integration |
| M48 Recall Election Workflow | 10 | Full-stack |
| M49 Referendum Mechanics | 16 | Back-end + integration |
| M50 By-Election Operations | 12 | Back-end |
| M51 Result Declaration | 12 | Full-stack |
| M52 Election Calendar | 8 | Back-end |
| M53 Voter ID Card Production | 10 | Back-end + integration |
| M54 Election Budget + Expenditure | 8 | Full-stack |
| M55 HR + Payroll Temp Staff | 12 | Back-end + integration |
| M56 County Coordination | 10 | Back-end |
| M57 International Observer Portal | 8 | Full-stack |
| M58 Parliamentary/Judiciary Reporting | 8 | Full-stack |
| M59 Auditor-General Interface | 8 | Full-stack |
| M60 Medical Coordination | 8 | Back-end |
| M61 Secrecy Assurance Testing | 10 | Data + back-end |
| M62 Transmission Legal-Basis Audit | 8 | Back-end + data |
| M63 Ballot Serial Chain | 10 | Back-end + integration |
| M64 Voter List Public Inspection | 10 | Full-stack |
| M65 Diaspora Ballot Physical Logistics | 8 | Full-stack |
| M66 Election Financial Forensics | 12 | Data + ML |
| M67 Party Symbol/Colour Registry | 8 | Full-stack + CV |
| M68 Referendum Explainer Platform | 10 | Full-stack |
| M69 Long-Term Archive | 12 | Platform + data |
| Platform foundations (security, observability, PKI, secrets, GitOps, service mesh) | 90 | Platform |
| QA, SRE, PMO overhead (15% uplift on module work) | 148 | Cross-cutting |
| **TOTAL** | **1,106** | |

**Effort justification:** M01-M28 baseline is 414 module-pm + 132 platform/overhead = 546 pm. M29-M69 add 442 module-pm. Shared platform scales from 60 to 90 pm (1.5x, not 2.5x, because platform primitives are reused). QA/SRE/PMO overhead at 15% on (414+442) = 128 pm, rounded to 148 pm to reflect 69-module integration complexity. Total 1,106 pm. Industry benchmark: India's AADHAAR biometric identity system (1.3B records, 30+ integrated systems) peaked at ~900 engineer-years; Kenya IEMS at 1,106 pm (~92 engineer-years) for a narrower scope is proportionate.

### 9.2 Cost estimate

| Category | Unit cost | Volume | KES estimate |
|---|---|---|---|
| Kenyan senior engineers (70% of 1,106 pm) | KES 480,000/pm | 774 pm | KES 371.6 M |
| International specialists (30% of 1,106 pm, capped) | KES 1,100,000/pm | 332 pm | KES 365.2 M |
| Field operations (103 staff × 90 days peak) | KES 120,000/pm | 927 pm | KES 111.2 M |
| Infrastructure (3 years, 2 Kubernetes clusters, storage, CDN, 18 services) | — | — | KES 160.0 M |
| ABIS engine licence (3-year contract with IDEMIA/Innovatrics/Thales) | — | — | KES 80.0 M |
| HSM appliances + maintenance (Thales Luna 7, cluster of 3) | — | — | KES 45.0 M |
| Third-party SaaS/BSL licences (Vault Enterprise, PagerDuty, Crowdin, etc.) | — | — | KES 45.0 M |
| Independent security audits + pen tests (annual + pre-election, 69-module scope) | — | — | KES 40.0 M |
| Biometric accuracy lab certification | — | — | KES 10.0 M |
| Digital-preservation specialist and KNADS integration (M69) | — | — | KES 8.0 M |
| Contingency (15%) | — | — | KES 186.9 M |
| **Total rough order of magnitude (software + services)** | | | **KES 1,423 M** |

**Cost benchmark:** KES 1.42B for 69 modules at 1,106 person-months is KES 1.29M per module-pm all-in, consistent with a complex sovereign identity/elections project. The UK Electoral Commission's digital transformation (2019-2023, approx 15 modules) cost GBP 45M (~KES 7.4B at current rates) — Kenya IEMS at a narrower geographic scope and lower wage base is proportionate. The 28-module baseline at KES 884M scaled to 69 modules yields KES 884M × (1,106/546) × 0.88 (shared-platform efficiency) ≈ KES 1,563M; our KES 1,423M estimate is slightly below this linear extrapolation, reflecting genuine platform reuse.

Note: This is the Datacraft software and services component only. KIEMS hardware (Mechrevo) is a separate BOM. Total programme including hardware estimated at KES 3.5B–4.5B, reflecting the larger software component, hardware costs unchanged from the 28-module BOM.

---

## Section 10 — Delivery Risk Register

| # | Risk | Owner | Probability | Impact | Mitigation | Trigger |
|---|---|---|---|---|---|---|
| R01 | ABIS engine vendor delays integration SDK delivery | M03 Tech Lead | Medium | High | Multi-vendor evaluation; parallel SDK evaluation with top 2 vendors; fallback to build-own engine if T-600 SDK not received | SDK not received by T-600 |
| R02 | IPRS API reliability insufficient for VPRS OTP at scale | M04 Tech Lead | Medium | High | Circuit breaker + graceful degradation to manual entry; pre-validate IPRS capacity with NIIMS T-600; SLA agreement with NIIMS | IPRS error rate >5% in pre-prod |
| R03 | 60K RTS session load test fails p99 target | M10 Tech Lead + SRE | Low | Critical | Stateless auth design; sharded Kafka; Redis session cache; HPA; load test at T-180 with 90% of target; 2-week remediation window | p99 >200ms at T-180 load test |
| R04 | ICR misreads Form 34A causing tally discrepancy | M10 ML Lead | Medium | High | Human review for all ICR outputs below 0.90 confidence; recount from signed image is authoritative; ICR labelled "auxiliary" always | ICR accuracy <95% on test set |
| R05 | Public Portal DDoS at publication moment | SRE Lead | High | High | Static-first CDN; signed snapshots served at edge; upstream DDoS scrubbing (Cloudflare/Akamai); origin auto-scales behind CDN | Traffic >2× baseline before CDN cache is warm |
| R06 | Per-kit certificate revocation fails to propagate within 60s | Security Architect | Low | Critical | OCSP must-staple + CRL push via MDM; test propagation time weekly from T-180; fallback: manual kit blacklist via M06 API | Propagation test >60s |
| R07 | Legacy 14,100 Android devices have undisclosed OS/SDK constraints | M01/M05 Tech Lead | Medium | High | Full device inventory + OS/SDK audit requested at T-720; compatibility layer designed for lowest common denominator; device farm CI for all SKUs | Device inventory not received by T-660 |
| R08 | Kenyan hiring: senior Python/Kotlin engineers in short supply | Programme Director | Medium | Medium | Start hiring T-720; partner with Nairobi tech community (iHub, Andela alumni); hire graduate cohort at T-600 for ramp-up; use international remote for shortfall up to 30% budget | <60% headcount target met by T-600 |
| R09 | Data migration from existing register finds more anomalies than expected | M02 Data Lead | Medium | Medium | 6-week deadline is hard; begin ETL analysis immediately at CED; exception queue + human review workflow; staged migration (clean records first, exceptions last) | >5% exception rate in ETL |
| R10 | IEBC data centres not ready at CED | Programme Director | Low | High | Contractual readiness gate at T-720; sandbox runs on Datacraft infrastructure until IEBC ready; IaC enables 48-hour migration to IEBC when ready | Data centres not ready by T-660 |
| R11 | Regulatory certification (Elections Tech Reg 2017) fails first attempt | QA Lead + Security Lead | Low | Critical | Certification scope defined T-540; mock certification T-180; independent pen-test T-180; remediation window T-150 to T-120 | Any finding rated Critical or High at T-180 mock |
| R12 | WhatsApp Business API terms change or rate limits imposed near election | M13 Tech Lead | Medium | Medium | USSD + SMS as backup channels for all WhatsApp queries; rate-limit headroom designed at 5× expected peak; meta commercial agreement T-360 | WhatsApp rate limit below 10M queries/day |
| R13 | Diaspora voting cryptographic design (homomorphic) proves infeasible in timeline | M24 Crypto Lead | Medium | High | Fallback to threshold encryption without homomorphic tally (uses mix-net for privacy); R&D gate at T-480; go/no-go decision at T-480 | Homomorphic prototype not complete by T-480 |
| R14 | Insider tamper with results by privileged Datacraft or IEBC staff | Security Architect | Low | Critical | Two-person integrity enforced by policy; escrow mirror independent of production; transparency log public; break-glass usage is public event; IAB has independent feed | Unexplained divergence between production and escrow mirror |
| R15 | Election calendar change post-mobilisation | Programme Director | Low | Medium | All milestones are T-relative; recalculate on new T-0 notification; contractual change-order process; contingency buffer of 30 days built into T-300 to T-180 phase | Gazette change to election date |
| R16 | 69-module integration complexity causes cascade failures across 18 services | Chief Architect + SRE Lead | Medium | High | Linkerd 2 service mesh isolates failures; circuit breakers on every inter-service call; chaos engineering weekly from T-300; OpenAPI contract tests enforce module boundaries; each service has independent failure mode documented | Cross-service error rate >1% in integration test |
| R17 | M62 audit-SDK adoption: one module fails to emit audit events | M62 Tech Lead | Medium | High | Shared pip package enforced by CI lint rule; audit-coverage report generated weekly; any module with <100% coverage blocked from deployment by ArgoCD pre-sync hook | Audit-coverage report below 100% at T-270 |
| R18 | M55 290K M-Pesa bulk disbursement fails on election night (Safaricom capacity) | M55 Tech Lead + SRE | Medium | Critical | Pre-tested with Safaricom for 290K-transaction batch at T-120; idempotency keys prevent double payment; bank-transfer fallback pre-authorised in runbook; escrow account pre-funded | M-Pesa batch API error rate >1% in T-120 test |
| R19 | M35 CVR data quality: NRB/CRS batch introduces systematic register errors before election | M35 Data Lead | Medium | High | Reconciliation checks on every batch; anomaly flag on change rates >0.5%; M36 appeal pipeline as safety valve; M64 public-inspection window as final correction gate; batch is held for human review if anomaly flag fires | >0.5% change rate in a single CRS/NRB batch within T-180 |
| R20 | M33 GPS spoofing by polling personnel (fake station check-in) | Security Architect | Medium | Medium | Multi-GNSS cross-check; Play Integrity attestation; station-coordinate distance anomaly flag in M16 ECC; post-election forensic report from M33 GPS logs | >5% of check-ins flagged as coordinate-anomaly |
| R21 | M49 referendum mode-switch to 65K kits fails due to MDM propagation delay | M49 Tech Lead + SRE | Low | Critical | Mode-switch rehearsal at T-90 and T-60; signed MDM config push with delivery confirmation; fallback: manually signed config USB stick pre-loaded at kit-prep; mode-switch is additive (referendum mode adds, does not replace, general-election firmware) | Mode-switch delivery confirmation <99.9% of kits at T-1 |
| R22 | M69 KNADS integration: Kenya National Archives API unavailable or non-existent | M69 Tech Lead | High | Low | KNADS integration falls back to physical deposit of BagIt packages on signed physical media if API is unavailable; API unavailability does not block archive; archive is functionally complete without KNADS API | KNADS API not available at T+180 |
| R23 | Hiring Flutter engineers for M29-M34 (6 new mobile apps): talent pool thinner than React Native | Programme Director | Medium | Medium | Flutter Kenya community is active (GDG Nairobi); 6 mobile apps share one Flutter codebase with different back-ends — hire 8 Flutter engineers and rotate across apps; React Native fallback for M29/M34 (public apps) if Flutter talent insufficient | <6 Flutter engineers hired by T-540 |
| R24 | M66 financial-forensics econometric model incorrectly identifies candidates as vote-buyers (false positive) | M66 Econometrics Lead | Medium | High | Dual-method requirement (two independent models must agree at p<0.01); referral is advisory to ODPP only; ODPP independently evaluates; full methodology published so ODPP can re-run; legal review of referral threshold by IEBC Legal team before first publication | Any referral generated before methodology is OAG-reviewed |
| R25 | 69-module scope causes scope-creep in individual modules (gold-plating) | Programme Director | High | Medium | Hard character-count budget per module spec (this document); PM weekly scope review; any scope addition requires CCB approval; effort estimate re-baselined if scope changes exceed 10% | Any module exceeding its HIGH effort bound by >20% at T-360 |

---

## Section 11 — Open Questions to Resolve Pre-Build

| # | Question | Owner | Must resolve by |
|---|---|---|---|
| Q01 | Which ABIS vendor (IDEMIA / Innovatrics / Thales)? SDK delivery timeline? | IEBC + Datacraft joint | T-660 |
| Q02 | Existing ABIS: does IEBC retain it or replace entirely? (Contract clarification C6) | IEBC | T-660 |
| Q03 | Full device inventory for ~14,100 legacy Android units: make, model, Android version, API level, biometric SDK already installed? | IEBC | T-680 |
| Q04 | Diaspora voting: which foreign jurisdictions are gazetted? What are the biometric capture options at each? | IEBC + Ministry of Foreign Affairs | T-600 |
| Q05 | IPRS: does NIIMS/KNBS provide a test environment before T-600? What is the production rate limit? | IEBC + NIIMS | T-680 |
| Q06 | HSM: has IEBC selected Thales Luna 7, Utimaco, or Entrust? Has hardware been procured? When is it available for PKI ceremony? | IEBC | T-690 |
| Q07 | KEBS and Communications Authority: which accredited independent certifier will conduct Elections (Tech) Reg 2017 certification? IEBC must nominate per the regulations. | IEBC | T-600 |
| Q08 | Diaspora voting legal authority: is the Elections Act amendment in force? What is the constitutional basis for diaspora voting in the 2027 election? | IEBC + Attorney General | T-540 |
| Q09 | IEBC data centres (primary + mirror): are they operational and network-connected at CED? What is the converged hardware spec? | IEBC | T-720 (CED) |
| Q10 | Telco VPN: which telco (Safaricom / Airtel / Telkom) is preferred carrier? Is dual-SIM + eSIM commercial arrangement in place? | IEBC | T-660 |
| Q11 | ODPC registration timeline: is IEBC's registration as data controller current? Has IEBC registered the IEMS as a new processing activity? | IEBC DPO | T-660 |
| Q12 | Speech Integrity (M17) escalation authority: which national authority receives escalations — NCIC, NCA, DCI, or all three? What is the escalation protocol and response SLA? | IEBC + NCIC + NCA | T-540 |
| Q13 | RLA (M21): what is IEBC's target risk limit (alpha)? Is 5% acceptable? Does IEBC require a full hand-count if the RLA does not stop? | IEBC statistician + IAB | T-480 |
| Q14 | Independent escrow custodian for Merkle audit log: has IEBC designated a custodian (Auditor-General / university)? What is the data-handoff protocol? | IEBC + Auditor-General | T-540 |
| Q15 | Electoral boundaries: is the boundary delimitation for 2027 complete and gazetted? When will final GeoJSON be available for M18? | IEBC Boundaries + Cabinet Secretary | T-540 |

---

## Section 12 — Success Criteria

The system is "shipped" on 2027-08-10 when every item below is true:

- All 55,393 polling stations open on time with a functional, certified KIEMS kit authenticated to the back-end.
- BVI authenticates voters in ≤ 3 seconds (fingerprint) and ≤ 5 seconds (iris / facial) at p90, across all open stations simultaneously.
- RTS back-end (M10) processes 60,000 concurrent sessions at p99 authorisation latency ≤ 100 ms for the full polling window.
- Zero KIEMS kit suffers unrecoverable failure for more than 15 consecutive minutes during the polling window (L1 spare deployed within SLA).
- First provisional national results publication occurs ≤ 1 hour after the last polling station closes.
- Every Form 34A is signed, transmitted, and retrievable from the Merkle-anchored store within 30 minutes of station close.
- The Public Results Portal (M11) serves ≥ 5 million concurrent unique visitors without degrading to non-signed content.
- Any citizen using M13 can verify their station's Form 34A hash against the Merkle root in ≤ 30 seconds on a 3G connection.
- M14 Reproducibility Bundle, published before polls close, allows an independent auditor to re-derive the national presidential tally on a commodity laptop in ≤ 4 hours, producing byte-identical results.
- M26 Anomaly Detection has produced zero unreviewed Level-3 alerts at election close; every alert has an assigned IEBC owner and a documented disposition.
- M15 IAB Dashboard has been continuously updated with < 5-minute lag throughout election day; zero unexplained divergence between IAB feed and production Merkle root.
- Zero P1 security incidents are undetected or uncontained during the polling window.
- Zero unauthorised bulk exports of the voter register occur during the contract period.
- Every court petition (up to T+30) receives a complete, cryptographically proven evidence pack within 24 hours of formal request.
- Elections (Technology) Regulations 2017 certification passed by independent accredited certifier at T-60 or earlier.
- ISO/IEC 27001:2022 certification obtained by T-120 or earlier.
- ODPC compliance audit at T+30 returns no P1 data-protection findings.

---

## Consolidated Build Manifest (28 Modules)

| # | Module | Ownership | Effort (pm MID) | Delivery horizon |
|---|---|---|---:|---|
| M01 | BVR Device Client | BUILD-NEW | 18 | T-270 field-ready |
| M02 | BVR Backend | BUILD-NEW | 22 | T-420 register ops |
| M03 | ABIS Integrator | INTEGRATE + BUILD | 16 | T-300 certified |
| M04 | VPRS | BUILD-NEW | 14 | T-600 public |
| M05 | BVI Device Client | BUILD-NEW | 16 | T-180 certified |
| M06 | BVI Backend | BUILD-NEW | 18 | T-180 certified |
| M07 | BVV | BUILD-NEW | 10 | T-600 public |
| M08 | RTS Device Client | BUILD-NEW | 14 | T-180 certified |
| M09 | RTS Returning Officer | BUILD-NEW | 14 | T-180 certified |
| M10 | RTS Backend | BUILD-NEW | 30 | T-120 load tested |
| M11 | Public Results Portal | BUILD-NEW | 18 | T-120 CDN tested |
| M12 | CRMS | BUILD-NEW | 14 | T-420 party portal |
| M13 | Public Verifier App | BUILD-NEW | 22 | T-120 app stores |
| M14 | Reproducibility Bundle | BUILD-NEW | 10 | T-0 published |
| M15 | IAB Dashboard | BUILD-NEW | 8 | T-60 live |
| M16 | Election Command Centre | BUILD-NEW | 20 | T-60 rehearsed |
| M17 | Speech Integrity | BUILD-NEW | 20 | T-30 live |
| M18 | Boundary Transparency | BUILD-NEW | 10 | T-420 public |
| M19 | Queue Transparency | BUILD-NEW | 8 | T-60 live |
| M20 | Inventory Management | BUILD-NEW | 20 | T-180 live |
| M21 | RLA Audit Tool | BUILD-NEW | 12 | T-60 ready, T+7 run |
| M22 | Campaign Finance Portal | BUILD-NEW | 12 | T-360 public |
| M23 | Grievance Channels | BUILD-NEW | 18 | T-60 live |
| M24 | Diaspora Voting | BUILD-NEW | 22 | T-120 pilot |
| M25 | Special Voter Workflows | BUILD-NEW | 10 | T-300 ready |
| M26 | Anomaly Detection | BUILD-NEW | 24 | T-60 live |
| M27 | Intent Preservation | BUILD-NEW | 10 | T-120 live |
| M28 | AI Monitoring | BUILD-NEW | 14 | T-180 live |
| Platform foundations | Cross-cutting | BUILD-NEW | 60 | T-660 baseline |
| QA + SRE + PMO overhead | Cross-cutting | — | 72 | Continuous |
| **TOTAL** | | | **546** | |

**IPR classification:** All 28 modules are Custom Software; IPR vests in IEBC at Operational Acceptance per GCC 15.4.

**Scope count:** 28 core modules + cross-cutting platform foundations = one integrated, sovereign, end-to-end elections management system.

---

*Document owner: Datacraft Solutions Architecture*
*Companion documents: PRD.md, ARCHITECTURE.md, DATA-MODEL.md, SECURITY.md, modules/M01-M28*
*Next revision: at Preliminary Project Plan approval (T-690)*

---

## Appendix A — Compliance Mapping (Modules vs. Regulatory Framework)

Every module must exhibit compliance with the entire NFR stack from PRD §7. The table below maps the primary regulatory anchors per module for traceability during the Elections (Technology) Regulations 2017 certification audit and any court petition.

| Module | Elections Act | Elections (Tech) Reg 2017 | DPA 2019 | ISO 27001 | Constitution |
|---|---|---|---|---|---|
| M01 BVR Device | §6A, §44 | Reg 5 (functional), Reg 6 (data capture) | §25 (biometric), §31 (consent) | A.8 (access control) | Art 83 (right to register) |
| M02 BVR Backend | §44, §18 | Reg 6, Reg 7 (system performance) | §26 (minimisation), §50 (retention) | A.12 (operations) | Art 83 |
| M03 ABIS | §44 (accuracy) | Reg 6 (dedup), Reg 11 (accuracy test) | §25 (biometric processing) | A.14 (system acquisition) | Art 83(1) |
| M04 VPRS | §6A | Reg 5, Reg 6 | §31 (consent), §26 | A.9 (access) | Art 83(1) |
| M05 BVI Device | §44A | Reg 8 (polling day authentication) | §25 | A.8, A.12 | Art 86 (free and fair) |
| M06 BVI Backend | §44A | Reg 8, Reg 4 (system integrity) | §25, §26 | A.12, A.17 (DR) | Art 86 |
| M07 BVV | §18 (inspection) | Reg 5 | §26, §29 (access rights) | A.9 | Art 83(3) |
| M08 RTS Device | §39(1C) | Reg 10 (transmission), Reg 11 | §26 (results are public) | A.12, A.14 | Art 138(3) |
| M09 RTS-RO | §39(1C) | Reg 10 | §26 | A.12 | Art 138(3) |
| M10 RTS Backend | §39(1C) | Reg 11 (stress, load, pen, functional) | §26 | A.12, A.17 | Art 138(10) |
| M11 Public Portal | §74 | Reg 5 | §26 | A.12 | Art 138(10) |
| M12 CRMS | §22, §27 | Reg 5 | §26 (candidate data — limited PII) | A.9, A.12 | Art 99, Art 177 |
| M13 Verifier App | §74 | Reg 5 | §26 | A.12 | Art 138(10) |
| M14 Reproducibility | §74 | Reg 11 (auditability) | §26 | A.12 | Art 138 |
| M15 IAB Dashboard | §44A | Reg 4, Reg 11 | §26 | A.9 | Art 86 |
| M16 ECC | §44A | Reg 4 | §47 (public interest processing) | A.12, A.16 | Art 86 |
| M17 Speech Integrity | — (NCA mandate) | — | §47 (public interest), §26 | A.12 | Art 33, Art 34 (with limits) |
| M18 Boundary | IEBC Act §26 | Reg 5 | §26 | A.12 | Art 89 |
| M19 Queue | §55 (PO duty) | Reg 8 | §26 | A.12 | Art 86 |
| M20 Inventory | PPDA 2015 | Reg 4 | §26 | A.8, A.11 | — |
| M21 RLA | Auditor-General Act | Reg 11 | §26 | A.18 (audit) | Art 229 (Auditor-General) |
| M22 Campaign Finance | §§103-106 | Reg 5 | §26 | A.9 | Art 88(4)(l) |
| M23 Grievance | §75, IEBC Act §4(c) | Reg 5 | §26, §29 (complainant rights) | A.16 | Art 47 (fair admin action) |
| M24 Diaspora | §44 (as amended) | Reg 8 | §25, §31 | A.12, A.14 | Art 83 |
| M25 Special Voter | §50 | Reg 8 | §25, §26, §29 | A.9 | Art 54 (persons with disabilities) |
| M26 Anomaly Detection | §44 | Reg 4, Reg 11 | §47 | A.12 | Art 86 |
| M27 Intent Preservation | §39(1C) | Reg 10 | §26 | A.12 | Art 138 |
| M28 AI Monitoring | DPA 2019 §47 | — | §47 (automated processing) | A.14 | — |

---

## Appendix B — Integration Catalogue

All integrations are built by Datacraft as resilient adapters with circuit breakers, retry logic (exponential backoff with jitter), timeout budgets, structured error logging, and their own test harnesses. No integration is an unmediated direct DB connection.

| Integration | Upstream system | Protocol | Auth method | Resilience pattern | Modules consuming |
|---|---|---|---|---|---|
| I01 IPRS | NIIMS / KNBS | REST/JSON over HTTPS | mTLS + API key | Circuit breaker, retry 3×, graceful degradation to manual entry | M01, M04, M05 |
| I02 MRZ parser | Internal (built library) | In-process | n/a (library call) | Input validation, quality gate, versioned parser | M01 |
| I03 Telco VPN | Safaricom / Airtel / Telkom | BGP-routed IPsec VPN | Pre-shared keys (HSM-stored) + X.509 | Carrier rotation, dead-peer-detection, automatic failover | M01, M05, M08, M09 |
| I04 IEBC Power BI | Microsoft Power BI | Power BI REST API + Semantic Model push | OAuth2 service principal | Async push, queue-backed, failure does not block election ops | M02, M06, M10, M20 |
| I05 Existing ABIS (if retained) | IEBC legacy ABIS | Vendor-specific (IDEMIA / Neurotechnology) | Certificate + API key | Adapter pattern, versioned mapping, fallback to new ABIS (M03) | M03 |
| I06 Legacy Android kits | ~14,100 Android devices (various SDK levels) | Same REST + gRPC as new kits | Per-device X.509 cert (same PKI) | Feature-matrix compatibility shim; graceful feature disable if hardware missing | M01, M05, M08 |
| I07 HSM signing service | Thales Luna 7 / Utimaco | PKCS#11 + vendor REST (Luna REST API) | HSM admin credentials (M-of-N quorum) | HSM cluster ×3; queue overflow alerts; signing never blocks on single HSM node | All modules that sign events |
| I08 SMS / USSD gateway | Africa's Talking / Safaricom Bulk SMS | REST/JSON | API key (Vault-managed) | Retry 3×; queue-backed; gateway switching if primary fails | M04, M07, M13, M19, M23 |
| I09 ODPC reporting | Office of the Data Protection Commissioner | Email + structured JSON upload | Keycloak service account | Manual trigger + automated 72-hour breach notification; no real-time dependency | M02, DPO compliance tooling |
| I10 KEBS / OEM warranty feed | Mechrevo / KEBS | SFTP + structured CSV | KEBS-issued credentials | Async pull; failure does not block operations; reconciled weekly | M20 |
| I11 WhatsApp Business API | Meta Cloud API / Twilio Conversations | REST/JSON webhook | OAuth2 bearer (Vault-managed) | Rate-limit header monitoring; Redis cache pre-warm; SMS fallback | M13, M23 |
| I12 KRA (candidate tax status) | Kenya Revenue Authority | REST/JSON (if KRA provides API) or manual verification | mTLS + API key | One-time verification at nomination; manual fallback if API unavailable | M12 |
| I13 IPFS pinning | Filebase / Web3.Storage / self-hosted Kubo | IPFS HTTP API | API key (Vault-managed) | Pinning to 3+ nodes; fallback to GitHub release asset | M14, audit log anchoring |
| I14 Satellite fallback | Starlink / VSAT (regional hubs) | IP over satellite | IPsec | Last-resort transport when all telco carriers fail; high latency tolerated for non-real-time paths | M01, M05, M08 edge nodes |

---

## Appendix C — Non-Functional Requirements Enforcement per Module

The PRD §7 NFR stack is inherited by every module. The table below documents the per-module enforcement mechanism for the four most critical NFRs.

| Module | Availability enforcement | Latency enforcement | Offline-first enforcement | Audit-log enforcement |
|---|---|---|---|---|
| M01 BVR Device | SQLCipher local queue; no network dependency for capture | UI response ≤200 ms enforced by Jetpack Compose profiler in CI | Full local queue; 5,000-record backlog capacity | Local event log signed at creation; synced on reconnect |
| M02 BVR Backend | Patroni HA + HAProxy; 3-replica K8s; HPA | SQLAlchemy async; Redis session cache; p99 monitored in Grafana | Not applicable (back-end is always-on) | append_audit() call on every mutation; Kafka outbox |
| M03 ABIS | 3-node matching cluster; queue-backed; single node kill tolerated | p99 ≤8 s monitored; SLA timer per case; alert at 80% budget consumed | Not applicable | Match results signed by ABIS service cert; Kafka outbox |
| M04 VPRS | CDN-cached public pages; K8s HA; Redis rate limit | SSR p99 ≤500 ms; Next.js ISR for static pages | IndexedDB draft on web client (form recovery) | Every pre-reg event in event store; Kafka outbox |
| M05 BVI Device | Local poll-book; no network dependency for auth | Fingerprint match ≤3 s enforced by biometric SDK profiler | Full local register bundle; 700-voter capacity | Poll-book events signed at source; submitted on reconnect |
| M06 BVI Backend | 3-replica K8s; Redis session cache; HPA to 20 replicas | p99 ≤100 ms at 60K sessions; Prometheus alert at >50 ms | Register bundles pre-distributed; kits operate without back-end during polling window | Every kit event in event store; Kafka outbox |
| M07 BVV | CDN-cached static pages; K8s HA | SSR p99 ≤1 s; rate limiting protects back-end | Not applicable | Every lookup in audit log; no bulk export |
| M08 RTS Device | Local signed queue; retry with jitter | Scan-to-submission < 60 s nominal; jitter up to 10 min | Full offline queue; submits on reconnect | Scan event + submission event signed at kit |
| M09 RTS-RO | IndexedDB draft offline; submits on reconnect | Form 34B generation <5 s | Core count-entry works offline | Every entry + signature event in audit log |
| M10 RTS Backend | Patroni HA; HPA 20-60 replicas; PgBouncer | p99 ≤100 ms at 60K; sharded Kafka | Not applicable | Every ingested form in Merkle-backed event store |
| M11 Public Portal | CDN signed snapshots; origin failure has no voter impact | CDN p99 ≤200 ms; origin p99 ≤2 s | Static snapshot served from CDN edge | Access log at CDN + origin; no PII in logs |
| M12 CRMS | K8s HA (1-10 replicas); MinIO HA | Form submission <5 s | Draft auto-save every 30 s (browser IndexedDB) | Every nomination event in event store |
| M13 Verifier App | CDN + K8s; USSD/SMS as backup | PWA p99 ≤2 s on 3G; USSD <10 s | App caches last 100 lookups; offline shows cached result with staleness label | Lookup events logged server-side; no PII stored |
| M14 Reproducibility | Static download from CDN + IPFS + GitHub | Bundle execution ≤4 h on reference laptop | Fully offline execution after initial download | Hermetic build log; SHA-256 manifest |
| M15 IAB Dashboard | Separate K8s namespace; no dependency on production write paths | Grafana panel p99 ≤2 s | IAB has read replica; no production write dependency | IAB access log; divergence alerts |
| M16 ECC | Primary + air-gapped war-room secondary; <15 min lag | WebSocket push <1 s for P0 alerts | War-room secondary operates autonomously on local Kafka mirror | All ECC actions in audit log |
| M17 Speech Integrity | K8s + GPU pool HA | Classification p99 ≤5 s; batch acceptable for historical scan | Not applicable | Every alert event in audit log |
| M18 Boundary | CDN-cached GeoJSON tiles; K8s HA | Tile serve ≤200 ms from CDN | Static GeoJSON downloadable for offline use | Boundary change events in audit log |
| M19 Queue | K8s HA; Redis counters | Metric update ≤5 min lag | Not applicable (aggregate only) | Queue metric events in Kafka |
| M20 Inventory | K8s HA; React Native offline-first | Mobile app ≤2 s for custody scan | Field officer app queues custody events offline | Every custody event in tamper-evident log |
| M21 RLA | K8s; post-election; no election-day dependency | Ballot manifest generation <10 min | Not applicable | All audit-board decisions in audit log |
| M22 Campaign Finance | K8s HA; CDN for public portal | Public portal p99 ≤1 s from CDN | Draft auto-save | Every filing event in event store |
| M23 Grievance | K8s HA; Kafka async queue | Acknowledgement to citizen <5 s | App queues submission offline | Every grievance event in audit log |
| M24 Diaspora | K8s HA; USB-key ballot backup at diaspora centre | Ballot submission <30 s nominal | USB-key ballot backup if network fails | Every ballot encryption event in audit log |
| M25 Special Voter | K8s HA | RO console <2 s | BVI device extended offline window | Assisted-voting event with two-officer biometric |
| M26 Anomaly | K8s + Flink/Kafka Streams HA | Stream processing lag <10 s | Not applicable | Every alert in audit log |
| M27 Intent Preservation | Kit-side; no network dependency | Classification p99 <2 s on-device | Fully offline; adjudication events queued and submitted | Adjudication events signed at kit |
| M28 AI Monitoring | K8s HA + kit-side daemon | Drift alert <60 s from detection | On-device statistical test; queues alert | All inference audit events in Kafka |

---

## Appendix D — Glossary (Key Terms)

| Term | Definition |
|---|---|
| ABIS | Automated Biometric Identification System — the biometric deduplication and matching engine. M03 is the integrator; the engine itself is a third-party product (IDEMIA / Innovatrics / Thales). |
| BVI | Biometric Voter Identification — the polling-day voter authentication system (M05 device, M06 back-end). |
| BVR | Biometric Voter Registration — the voter enrolment system (M01 device, M02 back-end). |
| BVV | Biometric Voter Verification — the 30-day public register inspection system (M07). |
| CED | Contract Effective Date — the date the IEBC/OIT/01/2026-2027 contract comes into force. All T-relative milestones recalculate if CED changes. |
| CRMS | Candidate Registration and Management System (M12). |
| ECC | Election Command Centre — unified operational picture (M16). |
| HSM | Hardware Security Module — FIPS 140-3 Level 3+ appliance for key storage and signing. Thales Luna 7 or equivalent. |
| IAB | Independent Assurance Board — external oversight body with read-only access to the IEMS (M15 dashboard). |
| ICR | Intelligent Character Recognition — the AI-assisted text extraction from Form 34A scanned images. Output is always auxiliary; signed image is authoritative. |
| IEMS | Integrated Elections Management System — the complete 28-module system. |
| IPRS | Integrated Population Registration System — the Kenya national ID register operated by NIIMS/KNBS. |
| KIEMS | Kenya Integrated Elections Management System — the term for the biometric kit hardware. Not to be confused with IEMS (the software system). |
| MDM | Mobile Device Management — remote configuration, wipe, and OTA update capability for the KIEMS fleet (see M20 inventory + M06 BVI backend). |
| Merkle log | A cryptographic data structure where every audit event is hashed and chained; the root hash commits to the entire history. Used for tamper evidence and public verifiability. |
| mTLS | Mutual TLS — both parties present X.509 certificates; every service-to-service call uses mTLS enforced by Linkerd. |
| ODPC | Office of the Data Protection Commissioner — the Kenya data protection regulator. |
| OTA | Over-the-air — signed software update pushed to kits via MDM without physical access. |
| PKI | Public Key Infrastructure — the hierarchy of certificate authorities (root CA, intermediate CAs, per-kit leaf certs). Root CA is air-gapped; ceremony requires M-of-N quorum. |
| PO | Presiding Officer — the IEBC official responsible for a polling station on election day. |
| RLA | Risk-Limiting Audit — a statistically rigorous post-election ballot audit (SHANGRLA framework, M21). |
| RO | Returning Officer — the IEBC official responsible for a constituency's results (290 total). |
| RTS | Results Transmission System — the system that transmits, tallies, and publishes election results (M08 device, M09 RO module, M10 back-end). |
| SPIFFE/SPIRE | Secure Production Identity Framework For Everyone / SPIRE Runtime Environment — the CNCF workload identity system used for service-to-service authentication. |
| T-0 | Election day: 2027-08-10. All milestone dates are expressed relative to T-0 (e.g., T-90 = 2027-05-12). |
| TPM | Trusted Platform Module — a hardware chip in the KIEMS kit that stores the kit's private key and provides attestation. |
| VPRS | Voter Pre-Registration System — the public portal for citizen-initiated voter pre-registration (M04). |
| WSQ | Wavelet Scalar Quantization — the FBI-certified fingerprint image compression format. Required at 500 PPI, ≤15:1 average compression ratio. |
