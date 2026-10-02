# Security Requirements — the Commission IEMS 2027

**Version:** 2.0 — Full 69-module scope
**Companion to:** `PRD.md`, `ARCHITECTURE.md`, `DATACRAFT_SOFTWARE_SCOPE.md`
**Election day:** 2027-08-10

---

## 1. Threat Model

### 1.1 Attack Surface Overview

The Strategy paper (Part VIII) identifies over 100 distinct attack surfaces grouped below into nine threat categories. Each category maps to the modules that mitigate it.

**T1 — Silicon and Firmware Supply Chain**
An adversary (nation-state or commercial competitor) compromises the KIEMS kit firmware, RFID chip firmware, or biometric sensor firmware before delivery. Attack vectors: tampered factory ROM, malicious OTA, unsigned firmware accepted by boot chain. Mitigated by: M01, M05, M08, M27 (kit firmware); M14 (reproducibility verifies firmware hashes); M20 (asset-chain integrity); M61 (adversarial firmware testing). HARDWARE-REQUIREMENTS-FROM-MITIGATIONS §B4-B6 mandates TPM 2.0 + UEFI secure boot measured from the Commission Root CA.

**T2 — Software Supply Chain**
Malicious dependency, compromised build worker, or unsigned artefact in the CI/CD pipeline. Attack vectors: dependency confusion, typosquatting, CI environment compromise. Mitigated by: all 69 modules (hermetic builds, internal mirror, Sigstore/Cosign signing); M14 (SBOM per artefact, reproducible builds).

**T3 — Physical Device (Kit)**
Kit theft, substitution, or tampering at station or in transit. Attack vectors: stolen kit operating elsewhere, cloned kit, tampered tamper-evident seal. Mitigated by: M01, M05, M08, M27 (TPM attestation, geo-fence, time-fence, certificate-bound identity); M20 (inventory chain-of-custody); M61 (red-team physical attacks).

**T4 — Cryptographic Key and PKI**
Compromise of HSM, root CA, or intermediate CA. Attack vectors: quorum coercion, HSM firmware exploit, stolen custodian token, harvest-now-decrypt-later on long-lived audit records. Mitigated by: PKI ceremony (M-of-N quorum, Part XVIII §6), HSM cluster (Thales Luna 7 / Utimaco, FIPS 140-3 L3+), post-quantum hybrid (ML-KEM + X25519 for KEM; ML-DSA + ECDSA P-384 for signing per FIPS 204/205), M14 (re-signing schedule for long-lived archive), M61 (HSM adversarial testing).

**T5 — Kit-to-Back-End Network**
BGP hijack, DNS spoofing, man-in-the-middle, carrier outage, GPS spoofing. Attack vectors: certificate impersonation without pinning, telco coercion, radio jamming. Mitigated by: M01, M05, M08, M27 (mTLS + certificate pinning, multi-carrier dual-SIM, anti-jam GNSS); M10, M09 (kit authentication + signed rejection); M26 (connectivity anomaly alerts).

**T6 — Data-Centre Perimeter and Internal**
Lateral movement after perimeter breach, service-to-service abuse without mTLS, privilege escalation inside the cluster. Attack vectors: compromised service token, Kubernetes API server attack, SSRF. Mitigated by: Linkerd 2 service mesh mTLS on all intra-cluster paths; Kyverno admission policies; Falco + Tetragon runtime enforcement; network policies default-deny; AppArmor/SELinux per node. Affects all back-end modules (M02, M03, M06, M09, M10, M11, M12, M16, M26, M28, and all others).

**T7 — Application Stack**
SQL injection, authentication bypass, IDOR, race conditions in tally aggregation, ICR model poisoning, API abuse. Attack vectors: malformed biometric input to parsers, concurrent double-authentication, tampered form image. Mitigated by: M01-M12 (parameterised queries, idempotency keys, OPA per-request policy, ASVS L3 target); M26 (anomaly detection on tally); M28 (AI monitoring); M61 (pen-testing).

**T8 — Insider Threat**
Privileged operator modifying registers, tally, or audit logs; results-portal operator injecting results. Attack vectors: DBA with standing access, CI pipeline with too-broad scope, rogue developer. Mitigated by: JIT privilege (no standing admin), dual-control on bulk actions, OPA separation-of-duties policy, session recording (dual custody), independent parallel-tally (M16), two-person integrity for HSM operations. Affects M02, M03, M06, M10, M11, M12, M15, M16, M51, M59, M61.

**T9 — Availability and DDoS**
DDoS on Public Results Portal or RTS ingress; targeted station outage; carrier exhaustion. Attack vectors: amplification attacks on public portal, SYN floods on RTS, deliberate station kit failure. Mitigated by: M11 (static-first CDN, origin-less public read), M10 (sharded ingress, back-pressure), M26 (anomaly detector), multi-carrier + SMS/USSD fallback (M08, M09), geo-redundant three-site deployment.

**T10 — Social Engineering and Impersonation**
Phishing poll clerks, impersonating helpdesk, spoof clone of citizen apps, lookalike domains. Attack vectors: SMS phishing, spoofed the Commission domain, fake Citizen Verify app (M29) on unofficial app stores. Mitigated by: FIDO2 mandatory for privileged users (SMS MFA prohibited on admin); helpdesk in-band challenge; M28 (clone-app detection); M17 (deepfake/disinformation detection); DMARC/DKIM/SPF strict; K1 hardware-signed app release (HSM code-signing for M13, M29).

**T11 — Ballot Secrecy**
Inferring voter choice from RTS transmission sequence, party-agent app leaks, kit camera exfiltration. Attack vectors: traffic analysis, coerced voter photographing ballot, RTS metadata timing correlation. Mitigated by: M61 (dedicated secrecy assurance red-team); M08 (RTS transmission does not carry vote choices, only form images); M30 (party agent app shows only aggregate, no individual); M27 (intent preservation seals form before transmission).

**T12 — Evidentiary Chain Integrity**
Breaking the chain-of-custody for court petitions; tampering with Merkle roots; replay or rollback attacks on signed events. Attack vectors: re-ordering Merkle leaves, modifying archived form images, clock rollback, replay of earlier signed payloads. Mitigated by: M14, M40 (court-admissible evidence bundle); M63 (ballot serial chain); M69 (long-term archive with re-signing); monotonic counters + RFC 3161 timestamps prevent rollback; M21 (RLA confirms statistical consistency).

**Threat-to-Module Index**

| Threat | Primary Modules |
|---|---|
| T1 Silicon/firmware supply chain | M01, M05, M08, M14, M20, M27, M61 |
| T2 Software supply chain | M14 + all 69 (SBOM required) |
| T3 Physical device | M01, M05, M08, M20, M27, M61 |
| T4 Cryptographic keys / PKI | M02, M03, M06, M10, M14, M61, M69 |
| T5 Kit-to-back-end network | M01, M05, M08, M09, M10, M26, M27 |
| T6 DC perimeter / internal | All back-end services |
| T7 Application stack | M01-M12, M26, M28, M61 |
| T8 Insider threat | M02, M03, M10, M11, M12, M15, M16, M51, M59, M61 |
| T9 Availability / DDoS | M10, M11, M26, M08, M09 |
| T10 Social engineering | M17, M28, M29, M30, M61 |
| T11 Ballot secrecy | M08, M27, M30, M61 |
| T12 Evidentiary chain | M14, M21, M40, M63, M69 |

---

## 2. Security Architecture

### 2.1 Zero-Trust Internal

All intra-cluster service-to-service calls traverse Linkerd 2 service mesh. Every connection requires a valid SPIFFE SVID (X.509 certificate issued by SPIRE). mTLS is enforced on every pod-to-pod path; clear-text is not possible within the cluster. Short-lived SVIDs (TTL ≤ 1 hour) are rotated automatically; a compromised pod loses its identity within the rotation window without requiring manual revocation.

Network policies are default-deny. Only explicitly declared egress paths are allowed. SPIRE workload attestor uses Kubernetes API + TPM attestation to issue SVIDs only to pods whose identity is verifiable.

### 2.2 HSM-Anchored Root of Trust

The root of trust is a Thales Luna Network HSM 7 (or Utimaco SecurityServer) cluster at each of the three sites (primary, mirror, escrow), minimum six HSM units total. All units are FIPS 140-3 Level 3+. Keys are generated inside the HSM and never exported in clear form. Per-domain KEK/DEK hierarchy: a root KEK sealed to the HSM protects intermediate domain KEKs; each domain KEK protects data-encryption DEKs for that service's data. HSM firmware update policy: dual-custodian signed approval recorded in CCB minutes; firmware hash published to transparency log before installation.

### 2.3 Network Segmentation

Five zones, enforced by hardware firewall rules and Kubernetes network policies:

| Zone | Content | Isolation rule |
|---|---|---|
| Edge | CDN origin, WAF, load balancer | Internet-facing; no inbound from Core without data diode |
| DMZ | API gateway, Linkerd ingress, OCSP responder | Receives internet; forwards only over mTLS to Core |
| Core | Back-end services, databases, Kafka, MinIO | No direct internet; pod-level mTLS; default-deny egress |
| Air-gapped build | CI/CD workers, internal mirror, HSM admin | No internet egress; HSM admin only via dedicated terminal |
| Management | Bastion, monitoring, SIEM, SOC | Read-only reach to Core; no production write; session-recorded |

### 2.4 OS and Container Hardening

- CIS Kubernetes Benchmark Level 2 (automated scan on every deploy)
- CIS Ubuntu 22.04 LTS Benchmark Level 2 on every node
- Kernel: AppArmor profiles per pod; SELinux on RHEL-based nodes if applicable; seccomp profiles locked to minimum syscall set
- Container images: distroless base; no shell in production images; Cosign-signed at build; Kyverno admission control rejects unsigned or unverified images
- SBOM in SPDX 2.3 format per image, signed with Cosign + Sigstore; published to internal OCI registry; hash published to transparency log
- Runtime anomaly detection: Falco rules on suspicious syscalls + network; Tetragon eBPF for kernel-level enforcement

---

## 3. Identity and Access Management

### 3.1 External User Identity

OAuth 2.1 + OIDC via Keycloak 26. Token lifetimes: ID token 5 min, access token 15 min (interactive), 5 min (service-to-service). Refresh token rotation on every use; absolute maximum session 8 hours before re-authentication.

RBAC defines roles per module. ABAC via OPA (or Cedar) adds attribute-based conditions evaluated per request (e.g., a County Returning Officer can only submit results for their county code). Policy code is version-controlled, reviewed via CCB, and deployed through GitOps.

### 3.2 Workload Identity

SPIFFE/SPIRE provides workload identity. Every pod presents an SVID; no pod has a static long-lived credential. SPIRE workload attestor validates pod identity against Kubernetes API server + optional TPM attestation for bare-metal nodes. Compromise of one pod's SVID cannot authenticate as another pod.

### 3.3 Administrative Identity

Privileged users (the Commission CISO, Datacraft Security Lead, DBA, SOC analyst): FIDO2 hardware token mandatory (YubiKey 5 NFC or equivalent). SMS MFA is prohibited on any admin account. JIT privileged access via Vault dynamic secrets: no standing production credentials; each elevated session is ticketed (change window required), dual-approved, time-limited (max 4 hours), and session-recorded.

Break-glass accounts: dormant; activation requires court-attested order; activation event generates a public transparency-log entry immediately; session fully recorded. Deactivation is dual-custody. Used fewer than three times per year on average; each use triggers a post-activation review.

### 3.4 HSM Operations

HSM administrative actions require hardware token from two custodians from separate organisations (the Commission + Datacraft). Every HSM admin session is recorded on tamper-evident video. CA operations require M-of-N quorum (minimum 3-of-5 custodians, two organisations).

### 3.5 Joiner-Mover-Leaver

New engineer: access granted via signed ticket; HSM token provisioned only after background check and security briefing signature. Transfer: access reviewed within 24 hours; old role revoked before new role granted. Departure: all credentials revoked within 1 hour of HR notification; HSM token destroyed; session tokens invalidated; audit log entry created. Quarterly access review across all privileged roles.

### 3.6 Session Policy

Short-lived tokens as above. Screen-lock after 5 minutes on admin workstations. Concurrent session limit: 1 per user on admin roles. High-risk operations (key ceremony, bulk register export, results certification) require MFA re-authentication at operation time even within an active session.

### 3.7 Credential Rotation

| Credential type | Rotation cadence |
|---|---|
| Service account tokens (SPIFFE SVID) | ≤ 1 hour (automated) |
| Access tokens | 15 min (automated) |
| Database credentials (Vault dynamic) | Per session |
| API gateway keys | 90 days |
| Intermediate CA certificates | 1 year |
| Root CA certificate | 10 years (with quorum ceremony) |
| HSM firmware signing key | Per HSM firmware update |
| Post-quantum signing keys (ML-DSA) | 2 years or on algorithm advisory |

---

## 4. Cryptography Standards

### 4.1 Transport Security

TLS 1.3 mandatory on all external connections. No TLS 1.2 or earlier permitted; explicit TLS 1.2 fallback disabled on all servers. Cipher suites: TLS_AES_256_GCM_SHA384 and TLS_CHACHA20_POLY1305_SHA256 only. Perfect forward secrecy mandatory; static RSA key exchange prohibited.

Post-quantum hybrid for high-value sessions: ML-KEM-768 (FIPS 203) combined with X25519 via the hybrid KEM scheme (IETF draft-ietf-tls-hybrid-design). Deployed on: kit-to-back-end TLS, HSM admin TLS, PKI management plane. Legacy TLS sessions (observer portals, media accreditation) remain TLS 1.3 classical until browser support for PQ-hybrid stabilises (targeted at T-180).

### 4.2 Signing Algorithms

| Use case | Algorithm | Key size | Notes |
|---|---|---|---|
| Kit-signed events (real-time) | ECDSA P-384 | 384-bit EC | HSM-backed; per-kit private key never exported |
| Long-lived audit records (15-year) | Hybrid ML-DSA-65 + ECDSA P-384 | NIST FIPS 204 | Both signatures required; either alone is insufficient for petition |
| Software artefact signing | Cosign + Sigstore | ECDSA P-384 or Ed25519 | Transparency log entry per signature |
| Code commit signing | GPG Ed25519 | 256-bit | Developer hardware key; no software key stores |
| Citizen verifier QR code | ECDSA P-256 + BLAKE3 | 256-bit EC | P-256 for broad mobile compatibility |
| Post-quantum re-signing (archive) | SLH-DSA (FIPS 205) | stateless hash | Applied at T+1 year to Merkle roots for 15-year survivability |

### 4.3 Hashing

SHA-256 for general integrity (Merkle leaves, SBOM). SHA-384 for certificate digests. BLAKE3 for high-throughput local hashing (kit local audit log). HMAC-SHA256 for message authentication codes. No SHA-1, MD5, or CRC for integrity anywhere.

### 4.4 Encryption at Rest

AES-256-GCM for all persistent stores (PostgreSQL tablespace encryption via pg_crypto + OS-level LUKS, MinIO server-side encryption, Kafka log encryption, OpenSearch node encryption). Unique GCM nonce per encryption operation; nonce collision detection enforced. Biometric template fields: field-level AES-256-GCM with a separate DEK from the per-tenant KEK hierarchy; DEK stored in Vault, sealed to HSM.

Per-tenant KEK wrapping: each election cycle has an independent KEK. Post-election, the cycle KEK is escrowed to the auditor-general interface (M59) and then removed from active Vault. Cryptographic erasure of a cycle's data: destroy the cycle KEK.

### 4.5 Key Ceremony

Per Strategy Part XVIII §6: initial root CA key generation is an M-of-N quorum ceremony (3-of-5 custodians, minimum two organisations). Ceremony requirements: air-gapped HSM room, tamper-evident video, two the Commission witnesses, two Datacraft witnesses, one independent IAB witness. Ceremony log signed by all five custodians and published to transparency log within 24 hours. Subsequent CA operations require 3-of-5 quorum.

Key ceremony for intermediate CAs: separate ceremony from root; minimum 2-of-4 quorum. Kit certificate bulk issuance: automated under intermediate CA authority after initial quorum ceremony authorises the batch signing policy.

### 4.6 Key Rotation Cadence

| Key | Cadence | Trigger for emergency rotation |
|---|---|---|
| Root CA | 10 years | Custodian compromise, HSM hardware failure |
| Intermediate CA (kit) | 2 years | Kit-CA intermediate compromise |
| Intermediate CA (server) | 2 years | Server-CA intermediate compromise |
| Data-at-rest DEK | Per election cycle | Suspected compromise, ODPC order |
| Signing keys (ML-DSA) | 2 years | Algorithm advisory from NIST |
| Vault transit engine key | 1 year | Automated via Vault auto-rotate |

---

## 5. Data Protection

### 5.1 Kenya DPA 2019 Scope

the Commission is the Data Controller. Datacraft is a Data Processor under written Data Processing Agreement per DPA 2019 s.34. ODPC registration is required and held at go-live. All personal data processing is lawful, fair, and transparent per s.25.

DPA 2019 special-category data (s.29): biometric data (fingerprint, iris, face templates) is special-category. Processing is lawful under s.30(1)(b) (legal obligation — Elections Act §6 mandates biometric registration) only. No other basis applies to biometric templates. Processing is strictly purpose-limited to voter identification during registration (M01-M03) and authentication (M05-M06).

### 5.2 Encryption in Transit

TLS 1.3 on all external paths. mTLS (Linkerd 2 / SPIFFE) on all intra-cluster paths. Post-quantum hybrid on high-value external paths (kit, HSM admin). Certificate pinning on all KIEMS kit clients: hard-coded expected leaf fingerprints; reject any chain not matching regardless of OS trust store.

### 5.3 Encryption at Rest

AES-256-GCM as described in §4.4. Field-level encryption for biometric templates (separate DEK). Tokenisation: voter ID (national ID number) is tokenised to a surrogate UUID at the point of ingestion into the voter register. Internal cross-system references use the surrogate; national ID is stored once in the identity record, encrypted with a dedicated key. This limits blast radius if any non-identity service is compromised.

### 5.4 Pseudonymisation and Data Minimisation

Analytics and monitoring systems (M26, M28, M44) receive pseudonymised or aggregated data only. Station-level counts are permissible; individual-level voter data is never exposed to analytics workloads. AI models (M26, M28, M44) are trained on synthetic data or anonymised historical records only; no live voter PII is used in model training.

### 5.5 DSAR Handling

Data Subject Access Request workflow per DPA 2019 s.26 (right of access): request received via dedicated secure form → verified against identity evidence → response within 30 calendar days → response includes all personal data held, purposes, recipients, retention period. DSAR log is retained for 3 years post-response. Automated components of the response are generated from the voter register (M35 / M02); manual review by the Commission DPO before dispatch.

### 5.6 Breach Notification

Data breach detection triggers an incident response (§10 below). If breach involves personal data, ODPC notification is required within 72 hours of awareness per DPA 2019 s.43. Notification includes: nature of breach, categories and approximate number of affected data subjects, likely consequences, measures taken or proposed. Data subject notification within reasonable time where breach is likely to result in high risk. Breach log retained permanently.

### 5.7 Retention and Deletion

| Data category | Statutory basis | Retention period | Deletion mechanism |
|---|---|---|---|
| Biometric templates (enrolled) | Elections Act §6A | 5 years post-cycle | Cryptographic erasure (destroy DEK) per DPA s.41 |
| Voter register (full) | Elections Act + the Commission Act | 6 years post-cycle | Archive to M69 + selective purge per court order |
| RTS results + form images | Elections Act §39 | Permanent (national record) | No deletion; M69 long-term archive |
| Merkle chain / signed events | Court petition basis | Permanent | No deletion; M69 |
| Audit logs (operational) | ISO 27001 + the Commission policy | 7 years | Tiered: 90 days online, remainder in M69 |
| CRMS nomination data | Elections Act §24-§27 | 6 years | Selective anonymisation after 3 years |
| Campaign finance | Election Campaign Financing Act 2013 | 7 years | Archive |
| DSAR records | DPA 2019 | 3 years | Delete after retention period |
| Incident records | ISO 27001 | 5 years | Archive |

### 5.8 Right-to-Erasure Exceptions

Article 17 GDPR analogue (DPA 2019 s.36): the right to erasure does not apply where retention is required for a legal obligation or for the exercise or defence of legal claims. All RTS results, Merkle chain entries, and signed Form 34A images are subject to permanent evidentiary hold under Elections Act §39 and the Constitutional right to petition (Art. 87). Biometric templates are subject to erasure per the schedule above, subject to any pending court order requesting preservation.

---

## 6. Application Security

### 6.1 OWASP ASVS Level 3

Target: OWASP ASVS 4.0 Level 3 for all modules M01-M12 (election-critical). Level 2 minimum for all other modules. Level 3 requires: formal verification of authentication and session management, hardened session controls, detailed threat modelling per feature, formal code review. Verified at FAT by independent ASVS auditor.

### 6.2 SAST + DAST + SCA + IAST Toolchain

| Tool | Category | Gate |
|---|---|---|
| Semgrep (custom + community rules) | SAST | PR merge; high severity blocks |
| Bandit | SAST (Python) | PR merge |
| cargo-audit / npm-audit / pip-audit | SCA | PR merge; critical blocks |
| Trivy | Container SCA + secret scan | PR merge; critical blocks |
| Snyk / Sonatype Nexus IQ | SCA (commercial) | Weekly scan; high severity tickets |
| Gitleaks | Secret scan | Pre-commit hook + CI |
| OWASP ZAP | DAST | Nightly against staging; critical blocks |
| Burp Suite Enterprise | DAST (commercial) | Pre-release |
| Contrast Security (IAST) | IAST | Continuous in integration environment |

Renovate bot manages dependency version PRs; weekly automated PR created for minor/patch updates. Major version updates require human CCB approval.

### 6.3 Mobile App Hardening

Applies to M29, M30, M31, M32, M33, M34 (all React Native mobile apps) and M01, M05, M08, M27 (Android APKs on KIEMS kit):

- Root/jailbreak detection via SafetyNet / Play Integrity API (Android) + custom TEE attestation
- Certificate pinning: hard-coded expected certificate fingerprint; refuse connection if pin fails
- Code obfuscation: R8 / ProGuard for Android; Hermes bytecode for React Native
- RASP (Runtime Application Self-Protection): integrity check on app binary at launch
- Screen capture prevention on sensitive screens (biometric display, form image, results)
- Debug build prevention: release builds verified by CI to have debug flags disabled; debug keys revoked post-build
- App signing via HSM (K1): code-signing key lives in HSM; Shamir 3-of-5 board-approval for any release

### 6.4 API Security

- Input validation via Pydantic v2 with `extra='forbid'` on all request models; invalid input rejected with 422, never coerced
- Output encoding: all HTML output context-escaped at framework level (Jinja2 autoescaping); JSON API returns only typed fields
- Rate limiting: per-user + per-IP + per-endpoint tiers enforced at API gateway; election-critical endpoints (RTS submission, BVI authentication) have asymmetric limits (tight write, looser read-only)
- Request signing for kit-originated calls: kit signs every HTTP request body with its ECDSA private key; back-end verifies before processing
- Idempotency keys on every write endpoint (UUID7 idempotency key required); duplicate requests return cached response without re-executing
- CORS: API endpoints allow only declared origins; no wildcard CORS on any endpoint handling personal data

---

## 7. Infrastructure Security

### 7.1 Kubernetes Cluster Hardening

CIS Kubernetes Benchmark Level 2 baseline enforced via automated scan (kube-bench) on every node provisioning and weekly thereafter. Deviations raise P1 ticket. Key controls:

- API server: RBAC enabled, anonymous auth disabled, admission controllers enabled (NodeRestriction, PodSecurity, MutatingAdmissionWebhook, ValidatingAdmissionWebhook)
- etcd: encrypted at rest (AES-GCM provider); TLS on all etcd communication; backup encrypted and stored in HSM-keyed bucket
- Kubelet: read-only port disabled; webhook authentication/authorization; rotate kubelet server certificates
- Pod Security: restricted profile enforced cluster-wide; exceptions require CCB approval and Kyverno policy exclusion with justification in CCB minutes

### 7.2 Kyverno Admission Policies

Policies enforced on every pod admission:
- Image must be signed (Cosign verified against the Commission public key)
- No privileged containers
- No hostPath volume mounts
- No host network / host PID
- Liveness and readiness probes required
- Resource limits required
- Labels (module, version, team) required for observability

### 7.3 Runtime Security

Falco rules: alert on shell spawn in container, unexpected network egress, privilege escalation attempt, file modification in read-only filesystem. Tetragon eBPF policy: enforce allowed system calls per container; block kernel exploits. Alerts route to SOC / SIEM within 30 seconds.

### 7.4 Secrets Management

HashiCorp Vault (BSL) as primary secrets store, sealed to HSM (Vault auto-unseal via Vault PKCS#11 provider). Vault dynamic secrets for all database credentials; each application receives a unique short-lived credential. Kubernetes External Secrets Operator synchronises Vault secrets to Kubernetes Secrets; pods never access Vault directly. Vault audit log is forwarded to SIEM; no Vault log entry should ever be suppressed.

---

## 8. Cloud and Hosting Security

### 8.1 Kenya-Jurisdiction Hosting

All production voter data, biometric templates, RTS results, and signed events reside exclusively on Commission-provided bare-metal infrastructure in Kenya per DPA 2019 s.18 and the sovereignty principle (M28 AI Monitoring Stack mandate). No production data is processed on non-Kenyan infrastructure. Remote support from Datacraft offshore personnel is permitted only via Commission-controlled bastion, per-session the Commission approval, full session recording, no bulk data access, no production write.

### 8.2 Three-Site Topology

| Site | Role | Location |
|---|---|---|
| Primary | Active production | Anniversary Towers, Nairobi |
| Mirror | Hot standby | Bomas of Kenya, Nairobi |
| Escrow | Cold backup + transparency log anchor | the Commission regional facility (county level, undisclosed) |

Primary and Mirror are active-active for RTS ingestion; active-passive for HSM operations. Escrow receives signed snapshots every 4 hours and at every election milestone. Escrow is air-gapped from public internet; data arrives via dedicated encrypted link.

### 8.3 Disaster Recovery

| Module class | RTO | RPO |
|---|---|---|
| Election-critical (M01-M10, M27) | ≤ 30 minutes | ≤ 60 seconds (signed events in Kafka) |
| Results publication (M11, M13, M14, M51) | ≤ 4 hours | ≤ 5 minutes (CDN snapshot) |
| Cycle-critical (M12, M15, M16, M21, M26) | ≤ 4 hours | ≤ 5 minutes |
| Routine operations (all others) | ≤ 24 hours | ≤ 1 hour |

DR exercise: quarterly full failover drill; results documented and reviewed by IAB.

### 8.4 Backup Policy

Continuous: Kafka replication (sync) across primary and mirror. Hourly: PostgreSQL WAL streaming to mirror. Every 4 hours: full snapshot to escrow (encrypted, HSM-keyed). Daily: SBOM and signed build artefacts replicated to escrow. Restore tested monthly on a scratch cluster. Restore test results logged and available to Auditor-General interface (M59).

---

## 9. Third-Party and Supply Chain Security

### 9.1 SBOM per Artefact

Every container image, every mobile APK, every firmware binary, and every reproducibility bundle (M14) ships with an SPDX 2.3 SBOM. SBOM is signed with Cosign and its hash published to the transparency log within 1 hour of build. SBOM includes all direct and transitive dependencies with licence and known CVE status at build time.

### 9.2 Third-Party Risk Assessment

Before onboarding any new vendor or open-source library in a critical path:
1. Security questionnaire covering SDLC, pen-test cadence, CVE response SLA, incident notification
2. SOC 2 Type II report (or ISO 27001 certificate) required for: biometric ABIS vendor (IDEMIA / Innovatrics / Thales), HSM vendor, cloud-adjacent services
3. Contractual right to audit within 30 days notice
4. For ABIS and HSM vendors: independent third-party code review of integration SDK before production integration

### 9.3 PGP-Signed Release Channels

All Datacraft artefact releases (APK, firmware, container image, bundle) are delivered via internal OCI registry with Cosign signature verification. No artefact is accepted into production without a valid Cosign signature traceable to the Commission Root CA or the Datacraft Artefact Signing Key (itself HSM-backed). Release channels for citizen-facing apps (M13, M29) additionally require HSM-backed code-signing certificate (K1).

### 9.4 Reproducible Builds

M14 Reproducibility Bundle enforces that any observer can produce a byte-identical build artefact from a tagged commit + declared input manifest. This is verified in CI: three consecutive builds from the same tag must produce identical hashes. Non-reproducibility in CI is a P1 finding. Enables independent audit of any deployed artefact.

### 9.5 Source-Code Escrow

Per Teaming Agreement Clause 10: all source code is escrowed to a neutral custodian (escrow agent nominated by the Commission) within 24 hours of every tagged release. Escrow holds: source code, build manifests, dependency lock files, SBOM, signed hashes. the Commission has right to activate escrow upon contractual termination or insolvency. Escrow tested annually (the Commission legal team verifies build succeeds from escrow).

---

## 10. Monitoring, Detection, and Response

### 10.1 SIEM

OpenSearch (self-hosted, Kenya-jurisdiction) as SIEM. Sigma rule set (open-source detection catalogue) plus custom the Commission rules authored by Datacraft Security team and reviewed quarterly by IAB. Log sources: all Kubernetes pod logs, Linkerd access logs, Vault audit log, HSM audit log, Keycloak event log, Falco alerts, Tetragon events, WAF events, kit telemetry (M01, M05, M08, M27). Retention: 90 days hot in OpenSearch; 7 years archive in M69.

### 10.2 E-SOC (Electronic Security Operations Centre)

Per Strategy Part XVI §5: a dedicated E-SOC staffed 24×7 during the election window (T-30 through T+14), with 12 analyst workstations, 4K video wall, and isolation from KIEMS production network. During non-election periods: business-hours staffed with automated pager escalation. SOC receives:
- SIEM alerts (all tiers)
- Falco runtime anomaly
- Network IDS/IPS alerts
- Kit health telemetry anomalies (M26)
- HSM operation alerts
- Transparency-log divergence alerts

### 10.3 Seven-Layer Detection Stack (per M26 Anomaly Detection)

| Layer | Detection type | Alert tier |
|---|---|---|
| L1 Network | BGP hijack, DDoS signature, anomalous packet rates | P0/P1 |
| L2 Infrastructure | Node CPU/memory spike, Falco alert, Tetragon block | P1 |
| L3 Application | Rate-limit breach, ASVS control failure, ZAP finding | P1/P2 |
| L4 Business logic | Double-authentication, tally regression, Benford anomaly | P0 |
| L5 Temporal | Transmission time-window violation, kit time-fence breach | P0/P1 |
| L6 Geospatial | Kit outside geofence, result from unregistered station | P0 |
| L7 Cross-domain | Parallel-tally divergence, Merkle-root mismatch, SIEM + kit anomaly correlation | P0 |

### 10.4 Threat Intelligence Integration

Commercial threat-intelligence feed (MISP or equivalent) subscribed; indicators of compromise ingested into SIEM daily. Kenyan-specific intelligence: NCCS (National Computer and Cybercrimes Coordination Committee) incident-sharing agreement. FIRST (Forum for Incident Response and Security Teams) membership for international sharing. Lookalike domain monitoring service: daily scan for eel.example variants; take-down pre-agreement with KE registry.

### 10.5 Incident Response Playbook

Severity classification:

| Tier | Definition | SLA | Escalation |
|---|---|---|---|
| P0 | Active integrity attack, confirmed tally tamper, PKI compromise, active DDoS impacting voting | Immediate (< 5 min) | Incident Commander + the Commission CISO + CEO + IAB Chair |
| P1 | Security breach (data access), service degradation > SLA, crypto finding | < 1 hour | Incident Commander + the Commission CISO |
| P2 | Security finding (no active breach), performance degradation | < 4 hours | On-call team |
| P3 | Low-severity anomaly, single-system minor failure | < 24 hours | Team lead |

War-room activation 30 days pre-election; 24×7 operation through T+14. Named Incident Commander per shift. Post-incident review published (redacted) within 30 days of resolution.

### 10.6 Chain of Custody for Forensic Artefacts

Per M40 Election Court IT Interface: any forensic artefact for court use must have an unbroken chain-of-custody record from creation to court submission. Chain-of-custody log: signed entry (who, what, hash, timestamp) per access. Forensic disk images: bit-for-bit copy + hash verification; original preserved; copy used for analysis. Expert witnesses designated pre-election. Evidence bundle generation: automated from signed events in Merkle chain; no human manipulation of raw events. Court petition response time: ≤ 24 hours from request (per Elections Act §75(1)).

---

## 11. Compliance Mapping

### 11.1 Constitution of Kenya 2010

| Article | Requirement | Mitigation |
|---|---|---|
| Art. 10 | Public participation, transparency | M11, M13, M14, M18, M19 (public transparency portals) |
| Art. 33 | Freedom of expression (exclusions: false propaganda, incitement) | M17 (speech integrity, deepfake detection), M28 (AI monitoring) |
| Art. 38 | Voter rights (register, vote, seek office) | M01-M12 (full voter lifecycle); M25 (special voter workflows) |
| Art. 43 | Social and economic rights (data protection implied) | DPA 2019 compliance; M35, M04 (voter data handling) |
| Art. 81 | Electoral system principles (secret ballot, free and fair) | M61 (secrecy assurance), M26, M27 |
| Art. 86 | Conduct of elections (the Commission mandate) | All modules |
| Art. 87 | Petition right (24-hour evidence delivery) | M40 (court interface), M14, M63 |
| Art. 88 | the Commission independence | Architecture sovereignty: Kenya-only hosting, Datacraft cannot unilaterally modify production |
| Art. 138 | Presidential election procedure | M08, M09, M10, M51 (results chain); M61 (secrecy) |
| Art. 140 | Presidential election dispute | M40 (court IT interface), M69 (archive) |

### 11.2 Elections Act 2011 (as amended 2016)

| Section | Requirement | Module / Control |
|---|---|---|
| §5 | Voter registration mandate | M01, M02, M35 |
| §6A | Registration window enforcement | M01 (kit time-fence), M35 (CVR) |
| §6B | Continuous registration | M35 |
| §24-§27 | Candidate nomination | M12 (CRMS) |
| §39(1C) | Results transmission | M08, M09, M10 |
| §44 | Polling station procedures | M05, M08, M31 |
| §44A | Failure declaration trigger | M26 (anomaly alert), M16 (ECC escalation) |
| §54 | Ballot secrecy | M61, M08, M27, M30 |
| §72 | Secrecy obligation (officers) | M61, insider threat programme |
| §75(1) | Petition filing (7 days) | M40 |
| §83 | the Commission certification of results | M51 |
| §85 | Referendum trigger | M49 |

### 11.3 Elections (Technology) Regulations 2017

Mandatory testing: stress test, load test, penetration test, functional test before every general election. Datacraft compliance: all four tests are embedded in the OAT and Regulatory Certification gates (see ACCEPTANCE-CRITERIA.md §4). Independent certifier nominated by the Commission performs the regulatory certification run.

### 11.4 Kenya Data Protection Act 2019 — Full Mapping

| DPA 2019 Section | Obligation | Datacraft control |
|---|---|---|
| s.17 | Lawful basis required | Per-endpoint lawful-basis table in §11.4.1 |
| s.18 | No transfer outside Kenya without ODPC authorisation | Kenya-only hosting; no offshore data transit |
| s.25 | Data processing principles (lawful, fair, transparent) | Privacy notices, DPIAs, purpose limitation |
| s.26 | Data subject rights (access, rectify, erase, port) | DSAR workflow, retention schedules |
| s.29 | Special-category data (biometric) | Lawful basis s.30(1)(b) only; field-level encryption; access limited to M01-M03, M05-M06 |
| s.30 | Lawful bases | See §11.4.1 |
| s.31 | Processing for research/statistics | Anonymised only; no voter PII; M44, M28 use synthetic/anonymised data |
| s.33 | Children's data | Voters must be 18+; M35 age verification |
| s.34 | Data Processing Agreement | Signed DPA between the Commission and Datacraft before go-live |
| s.36 | Erasure | Right-to-erasure exceptions per §5.8 |
| s.41 | Destruction of data | Cryptographic erasure; M01 kit decommission (M3 hardware requirement) |
| s.43 | Breach notification (72 hours) | Incident response playbook §10.5 |

**§11.4.1 Per-Endpoint Lawful Basis**

| Module | Personal data processed | DPA 2019 lawful basis |
|---|---|---|
| M01 BVR Device | Biometric (fingerprint, iris, face), NID, name, DOB, address | s.30(1)(b) legal obligation (Elections Act §6) |
| M02 BVR Backend | Same as M01 + voter register ID | s.30(1)(b) |
| M03 ABIS | Biometric templates (special-category) | s.30(1)(b); processing strictly for deduplication |
| M04 VPRS | NID, name, phone, email, ward preference (consent-based) | s.30(1)(a) consent + s.30(1)(b) |
| M05 BVI Device | Biometric (1:1 match only); voter name returned | s.30(1)(b) |
| M06 BVI Backend | Biometric match result; voter record | s.30(1)(b) |
| M07 BVV | Name, ward, polling station (read-only by voter) | s.30(1)(e) public task |
| M08 RTS Device | Presiding officer identity + kit telemetry | s.30(1)(b) |
| M09 RTS-RO | Returning officer identity | s.30(1)(b) |
| M10 RTS Backend | Aggregated results + officer audit trail | s.30(1)(b) |
| M11 Public Portal | No PII; aggregated results only | Not applicable (public data) |
| M12 CRMS | Candidate name, party, NID, fee payment | s.30(1)(b) (Elections Act §24) |
| M13 Public Verifier | No PII | Not applicable |
| M14 Reproducibility | No PII | Not applicable |
| M15 IAB Dashboard | IAB member names + access logs | s.30(1)(e) public task |
| M16 ECC | Staff identity + incident records | s.30(1)(b) |
| M17 Speech Integrity | Public utterances + speaker identity (public figures only) | s.30(1)(e) public task |
| M18 Boundary Transparency | Geographic data; no personal data | Not applicable |
| M19 Queue Transparency | Aggregate station counts; no PII | Not applicable |
| M20 Inventory | Kit serial + assigned officer name | s.30(1)(b) |
| M21 RLA | Aggregated results; no PII | Not applicable |
| M22 Campaign Finance | Donor amounts + party identity | s.30(1)(b) (Campaign Financing Act 2013) |
| M23 Grievance | Complainant name + contact | s.30(1)(a) consent + s.30(1)(b) |
| M24 Diaspora Voting | Voter NID + diaspora address | s.30(1)(b) |
| M25 Special Voter | Voter identity + disability/PWD status | s.30(1)(b); special-category where disability disclosed |
| M26 Anomaly Detection | Pseudonymised station metrics | s.30(1)(b) |
| M27 Intent Preservation | Voter identity tag (for integrity only) | s.30(1)(b) |
| M28 AI Monitoring | Public social media data + device telemetry (anonymised) | s.30(1)(e) public task |
| M29 Mobile Citizen Verify | Voter NID (self-lookup) | s.30(1)(a) consent |
| M30 Mobile Party Agent | Agent identity | s.30(1)(b) |
| M31 Mobile Poll Clerk | Clerk identity + station attendance | s.30(1)(b) |
| M32 Mobile Field Support | Engineer identity + kit assignment | s.30(1)(b) |
| M33 Mobile Personnel Dispatch | Staff identity + location | s.30(1)(b) |
| M34 Mobile Observer | Observer identity + accreditation | s.30(1)(b) |
| M35 CVR | Full voter register | s.30(1)(b) |
| M36 Deregistration Appeal | Voter identity + grounds | s.30(1)(b) |
| M37 Voter Education CMS | No PII | Not applicable |
| M38 Training LMS | Poll worker name + training records | s.30(1)(b) |
| M39 Ballot Printing | Ballot serial + station assignment | s.30(1)(b) |
| M40 Court IT Interface | Case-specific voter/officer records (court order required) | s.30(1)(b) + judicial authority |
| M41 Post-Election Review | Aggregated metrics; named officers for accountability | s.30(1)(b) |
| M42 Media Accreditation | Journalist name + employer + credentials | s.30(1)(a) consent |
| M43 Observer Accreditation | Observer name + organisation | s.30(1)(a) consent |
| M44 Turnout Analytics | Station-level counts (no PII) | Not applicable |
| M45 Party Registration | Party officials + registration data | s.30(1)(b) (Political Parties Act 2011) |
| M46 Party Primaries | Candidate name + primary results | s.30(1)(b) |
| M47 Nomination Fee | Candidate name + payment reference | s.30(1)(b) |
| M48 Recall Election | Petition signatories (voter identity) | s.30(1)(b) |
| M49 Referendum | Voter participation record | s.30(1)(b) |
| M50 By-Election | Voter + officer records (same as general) | s.30(1)(b) |
| M51 Result Certification | Returning officer identity | s.30(1)(b) |
| M52 Election Calendar | No PII | Not applicable |
| M53 Voter ID Card | Photo + voter ID + name | s.30(1)(b) |
| M54 Budget Expenditure | Staff names + vendor details | s.30(1)(b) + PPADA 2015 |
| M55 HR Payroll | Temporary staff PII + bank details | s.30(1)(b) |
| M56 County Coordination | Officer identity | s.30(1)(b) |
| M57 International Observer | Observer name + nationality + accreditation | s.30(1)(a) consent |
| M58 Parliamentary Reporting | Aggregate results; named officials | s.30(1)(b) |
| M59 Auditor General | Audit artefacts; named officials | s.30(1)(b) + Access to Information Act |
| M60 Medical First Response | Staff + incident location | s.30(1)(c) vital interests |
| M61 Secrecy Assurance | Pen-test findings; no live voter data | s.30(1)(b) |
| M62 Transmission Legal Basis | Transmission logs; no PII | s.30(1)(b) |
| M63 Ballot Serial Chain | Ballot serial number (no voter linkage) | s.30(1)(b) |
| M64 Voter List Public Inspection | Redacted voter register (as permitted by Elections Act §35) | s.30(1)(b) |
| M65 Diaspora Ballot Logistics | Voter identity + ballot tracking | s.30(1)(b) |
| M66 Election Financial Forensics | Financial records + named parties | s.30(1)(b) + POCAMLA |
| M67 Party Symbol Registry | Party name + symbol | s.30(1)(b) |
| M68 Referendum Explainer | No PII | Not applicable |
| M69 Long-Term Archive | All voter and results data (archival) | s.30(1)(b) + statutory archival obligation |

### 11.5 Additional Statutes

| Statute | Key obligations | Controls |
|---|---|---|
| the Commission Act 2011 | the Commission mandate, governance, independence | Architecture sovereignty (§8.1) |
| Election Offences Act §6-§12 | Bribery, impersonation, unlawful possession of kit | M20 (kit custody), M61 (physical attack testing) |
| Political Parties Act 2011 | Party registration, reporting | M45, M22 |
| Election Campaign Financing Act 2013 | Campaign finance limits and disclosure | M22, M66 |
| PPADA 2015 | Procurement governance | M54, procurement audit trail |
| Computer Misuse and Cybercrimes Act 2018 | Cybercrime reporting, unauthorised access | Incident response §10.5; E-SOC §10.2 |
| National Cohesion and Integration Act 2008 | Hate speech, incitement | M17 (speech integrity monitoring) |
| Kenya Information and Communications Act | ICT licensing, network regulation | Telco integration governance |
| Whistleblower Protection Act | Anonymous reporting protection | Anonymous whistleblower channel (M61, third-party hosted) |
| POCAMLA 2009 | Money laundering in campaign finance | M22, M66 |
| Access to Information Act 2016 | Public access to election data | M11, M13, M59 (auditor interface) |

---

## 12. Security Assurance

### 12.1 External Penetration Testing

Annual: independent pen-test by CREST-accredited firm. Pre-election: additional pen-test 90 days before election day. Scope: full-stack (application, infrastructure, mobile, kit physical access, social engineering). Findings classified P0-P3; P0 and P1 must be resolved before any production deployment in the affected module. P2 resolved within 30 days; P3 tracked in the risk register.

### 12.2 Internal Penetration Testing

Quarterly internal pen-test by Datacraft Security team + the Commission ICT security officer. Scope rotated to cover different attack surfaces each quarter. Results reviewed by IAB.

### 12.3 Continuous Bug Bounty

Per M28 scope and Strategy Part XIX §2 Part D: a continuous bug-bounty programme (managed via HackerOne or equivalent) for the public-facing surfaces (M11, M13, M29). Scope: all authenticated voter-facing features; no live election data accessible; sandbox environment only. Rewards: KES 50,000 to KES 2,000,000 per finding by severity. Programme suspended during the 30-day software freeze (T-30 to T-0) and reinstated at T+14.

### 12.4 Red-Team Engagements

Per Strategy Part XIX §2 Part D: cross-organisation red team engaged 90 days pre-election, mandate to compromise the sandbox election by any technical or social means. Red team has full disclosure of architecture (white-box) for the first phase (days 1-30) and operates blind for the second phase (days 31-60). Findings reviewed by IAB. All P0/P1 findings resolved before production release.

### 12.5 M61 Secrecy Assurance Testing

M61 is a dedicated continuous red-team programme focused exclusively on proving ballot secrecy holds under adversarial conditions. It runs independently of the main pen-test programme, with a separate mandate, separate findings tracker, and separate IAB attestation. M61 findings are classified RESTRICTED; proof-of-concept details are encrypted blobs accessible only to Datacraft Security Lead and the Commission CISO.

### 12.6 SOC 2 Type II and ISO 27001

Targets for Datacraft's delivery organisation:
- ISO 27001:2022 certification: initial certification before T-180; annual surveillance audit; full recertification at T+3 years
- SOC 2 Type II: completed before T-90; covers Security, Availability, Confidentiality trust service criteria

The Commission IEMS platform itself undergoes ISO 27001 certification as a system (not just organisational); certification scope covers all 69 modules.

### 12.7 Datacraft Security Review Board

Per Teaming Agreement §8: a Datacraft Security Review Board (SRB) convenes monthly. Membership: Datacraft CISO, Datacraft Security Lead, the Commission CISO, one IAB representative. Mandate: review all P0/P1 findings, approve waivers, sign off on security architecture changes, approve annual pen-test scope, review red-team findings. SRB minutes are countersigned by the Commission CISO and available to ODPC on request.

---

## 13. Per-Module Security Matrix

One row per module. Columns:
- **TC**: Threat Category (from §1) — primary threat the module defends against or is most exposed to
- **Auth**: Authentication requirement for this module's users / services
- **Crypto**: Cryptography requirements specific to this module
- **DataClass**: Data classification (RESTRICTED / CONFIDENTIAL / OFFICIAL / PUBLIC)
- **DPA basis**: DPA 2019 lawful basis (from §11.4.1)
- **Retention**: Evidentiary retention period
- **Audit-log**: Audit-log requirement
- **MonClass**: Monitoring class (TIER-1 / TIER-2 / TIER-3)
- **IR Tier**: Incident-response tier for failures in this module

| Module | TC | Auth | Crypto | DataClass | DPA basis | Retention | Audit-log | MonClass | IR Tier |
|---|---|---|---|---|---|---|---|---|---|
| M01 BVR Device | T1 T3 T11 | TPM cert + biometric + enrolment-clerk PIN | ECDSA P-384 signing; AES-256-GCM at rest; mTLS kit-to-back-end; cert pinning | RESTRICTED | s.30(1)(b) | 5 yr (biometric templates) | Per-capture 5-tuple; local Merkle-chained; sync on connect | TIER-1 | P0 |
| M02 BVR Backend | T4 T7 T8 | SPIFFE SVID (intra); FIDO2 admin | HSM-countersigned events; envelope encryption for register exports; AES-256-GCM at rest | RESTRICTED | s.30(1)(b) | 6 yr (register); 7 yr (audit) | Every write event HSM-signed; per-lookup audit entry | TIER-1 | P0 |
| M03 ABIS | T4 T7 T8 | SPIFFE SVID; FIDO2 for adjudication workbench admin | Confidential compute (Intel TDX/AMD SEV-SNP); field-level AES-256-GCM for templates; mTLS | RESTRICTED | s.30(1)(b) | 5 yr (templates); 7 yr (match log) | Every 1:N match result logged with inputs hash; every adjudication action | TIER-1 | P0 |
| M04 VPRS | T7 T10 | OIDC via Keycloak; email + phone verification; optional FIDO2 | TLS 1.3; AES-256-GCM at rest; CAPTCHA on public form | CONFIDENTIAL | s.30(1)(a) consent | 5 yr (pre-reg record) | Every submission; every status update | TIER-2 | P1 |
| M05 BVI Device | T1 T3 T5 T11 | TPM cert + biometric 1:1 on-device | ECDSA P-384 signing; AES-256-GCM voter roll; cert pinning; mTLS | RESTRICTED | s.30(1)(b) | 6 yr (auth record) | Per-authentication 5-tuple; HSM-countersigned | TIER-1 | P0 |
| M06 BVI Backend | T4 T7 T8 | SPIFFE SVID; FIDO2 admin | HSM-countersigned events; mTLS; AES-256-GCM at rest | RESTRICTED | s.30(1)(b) | 6 yr (auth log) | Every auth event signed + Merkle-chained; per-lookup audit | TIER-1 | P0 |
| M07 BVV Public Portal | T9 T10 | CAPTCHA; rate-limited public read; no account required | TLS 1.3; read-only; no write API | OFFICIAL | s.30(1)(e) | 6 yr (query log) | IP + timestamp + query hash per lookup; no full NID logged | TIER-2 | P2 |
| M08 RTS Device | T1 T3 T5 T11 | TPM cert + RO biometric; time-fence + geo-fence enforced | ECDSA P-384 signing; AES-256-GCM local store; cert pinning; mTLS; signed SMS fallback | RESTRICTED | s.30(1)(b) | Permanent (Form 34A) | Every form submission HSM-signed; transmission event 5-tuple | TIER-1 | P0 |
| M09 RTS-RO Tablet | T7 T8 T11 | FIDO2 + biometric of RO; SPIFFE SVID for back-end calls | ECDSA P-384 signing; TLS 1.3; AES-256-GCM local | RESTRICTED | s.30(1)(b) | Permanent | Every constituency result signed by RO; HSM-countersigned | TIER-1 | P0 |
| M10 RTS Backend | T4 T6 T7 T8 T9 | SPIFFE SVID; FIDO2 admin; idempotency keys enforced | HSM-countersigned tally events; Merkle-chained; AES-256-GCM; TLA+ verified state machine | RESTRICTED | s.30(1)(b) | Permanent | Every tally aggregation event signed + Merkle root published | TIER-1 | P0 |
| M11 Public Results Portal | T9 T10 | Anonymous public read; CDN-served signed snapshots; no write API | TLS 1.3; Cosign-signed snapshot; SRI on all assets; CSP strict; HSTS | PUBLIC | Not applicable | Permanent (published results) | Access log (anonymised IP); snapshot generation events | TIER-2 | P1 |
| M12 CRMS | T7 T8 | OIDC + FIDO2 for party officials; SPIFFE SVID intra; RBAC by party | TLS 1.3; AES-256-GCM; nomination documents signed with ECDSA | CONFIDENTIAL | s.30(1)(b) | 6 yr | Every nomination action 5-tuple; ballot proof generation logged | TIER-2 | P1 |
| M13 Public Verifier App | T10 | Anonymous public; no account; QR code verification only | TLS 1.3; HSM-signed QR codes; code-signing via HSM (K1) | PUBLIC | Not applicable | N/A | Verification attempt log (no PII); version check events | TIER-2 | P2 |
| M14 Reproducibility Bundle | T2 T12 | FIDO2 admin only for bundle generation; public download | Cosign-signed bundle; SBOM in SPDX; hash published to transparency log | PUBLIC | Not applicable | Permanent | Every bundle generation event; every signing operation | TIER-2 | P1 |
| M15 IAB Dashboard | T8 | FIDO2 mandatory for IAB members; SPIFFE SVID intra; RBAC IAB roles | TLS 1.3; AES-256-GCM; read-only replica of signed events | CONFIDENTIAL | s.30(1)(e) | 7 yr | Every IAB member access; every alert acknowledgement | TIER-1 | P1 |
| M16 Election Command Centre | T8 T9 | FIDO2 + RBAC for ECC staff; SPIFFE SVID intra | TLS 1.3; AES-256-GCM; session recording for privileged actions | CONFIDENTIAL | s.30(1)(b) | 7 yr | Every command / escalation action; every alert; session recordings | TIER-1 | P1 |
| M17 Speech Integrity | T10 | OIDC for the Commission analysts; SPIFFE SVID intra | TLS 1.3; AES-256-GCM; model inference results signed | OFFICIAL | s.30(1)(e) | 6 yr | Every content flagged + analyst disposition | TIER-2 | P2 |
| M18 Boundary Transparency | T6 | OIDC for the Commission admin; public read anonymous | TLS 1.3; no personal data; GeoJSON signed with ECDSA | PUBLIC | Not applicable | Permanent | Every boundary update; every publication event | TIER-3 | P3 |
| M19 Queue Transparency | T6 | Anonymous public read; OIDC for data ingestion | TLS 1.3; aggregate only; no PII | PUBLIC | Not applicable | 7 yr (aggregate records) | Every aggregate push signed | TIER-3 | P3 |
| M20 Inventory Management | T3 T6 | OIDC + MFA for logistics staff; SPIFFE SVID intra | TLS 1.3; AES-256-GCM; RFID tag chain cryptographically signed | OFFICIAL | s.30(1)(b) | 7 yr (asset lifecycle) | Every kit movement; every seal-break event | TIER-2 | P1 |
| M21 RLA Audit Tool | T12 | FIDO2 for auditors; SPIFFE SVID intra | TLS 1.3; AES-256-GCM; audit results signed with ECDSA | OFFICIAL | Not applicable | Permanent | Every sample draw; every tally comparison; every RLA round result | TIER-2 | P1 |
| M22 Campaign Finance Transparency | T7 | OIDC for party finance officers; public read anonymous | TLS 1.3; AES-256-GCM at rest; public reports signed | OFFICIAL | s.30(1)(b) | 7 yr | Every finance submission; every public report publication | TIER-2 | P2 |
| M23 Grievance Channels | T10 | OIDC optional (anonymous submission allowed); CAPTCHA | TLS 1.3; AES-256-GCM; complainant identity encrypted separately from complaint | CONFIDENTIAL | s.30(1)(a) | 6 yr | Every submission; every status update; every resolution | TIER-2 | P2 |
| M24 Diaspora Voting | T5 T7 | OIDC + MFA + identity verification; SPIFFE SVID intra | TLS 1.3; AES-256-GCM; ballot signed; transmission via mTLS | RESTRICTED | s.30(1)(b) | Permanent (diaspora ballots) | Every ballot submission; every auth event | TIER-1 | P1 |
| M25 Special Voter Workflows | T7 | OIDC + MFA + PWD officer auth; SPIFFE SVID intra | TLS 1.3; AES-256-GCM; disability status (special-category) field-encrypted | RESTRICTED | s.30(1)(b) | 6 yr | Every special-workflow event; PWD officer actions | TIER-2 | P1 |
| M26 Anomaly & Fraud Detection | T7 T9 | SPIFFE SVID intra; FIDO2 for SOC analysts | TLS 1.3; AES-256-GCM; model outputs signed; pseudonymised inputs | CONFIDENTIAL | s.30(1)(b) | 7 yr (alerts and dispositions) | Every alert raised + SOC analyst disposition | TIER-1 | P0 |
| M27 Intent Preservation | T3 T11 | TPM cert on kit; SPIFFE SVID back-end | ECDSA P-384 signing; AES-256-GCM; cert pinning; offline-first local queue | RESTRICTED | s.30(1)(b) | Permanent (signed form images) | Every form sealed event; every transmission event | TIER-1 | P0 |
| M28 AI Monitoring Stack | T10 | SPIFFE SVID intra; FIDO2 for security analysts | TLS 1.3; AES-256-GCM; model outputs signed; no PII in model inputs | CONFIDENTIAL | s.30(1)(e) | 7 yr (monitoring records) | Every detection alert; every analyst action; model version log | TIER-2 | P1 |
| M29 Mobile Citizen Verify | T10 | OIDC optional; anonymous QR scan; rate-limited | TLS 1.3; cert pinning; code-signing via HSM; root/jailbreak detect | PUBLIC | s.30(1)(a) consent | 1 yr (anonymised query log) | Anonymised verification attempt log | TIER-2 | P3 |
| M30 Mobile Party Agent | T11 | OIDC + MFA; party-agent role; SPIFFE intra | TLS 1.3; cert pinning; screen capture prevention; no individual voter data | CONFIDENTIAL | s.30(1)(b) | 6 yr | Every agent action; every aggregate view | TIER-2 | P2 |
| M31 Mobile Poll Clerk | T7 | OIDC + MFA; clerk role; kit pairing token | TLS 1.3; cert pinning; RASP; no biometric data | OFFICIAL | s.30(1)(b) | 6 yr | Every clerk action; every kit pairing event | TIER-2 | P2 |
| M32 Mobile Field Support | T6 | OIDC + MFA; field engineer role | TLS 1.3; cert pinning | OFFICIAL | s.30(1)(b) | 3 yr | Every support ticket; every kit action | TIER-3 | P2 |
| M33 Mobile Personnel Dispatch | T6 | OIDC + MFA; dispatch manager role | TLS 1.3; location data encrypted at rest | OFFICIAL | s.30(1)(b) | 3 yr | Every dispatch event; location ping log | TIER-3 | P2 |
| M34 Mobile Observer | T6 | OIDC + accreditation token; observer role | TLS 1.3; cert pinning | OFFICIAL | s.30(1)(b) | 3 yr | Every observation submitted | TIER-3 | P3 |
| M35 CVR | T7 T8 | OIDC + FIDO2 for the Commission register officers; SPIFFE intra | TLS 1.3; AES-256-GCM; voter record updates HSM-signed | RESTRICTED | s.30(1)(b) | 6 yr | Every register change; every access by officer 5-tuple | TIER-2 | P1 |
| M36 Voter Deregistration Appeal | T7 | OIDC for voter + the Commission officer; dual approval for deregistration | TLS 1.3; AES-256-GCM; appeal record signed | CONFIDENTIAL | s.30(1)(b) | 6 yr | Every appeal submission; every decision | TIER-3 | P2 |
| M37 Voter Education CMS | T6 | OIDC for content authors; public anonymous read | TLS 1.3; CSP strict; no PII stored | PUBLIC | Not applicable | 7 yr (content versions) | Every publish action; every content change | TIER-3 | P3 |
| M38 Poll Worker Training LMS | T6 | OIDC for trainees + instructors; MFA for instructors | TLS 1.3; AES-256-GCM; completion records signed | OFFICIAL | s.30(1)(b) | 7 yr (training records) | Every module completion; every assessment result | TIER-3 | P3 |
| M39 Ballot Printing Chain | T3 T12 | OIDC + FIDO2 for print officers; printer API cert | TLS 1.3; AES-256-GCM; ballot spec signed; serial chain published to M63 | CONFIDENTIAL | s.30(1)(b) | Permanent (serial chain) | Every ballot proof generated; every serial number issued | TIER-2 | P1 |
| M40 Election Court IT Interface | T12 | FIDO2 + court order verification; dual-custody for evidence bundle | TLS 1.3; evidence bundle signed with ECDSA; chain-of-custody record | RESTRICTED | s.30(1)(b) + judicial | Permanent | Every evidence request; every bundle generated; every court submission | TIER-1 | P0 |
| M41 Post-Election Review | T8 | FIDO2 for review officers; read-only access to signed events | TLS 1.3; read-only replica; no write to live data | OFFICIAL | s.30(1)(b) | 7 yr | Every review session; every finding recorded | TIER-3 | P3 |
| M42 Media Accreditation | T10 | OIDC; identity verification by the Commission press office | TLS 1.3; AES-256-GCM; accreditation badge signed | OFFICIAL | s.30(1)(a) | 3 yr | Every application; every badge issued/revoked | TIER-3 | P3 |
| M43 Observer Accreditation | T10 | OIDC + MFA; organisation verification | TLS 1.3; AES-256-GCM; badge signed | OFFICIAL | s.30(1)(a) | 3 yr | Every application; every accreditation decision | TIER-3 | P3 |
| M44 Turnout Forecasting Analytics | T7 | SPIFFE SVID intra; OIDC for analysts | TLS 1.3; pseudonymised station metrics only | OFFICIAL | Not applicable | 7 yr (model outputs) | Every model run; every forecast published | TIER-3 | P3 |
| M45 Party Registration | T7 | OIDC + MFA for party officials + the Commission registrar | TLS 1.3; AES-256-GCM; registration documents signed | OFFICIAL | s.30(1)(b) | 7 yr | Every registration event; every status change | TIER-3 | P2 |
| M46 Party Primaries Oversight | T7 | OIDC + MFA for party officials + the Commission observers | TLS 1.3; AES-256-GCM; results signed | OFFICIAL | s.30(1)(b) | 6 yr | Every result submission; every certification event | TIER-2 | P2 |
| M47 Nomination Fee Payment | T7 | OIDC + MFA; payment gateway mTLS; PCI DSS scope | TLS 1.3; PCI DSS compliance for card processing; payment token not stored | CONFIDENTIAL | s.30(1)(b) | 7 yr (financial records) | Every payment attempt; every receipt issued | TIER-2 | P2 |
| M48 Recall Election Workflow | T7 | OIDC + MFA for petition coordinators + the Commission | TLS 1.3; AES-256-GCM; petition signatures signed | CONFIDENTIAL | s.30(1)(b) | 6 yr | Every petition event; every signature submitted | TIER-2 | P2 |
| M49 Referendum Mechanics | T7 T11 | Same as general election modules (M05, M08, M10 reused) | Same as M05, M08, M10 | RESTRICTED | s.30(1)(b) | Permanent | Same as M08, M10 | TIER-1 | P0 |
| M50 By-Election Operations | T7 | Same as general election | Same as election modules | RESTRICTED | s.30(1)(b) | Permanent | Same as election modules | TIER-1 | P0 |
| M51 Result Declaration Certification | T8 T12 | FIDO2 + RO biometric; dual-custody certification | ECDSA P-384 signing; HSM-countersigned certificate; Merkle-anchored | RESTRICTED | s.30(1)(b) | Permanent | Every certification event; every signature | TIER-1 | P0 |
| M52 Election Calendar & Gazette | T6 | OIDC for the Commission admin; public anonymous read | TLS 1.3; calendar events signed; no PII | PUBLIC | Not applicable | Permanent | Every calendar event published | TIER-3 | P3 |
| M53 Voter ID Card | T3 T7 | OIDC + FIDO2 for card issuance officers | TLS 1.3; AES-256-GCM; card data signed; photo biometric handled per M01-M03 rules | RESTRICTED | s.30(1)(b) | 6 yr | Every card issuance / revocation event | TIER-2 | P1 |
| M54 Election Budget Expenditure | T8 | OIDC + MFA; finance officer role; dual approval for large transactions | TLS 1.3; AES-256-GCM; financial records signed | CONFIDENTIAL | s.30(1)(b) | 7 yr | Every expenditure entry; every approval | TIER-3 | P2 |
| M55 HR Payroll Temporary Staff | T8 | OIDC + MFA; HR manager role; payroll system mTLS | TLS 1.3; AES-256-GCM; bank details field-encrypted | CONFIDENTIAL | s.30(1)(b) | 7 yr | Every payroll event; every bank detail change | TIER-3 | P2 |
| M56 County-National Coordination | T6 | OIDC + MFA; county RO + national roles | TLS 1.3; AES-256-GCM | OFFICIAL | s.30(1)(b) | 6 yr | Every coordination message; every escalation | TIER-2 | P2 |
| M57 International Observer Coordination | T10 | OIDC; accreditation token; MFA for sensitive briefings | TLS 1.3; AES-256-GCM; briefing materials signed | OFFICIAL | s.30(1)(a) | 3 yr | Every briefing access; every accreditation event | TIER-3 | P3 |
| M58 Parliamentary & Judiciary Reporting | T8 | FIDO2 for the Commission officials; read-only to signed events | TLS 1.3; report signed with ECDSA before transmission | OFFICIAL | s.30(1)(b) | 7 yr | Every report generated; every submission | TIER-3 | P2 |
| M59 Auditor General Interface | T8 | FIDO2 + AG office certificate; read-only to archive | TLS 1.3; mTLS to archive; evidence package signed | OFFICIAL | s.30(1)(b) + Access to Information Act | Permanent | Every evidence package delivered; every access | TIER-2 | P1 |
| M60 Medical First Response | T6 | OIDC + MFA; medical coordinator role | TLS 1.3; AES-256-GCM; incident location minimised to station level | OFFICIAL | s.30(1)(c) vital interests | 7 yr | Every incident report; every dispatch | TIER-3 | P2 |
| M61 Secrecy Assurance Testing | T11 | FIDO2 + CISO credentials; PoC blobs double-encrypted | TLS 1.3; finding PoCs encrypted (AES-256-GCM, dual-key); no live voter data | RESTRICTED | s.30(1)(b) | 7 yr | Every engagement; every finding; every remediation action | TIER-2 | P1 |
| M62 Transmission Legal Basis Audit | T12 | OIDC + MFA; legal compliance officer | TLS 1.3; audit records signed; no PII in transmission logs | OFFICIAL | s.30(1)(b) | Permanent | Every legal basis assertion; every audit event | TIER-2 | P1 |
| M63 Ballot Serial Chain | T12 | OIDC + FIDO2 for print officers; public read anonymous | TLS 1.3; serial records Merkle-chained; signed with ECDSA | PUBLIC | s.30(1)(b) | Permanent | Every serial issued; every chain root published | TIER-2 | P1 |
| M64 Voter List Public Inspection | T7 | Anonymous public read (redacted list only); CAPTCHA; rate-limited | TLS 1.3; list signed; NID and biometric not exposed | PUBLIC | s.30(1)(b) | 6 yr (inspection records) | Every list published; IP + query hash log | TIER-3 | P2 |
| M65 Diaspora Ballot Physical Logistics | T3 | OIDC + MFA; logistics manager role | TLS 1.3; AES-256-GCM; ballot serial tracking | CONFIDENTIAL | s.30(1)(b) | Permanent | Every ballot dispatch / return event | TIER-2 | P1 |
| M66 Election Financial Forensics | T8 | FIDO2 + forensics officer role; dual-custody for case access | TLS 1.3; AES-256-GCM; forensic case files signed; chain-of-custody | CONFIDENTIAL | s.30(1)(b) + POCAMLA | 7 yr | Every case action; every file access | TIER-2 | P1 |
| M67 Party Symbol & Colour Registry | T6 | OIDC for the Commission registrar; public anonymous read | TLS 1.3; registry entries signed | PUBLIC | s.30(1)(b) | Permanent | Every registry update | TIER-3 | P3 |
| M68 Referendum Explainer Platform | T10 | Anonymous public read; OIDC for the Commission content authors | TLS 1.3; CSP strict; WCAG 2.2 AA (AAA target per module spec); no PII | PUBLIC | Not applicable | 7 yr (content versions) | Every content publish; every significant user interaction | TIER-3 | P3 |
| M69 Long-Term Archive | T4 T12 | FIDO2 + dual-custody for write; AG certificate for authorised read | TLS 1.3 + mTLS; AES-256-GCM + envelope encryption; SLH-DSA re-signing annually; WORM storage | RESTRICTED | s.30(1)(b) + statutory | Permanent | Every archive write; every access; every re-signing event | TIER-2 | P1 |

---

## 14. Security Requirements Reference (SR-* Catalogue)

This section lists all SR-* requirements used as evidence criteria throughout this document and in the per-module matrix. Each SR-* is a testable control; its evidence type is how it is verified at acceptance.

### 14.1 Identity, Authentication, Authorisation (SR-IAM)

| Req | Statement | Evidence type |
|---|---|---|
| SR-IAM-001 | Central IAM (Keycloak 26) enforces RBAC + ABAC for every user, service, and device principal | Policy test suite + OPA rule coverage report |
| SR-IAM-002 | MFA mandatory on every non-anonymous entrypoint; FIDO2 hardware key required for admin; SMS MFA prohibited on admin accounts | Config audit + penetration test (MFA bypass attempt) |
| SR-IAM-003 | Password policy: min 8 chars, upper+lower+digit+special, no dictionary words, 10-cycle history, screensaver lockout | Keycloak config export + automated policy test |
| SR-IAM-004 | Session tokens: ≤ 15 min interactive; ≤ 5 min service-to-service; refresh rotation on every use | Token inspection + expiry test |
| SR-IAM-005 | JIT privileged access; no standing admin rights on production; elevation requires ticketed change window + dual approval | Vault policy audit + access review log |
| SR-IAM-006 | Separation of duties enforced by OPA/Cedar: no single account can simultaneously read biometric templates, modify registers, and access audit logs | Policy test suite; red-team insider simulation |
| SR-IAM-007 | Break-glass accounts dormant; activation requires court-attested order; use generates transparency-log entry | Drill log + transparency-log entry verification |
| SR-IAM-008 | Every principal has a unique attributable identity; no shared credentials | Access inventory audit; no shared account found |

### 14.2 PKI (SR-PKI)

| Req | Statement | Evidence type |
|---|---|---|
| SR-PKI-001 | Root CA offline, air-gapped, ceremony-signed with M-of-N (3-of-5) the Commission + Datacraft dual-witnessed quorum | Ceremony video + signed ceremony log |
| SR-PKI-002 | Two-level intermediate CAs: one kit, one server; blast radii separated | CA hierarchy diagram + cert inspection |
| SR-PKI-003 | Every KIEMS kit has unique digital certificate bound to TPM/secure element, issued at kit-prep | Certificate inventory + TPM attestation report |
| SR-PKI-004 | Certificate revocation propagates ≤ 60 seconds via CRL + OCSP-must-staple | Revocation timing test (kit→back-end refuse) |
| SR-PKI-005 | All kit boot attestations written to public transparency log | Transparency log query by observers |
| SR-PKI-006 | CT log divergence raises P0 alert | Alert rule test + SIEM trigger verification |

### 14.3 HSM and Key Management (SR-HSM)

| Req | Statement | Evidence type |
|---|---|---|
| SR-HSM-001 | HSM cluster is FIPS 140-3 Level 3+; tier-1 vendor (Thales Luna 7 / Utimaco SecurityServer / Entrust nShield) | Vendor FIPS certificate + purchase record |
| SR-HSM-002 | Keys generated in-HSM; never exported in clear form | HSM audit log + key inventory (no export flag) |
| SR-HSM-003 | Dual-control quorum for CA operations | CA operation log showing two-custodian tokens |
| SR-HSM-004 | HSM cluster capacity: 3× peak signing rate; queue overflow pages immediately | Load test at 3× peak; pager alert firing |
| SR-HSM-005 | Every HSM administrative action recorded on tamper-evident video with two-person present | Video archive index; HSM audit log correlation |
| SR-HSM-006 | Post-quantum readiness: ML-KEM-768 + X25519 hybrid KEM; ML-DSA-65 + ECDSA P-384 hybrid signing | Algorithm negotiation trace; library version audit |

### 14.4 Cryptography (SR-CRYP)

| Req | Statement | Evidence type |
|---|---|---|
| SR-CRYP-001 | TLS 1.3 mandatory; no cipher suite weaker than AES-256-GCM; no TLS 1.2 fallback | TLS scan (testssl.sh); cipher negotiation test |
| SR-CRYP-002 | AES-256-GCM at rest on every persistent store | DB encryption inspection (keys withheld); LUKS audit |
| SR-CRYP-003 | Perfect forward secrecy on every session | TLS session inspection; no static RSA key exchange |
| SR-CRYP-004 | Field-level encryption for biometric templates with separate keys from index rows | Schema inspection + DEK separation verification |
| SR-CRYP-005 | Envelope encryption for register exports | Export pipeline code review; encrypted export verification |
| SR-CRYP-006 | RFC 3161 timestamping backed by HSM for every signed event | Timestamp token inspection; TSA certificate chain |
| SR-CRYP-007 | Anti-rollback: monotonic counter + fresh nonce; back-end rejects regressed counter | Replay attack test; counter-regression rejection confirmed |
| SR-CRYP-008 | No custom crypto implementations; audited libraries only (BoringSSL, libsodium, ring, HSM-provided) | SBOM + code review; no custom crypto patterns in SAST |

### 14.5 Kit-to-Back-End Network (SR-NET)

| Req | Statement | Evidence type |
|---|---|---|
| SR-NET-001 | mTLS mandatory kit-to-back-end; both certificates from the Commission internal CA | Traffic capture (TLS inspection); cert chain verification |
| SR-NET-002 | Certificate pinning on kits; hard-coded expected fingerprints | Pin bypass test; kit refuses non-matching chain |
| SR-NET-003 | VPN encapsulation over telco carrier data channels; second auth layer on top of TLS | VPN config audit; traffic capture shows VPN layer |
| SR-NET-004 | Multi-carrier redundancy: dual-SIM + eSIM (Safaricom / Airtel / Telkom); automatic failover | Carrier flap test; failover time ≤ 30 s |
| SR-NET-005 | No 2G fallback; kits configured LTE/5G only; 3G as constrained fallback with explicit warning | Kit radio config audit; 2G block verified |
| SR-NET-006 | DNSSEC + hardcoded fallback endpoints in kit config | DNS inspection; fallback endpoint test |
| SR-NET-007 | BGP monitoring via RPKI with alerting on prefix hijack | BGPmon subscription confirmation; alert test |
| SR-NET-008 | Signed SMS/USSD fallback for RTS transmission when data unavailable | SMS fallback field trial; signed message verification |
| SR-NET-009 | LoRa mesh option for high-risk stations | LoRa config documentation; field test at 3 high-risk stations |
| SR-NET-010 | Anti-jam GNSS: multi-constellation (GPS + Galileo + BeiDou) + IMU cross-check | GNSS spoof test; IMU cross-check alert firing |

### 14.6 Data-Centre Perimeter and Internal (SR-PERIM)

| Req | Statement | Evidence type |
|---|---|---|
| SR-PERIM-001 | Zero-trust; mTLS + short-lived tokens on every service-to-service call | Linkerd mesh report; SPIFFE SVID audit |
| SR-PERIM-002 | Micro-segmentation at pod/VM level; blast-radius contained per service | Kubernetes network policy audit; lateral movement test |
| SR-PERIM-003 | Bastion-only administrative access; hardware-token MFA; session recording; JIT elevation; full audit | Bastion access log; session recording archive; no direct prod access |
| SR-PERIM-004 | Egress control: back-end services reach only declared external endpoints | Network policy audit; undeclared egress blocked and alerted |
| SR-PERIM-005 | Immutable infrastructure: servers from signed images; no in-place patching | Deployment log; GitOps diff; no SSH to prod nodes |
| SR-PERIM-006 | Physical security per the Commission DC baseline: biometric entry, mantraps, 24×7 guard, environmental sensors, 72-hr UPS, Novec suppression | the Commission DC audit report; Datacraft on-site inspection record |
| SR-PERIM-007 | Upstream DDoS scrubbing + on-prem WAF + L7 challenge for anonymous public traffic | DDoS simulation; WAF log; challenge trigger test |
| SR-PERIM-008 | Anycast + per-IP rate limiting at ingress | Ingress config audit; rate-limit trigger test |

### 14.7 Application Security (SR-APP)

| Req | Statement | Evidence type |
|---|---|---|
| SR-APP-001 | OWASP ASVS Level 2 compliance across all applications (Level 3 for election-critical) | ASVS assessment report by independent auditor |
| SR-APP-002 | Parameterised queries mandatory; dynamic SQL construction rejected in code review | SAST (Semgrep SQL injection rules); PR code review log |
| SR-APP-003 | Framework-level authn/authz + per-request OPA/Cedar policy check | Policy test suite; per-request policy trace |
| SR-APP-004 | Idempotency keys on every write endpoint; prevents double-vote and double-count | Duplicate request test; idempotency key enforcement confirmed |
| SR-APP-005 | Optimistic concurrency control on register writes with version stamps | Concurrent write test; version-conflict detection |
| SR-APP-006 | Fuzzing of MRZ, WSQ, JPEG-2000, ICR/OCR pipelines | Fuzzing report (1-hour runs, no crashes) |
| SR-APP-007 | ICR model output below confidence threshold routed to human review; never silent auto-tally | ICR confidence test; low-confidence sample manually reviewed |
| SR-APP-008 | Property-based tests on tally aggregation (sums always non-negative, monotonic, grouped) | Hypothesis/proptest report; 10,000 draws, all pass |
| SR-APP-009 | TLA+ formal specification for RTS state machine; no double-counting, no lost writes | TLC model checker run; zero violations |
| SR-APP-010 | CSP strict on every web surface | CSP header inspection; no unsafe-inline/eval |
| SR-APP-011 | Session hardening: HttpOnly + Secure + SameSite cookies; MFA re-check on high-risk operations | Cookie inspection; MFA re-challenge test |
| SR-APP-012 | Rate limiting per user/IP/endpoint at every public boundary | Rate-limit test per endpoint class |

### 14.8 Data at Rest (SR-DATA)

| Req | Statement | Evidence type |
|---|---|---|
| SR-DATA-001 | AES-256-GCM encryption at rest on every persistent store | DB inspection with keys withheld; unreadable without HSM |
| SR-DATA-002 | Field-level encryption for biometric templates with separate keys | Schema inspection; DEK rotation test |
| SR-DATA-003 | Confidential compute enclaves (Intel TDX / AMD SEV-SNP) for ABIS matching | Enclave attestation report; template-in-clear-memory test fails |
| SR-DATA-004 | No production data in non-production environments | Env audit; synthetic data verification in staging |
| SR-DATA-005 | Bulk exports (> N rows) require dual-control approval + audit entry | Bulk export test; single-approver attempt blocked |
| SR-DATA-006 | Register access logged per-lookup; audit trail available for ODPC | Audit log query; per-lookup entry confirmed |
| SR-DATA-007 | Post-quantum migration path for long-lived confidential data | Re-signing schedule documented in M69; annual SLH-DSA re-sign tested |

### 14.9 Audit and Tamper Evidence (SR-AUDIT)

| Req | Statement | Evidence type |
|---|---|---|
| SR-AUDIT-001 | Every user action captured with 5-tuple: (who, what, which entity, from where, when) | Audit log schema inspection; completeness test |
| SR-AUDIT-002 | Every state-change event signed by origin, countersigned by back-end HSM, RFC 3161 timestamped | Event inspection; signature chain verification |
| SR-AUDIT-003 | Events hash-chained into per-constituency Merkle logs | Merkle tree verification; inclusion proof test |
| SR-AUDIT-004 | Merkle log roots published to national transparency-log service at regular intervals | Transparency log public query; cadence verified |
| SR-AUDIT-005 | Independent escrow custodian receives read-only mirror | Escrow custody sign-off; mirror sync verification |
| SR-AUDIT-006 | 90-day online retention + permanent immutable archive (M69) | Retention policy audit; archive integrity test |
| SR-AUDIT-007 | Any observer can retrieve a Merkle-inclusion proof for any published event within 30 s | Observer retrieval timing test; 30-s SLA met |

### 14.10 Availability and DDoS (SR-AVAIL)

| Req | Statement | Evidence type |
|---|---|---|
| SR-AVAIL-001 | Multi-tier DDoS defence: carrier scrub + anycast + WAF + L7 challenge | DDoS simulation; origin survives full CDN bypass |
| SR-AVAIL-002 | Public Results Portal static-first: signed snapshots on CDN; origin serves update generator only | Architecture inspection; origin traffic < 1% under peak |
| SR-AVAIL-003 | RTS ingress horizontally scaled with sharded queues by station; no single hot path | Load test; shard distribution analysis |
| SR-AVAIL-004 | Back-pressure everywhere: refuse-fast when overloaded; signed rejection from kit | Overload test; back-pressure signal verified |
| SR-AVAIL-005 | Warm capacity headroom 25%; auto-scale to 200% peak | Auto-scale test; 200% load sustained |
| SR-AVAIL-006 | Circuit breakers on external dependencies; fail-open on non-integrity-critical reads | Circuit breaker test; fail-open / fail-closed behaviour confirmed |
| SR-AVAIL-007 | Kill-switches to shed non-critical load during peak | Kill-switch test; analytics shed while RTS continues |

### 14.11 Data Protection and Privacy (SR-PRIV)

| Req | Statement | Evidence type |
|---|---|---|
| SR-PRIV-001 | Kenya DPA 2019 compliance; ODPC registration at go-live | ODPC registration certificate |
| SR-PRIV-002 | DPIA per subsystem, reviewed annually | DPIA documents on file; ODPC acknowledgement |
| SR-PRIV-003 | Purpose limitation: voter records used only for the Commission statutory mandate | Data flow diagram; no undeclared data sharing |
| SR-PRIV-004 | Consent management for VPRS pre-registration | Consent log; withdrawal test |
| SR-PRIV-005 | DSAR workflow with 30-day SLA | DSAR test; response within 30 days confirmed |
| SR-PRIV-006 | Retention scheduler enforces statutory limits automatically | Scheduler test; records deleted on schedule |
| SR-PRIV-007 | Data-breach notification workflow to ODPC within 72 hours | Breach drill; notification sent within 72 h confirmed |
| SR-PRIV-008 | Media sanitisation before repair/replacement/disposal | Erasure certificate per kit decommission; NIST 800-88 compliance |

### 14.12 Physical Device Security (SR-DEV)

| Req | Statement | Evidence type |
|---|---|---|
| SR-DEV-001 | Per-kit digital certificate bound to TPM/secure element | Certificate inventory; TPM binding verification |
| SR-DEV-002 | Sub-minute central revocation | Revocation timing test ≤ 60 s |
| SR-DEV-003 | Tamper-evident chassis (epoxy-potted secure element, kill-on-open switches) | Physical inspection; kill-on-open test |
| SR-DEV-004 | Geo-fence enforcement on device and back-end | GPS spoof test; kit refuses outside geofence |
| SR-DEV-005 | Time-fence enforcement: kit refuses operation outside gazetted polling window | Time-fence bypass test; refusal confirmed |
| SR-DEV-006 | Biometric-under-duress signal: designated finger triggers covert distress signal | Duress finger test; back-end receives covert alert |
| SR-DEV-007 | Two-person integrity for kit reset or session recovery | Kit reset test; single-person attempt blocked |
| SR-DEV-008 | QR-verifiable tamper seals with cryptographic chain | Seal tamper test; QR verification fails on tampered seal |
| SR-DEV-009 | JTAG/UART pads unpopulated on production units; fuses set to disable debug | Hardware inspection; debug port probe fails |
| SR-DEV-010 | Stolen-kit honey mode: revoked kit receives plausible sandbox; captures operator biometrics + GPS + camera | Honey-mode test; forensic capture confirmed |

### 14.13 Supply Chain Software (SR-SUPPLY)

| Req | Statement | Evidence type |
|---|---|---|
| SR-SUPPLY-001 | Internal package mirror; no direct public registry pulls in build | CI network log; no external registry traffic in hermetic build |
| SR-SUPPLY-002 | Reproducible builds with pinned hashes | M14 reproducibility test; three identical builds |
| SR-SUPPLY-003 | SBOM per build in SPDX format, signed, stored immutably | SBOM inventory; Cosign signature verification |
| SR-SUPPLY-004 | Signed commits mandatory (GPG or Sigstore) | Git log audit; no unsigned commits on protected branches |
| SR-SUPPLY-005 | Pushes to protected branches require two-person review | GitHub branch protection config; PR audit log |
| SR-SUPPLY-006 | Hermetic CI/CD builds; no network egress except internal mirror; secrets via short-lived tokens; every artefact signed at exit | CI network log; artefact signature verification |
| SR-SUPPLY-007 | Container images minimal (distroless) and signed with Cosign; admission control refuses unsigned | Image manifest inspection; Kyverno policy test |
| SR-SUPPLY-008 | SAST + DAST + secret scanning on every PR; high-severity findings block merge | CI gate configuration; finding-block test |
| SR-SUPPLY-009 | Dependency freshness reviewed monthly; SBOM published quarterly | Renovate PR log; SBOM publication timestamps |
| SR-SUPPLY-010 | Independent third-party code review on RTS, ABIS, and PKI modules before production release | Third-party review report; finding log |

### 14.14 Insider Threat (SR-INSIDER)

| Req | Statement | Evidence type |
|---|---|---|
| SR-INSIDER-001 | Full session recording on privileged sessions; recordings encrypted with dual custody | Recording archive; dual-custody key inventory |
| SR-INSIDER-002 | Random second-line audit of sample of each action type daily | Audit sampling log; finding rate report |
| SR-INSIDER-003 | Anonymous whistleblower channel, third-party hosted | Channel URL; third-party SLA |
| SR-INSIDER-004 | Cross-organisation red team engaged 90 days pre-election | Red-team engagement report; finding disposition |
| SR-INSIDER-005 | Public Results Portal is read replica derived from signed events; no operator can edit portal without signed source event | Architecture inspection; edit-without-event-attempt blocked |
| SR-INSIDER-006 | Independent parallel-tally service replays signed events; drift raises P0 alert | Parallel-tally divergence test; P0 alert fires within 30 s |

### 14.15 Social Engineering Resistance (SR-SE)

| Req | Statement | Evidence type |
|---|---|---|
| SR-SE-001 | FIDO2 hardware keys mandatory for privileged users; SMS MFA prohibited on admin | Config audit; SMS MFA bypass attempt fails |
| SR-SE-002 | Anti-phish training + simulated-phish programme quarterly with metrics | Training completion records; phish campaign results |
| SR-SE-003 | the Commission press releases signed with key on multiple canonical domains | Key publication on eel.example and escrow |
| SR-SE-004 | DMARC/DKIM/SPF strict-enforce on eel.example | Email header inspection; DMARC reject policy |
| SR-SE-005 | Poll clerks have no back-end credentials to steal; kits authenticate via certificate + biometric | Credential inventory; no clerk back-end credentials found |
| SR-SE-006 | In-band-only support authentication: helpdesk asks for challenge only the clerk can produce from the kit | Helpdesk procedure audit; social-engineering drill |
| SR-SE-007 | Look-alike-domain hunting service subscribed; take-down pre-agreed with registrars | Service subscription; take-down playbook |

### 14.16 Testing and Certification (SR-TEST)

| Req | Statement | Evidence type |
|---|---|---|
| SR-TEST-001 | Elections (Technology) Regulations 2017 certification: stress + load + penetration + functional | Regulatory certification report |
| SR-TEST-002 | Independent pen-test annually + 90 days pre-election | Pen-test reports; finding disposition |
| SR-TEST-003 | Biometric accuracy re-certification annually by independent lab | Lab report; FAR/FRR/throughput results |
| SR-TEST-004 | ISO 27001:2022 surveillance audit annually | Audit letter or certificate |
| SR-TEST-005 | DPIA review annually | DPIA revision log; ODPC acknowledgement |
| SR-TEST-006 | Sandbox drill by the Commission operations quarterly | Drill log; the Commission sign-off |
| SR-TEST-007 | Full election-day dress rehearsal 60 days pre-election | Dress rehearsal report; IAB observer sign-off |
| SR-TEST-008 | Chaos engineering weekly | Chaos Mesh experiment log; SLO measurement |
| SR-TEST-009 | Failover/DR exercise quarterly | DR exercise report; RTO/RPO metrics |
| SR-TEST-010 | Purple-team quarterly with red-team | Purple-team report; detection rate improvement |

### 14.17 Incident Response (SR-IR)

| Req | Statement | Evidence type |
|---|---|---|
| SR-IR-001 | Incident-response plan with severity classification, escalation, forensic preservation, the Commission notification | IR plan document; annual review |
| SR-IR-002 | War-room activation 30 days pre-election; 24×7 during polling window | War-room roster; activation record |
| SR-IR-003 | Named Incident Commander per shift | Shift schedule; trained IC list |
| SR-IR-004 | Communication rules of engagement pre-agreed with the Commission comms | Communication plan; sign-off by the Commission comms director |
| SR-IR-005 | Court-order response playbook pre-approved by the Commission legal counsel | Playbook; legal counsel sign-off |
| SR-IR-006 | Post-incident review with public disclosure (redacted) within 30 days | Review report; publication record |

### 14.18 Data Residency (SR-RES)

| Req | Statement | Evidence type |
|---|---|---|
| SR-RES-001 | Production voter and results data reside in Kenya on Commission-provided infrastructure | Infrastructure audit; data-flow diagram |
| SR-RES-002 | Backups stored in Kenya; off-site backup encrypted at mirror DC | Backup config; mirror DC location confirmation |
| SR-RES-003 | Escrow mirror in Kenya at designated independent custodian | Escrow agreement; mirror sync verification |
| SR-RES-004 | Offshore remote support only via Commission-controlled bastion; per-session the Commission approval; full recording; no bulk data access; no production write | Bastion access log; session recordings; no prod write from offshore |

### 14.19 Governance and Transparency (SR-GOV)

| Req | Statement | Evidence type |
|---|---|---|
| SR-GOV-001 | Independent Technical Advisory Committee with academic + civil-society + international observer members | Committee charter; member list; clearance record |
| SR-GOV-002 | Public system-security specification (adapted for publication) | Published document URL |
| SR-GOV-003 | Post-election audit by Office of the Auditor-General + rotating international observer | AG engagement letter; observer mandate |
| SR-GOV-004 | Real-time public dashboard on transparency log showing every published root, quorum operation, high-severity incident | Dashboard URL; public accessibility test |

---

### 14.20 Mobile Application Security (SR-MOB)

Applies to M29-M34 (Flutter-based citizen, party-agent, poll-clerk, field-support, personnel-dispatch, and observer apps). Full requirements in `sw/mobile/SECURITY.md`; the SR-MOB namespace provides the programme-level control identifiers.

| Req | Statement | Evidence type |
|---|---|---|
| SR-MOB-001 | All mobile apps submitted to Play Store under the Commission developer account; APKs also side-loadable via MDM for staff apps | Play Store listing; MDM enrolment record |
| SR-MOB-002 | Staff apps (M30-M34) require FIDO2 device-bound credential + Play Integrity API attestation on every session start; citizen app (M29) requires OTP tied to national ID + registered mobile number | Penetration test report; attestation server audit log |
| SR-MOB-003 | Encrypted local storage: SQLCipher AES-256 for offline queue; keys derived from device-bound secret + user PIN (not solely device IMEI) | Static analysis report; key derivation spec |
| SR-MOB-004 | Certificate pinning for all backend connections; mTLS for M30-M34 staff apps using MDM-provisioned client cert; JWT+OTP for M29 citizen app | TLS audit; MDM provisioning log |
| SR-MOB-005 | Offline operation: signed payloads queued locally with sequence number; replay-protected on sync; duplicate-detection at backend | Integration test results; sequence-gap audit |
| SR-MOB-006 | GPS spoofing detection: compare reported GPS against triangulated cell/WiFi position; flag discrepancy > 1 km as anomaly | Red-team GPS spoofing test |
| SR-MOB-007 | Biometric data never stored on device beyond the active session; cleared on app backgrounding | Static analysis; OWASP Mobile L2 report |
| SR-MOB-008 | All mobile apps pass OWASP Mobile Application Security Verification Standard (MASVS) Level 2 | Independent MASVS audit report |
| SR-MOB-009 | Remote wipe command from M33 MDM layer erases all the Commission data within 60 s of command receipt; verified in FAT | MDM wipe test log |
| SR-MOB-010 | `libcommission-verifier` Rust FFI library (shared between M13 and M29) is subject to same code-review and penetration-test cadence as backend services | Library audit report; dependency diff log |

**Cross-reference:** `sw/mobile/SECURITY.md` (full mobile security architecture); MOBILE-INTEGRATION-NOTES.md §5.4; SR-DEV-* (physical device security for kits — separate from mobile app security).

---

## 15. Security Control Implementation Priorities

Controls are sequenced by election-cycle phase. This section gives the implementation order for Datacraft and the Commission teams.

### 15.1 Pre-Contract (before T-720)

- Confirm HSM vendor and begin procurement (16-24 week lead time for Thales Luna 7 / Utimaco with ML-DSA firmware)
- Confirm Kenya-jurisdiction data-centre availability (Anniversary Towers primary + Bomas mirror confirmed with the Commission)
- Nominate HSM custodians from the Commission and Datacraft (5 total; vetting complete before key ceremony)
- Agree escrow custodian (neutral third party, Kenya-based)
- Stand up internal package mirror and air-gapped build environment

### 15.2 Foundation Phase (T-720 to T-540)

- Root CA key ceremony (air-gapped; quorum-signed; video recorded)
- SPIRE deployment and SVID issuance for all core services
- Vault cluster deployment; seal to HSM; dynamic secrets for all databases
- Keycloak deployment; FIDO2 policy enforced; RBAC structure loaded
- Linkerd 2 service mesh deployed; mTLS verified on all pod-to-pod paths
- Kyverno admission policies enforced (image signing, no privileged containers)
- Falco + Tetragon deployed; SIEM (OpenSearch) receiving all log sources
- SAST + DAST + secret-scan CI gates active on all repositories

### 15.3 Engineering Phase (T-540 to T-180)

- Per-module DPIA filed with ODPC as each module reaches integration test
- ISO 27001 initial certification scope defined and gap analysis complete
- Bug-bounty programme opened for public-facing modules (M11, M13, M29)
- First independent pen-test (annual baseline) completed
- E-SOC staffing and tooling established; first purple-team exercise
- Biometric accuracy lab certification (M03 ABIS)
- M61 Secrecy Assurance Testing red-team engagement begins

### 15.4 Pre-Election Phase (T-180 to T-30)

- ISO 27001 initial certification obtained
- SOC 2 Type II audit completed
- Pre-election independent pen-test (T-90) completed; all P0/P1 findings resolved
- Red-team engagement (T-90 to T-60) completed; IAB reviews findings
- Dress rehearsal (T-60): full election-day simulation; all security controls exercised
- M61 secrecy attestation signed by IAB before T-30
- Software freeze at T-30: no production changes without CCB dual-approval

### 15.5 Election Window (T-30 to T+14)

- War-room active; 24×7 E-SOC staffing
- Bug-bounty suspended
- All P0 alert thresholds reviewed and confirmed active
- Break-glass account drill at T-7

### 15.6 Post-Election (T+14 to T+90)

- Bug-bounty reinstated (T+14)
- Biometric template cryptographic erasure complete (T+90)
- Post-election security review (M41); lessons fed into M61 next cycle
- Annual re-signing of M69 archive scheduled
- ISO 27001 surveillance audit scheduled

---

## 16. Security Governance Roles and Responsibilities

| Role | Organisation | Responsibilities |
|---|---|---|
| the Commission CISO | the Commission | Owns security policy; approves waivers; chairs SRB; signs off OAT security gate; receives all P0 notifications |
| Datacraft Security Lead | Datacraft | Day-to-day ISMS operation; pen-test coordination; M61 red-team engagement; SAST/DAST toolchain; SBOM management |
| the Commission DPO | the Commission | DPA 2019 compliance; DPIA filing; ODPC liaison; DSAR oversight; breach notification signatory |
| IAB Chair (Technical) | Independent | Reviews all P0/P1 security findings; signs secrecy attestation; approves break-glass activation; quarterly SRB member |
| HSM Custodians (3-of-5 quorum) | the Commission (2) + Datacraft (2) + IAB (1) | Key ceremony participation; CA operations quorum; HSM firmware approval |
| E-SOC Manager | Datacraft | 24×7 SOC operations during election window; alert triage; incident escalation; threat intelligence |
| Incident Commander | Datacraft / the Commission (co-named) | P0/P1 incident management; war-room lead; the Commission notification; media-rules enforcement |
| Configuration Control Board | the Commission + Datacraft + IAB observer | Approves all production changes; model updates; security waiver decisions; CCB minutes are public |

---

*Document version 2.0. Approved: Datacraft Security Review Board. Effective: pre-award. Update trigger: module specification change, threat intelligence update, or regulatory amendment.*
