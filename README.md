# eel

Integrated elections management platform, built for the Independent Electoral and Boundaries Commission of Kenya (IEBC) under tender IEBC/OIT/01/2026-2027. Software prime: Datacraft. Hardware prime: Mechrevo. Three-year build and steady-state operations, fifteen-year audit retention, Kenya-sovereign hosting on two IEBC-provided DR sites plus five regional edge clusters.

[![build](https://img.shields.io/github/actions/workflow/status/Datacraft-GaaS/eel/ci.yml?branch=main)](https://github.com/Datacraft-GaaS/eel/actions)
[![coverage](https://img.shields.io/codecov/c/github/Datacraft-GaaS/eel)](https://codecov.io/gh/Datacraft-GaaS/eel)
[![licence](https://img.shields.io/badge/licence-Apache--2.0-blue)](https://opensource.org/licenses/Apache-2.0)

The identifier prefix `eel` is neutral by design: no class, package, event type, topic, header, or trust domain carries the client's name. The word IEBC appears only in prose meaning the Commission.

## What this is

This codebase is a full production election management system: 69 functional modules across 18 Kubernetes services handling the entire electoral life cycle in Kenya, from voter registration and biometric identification through polling-day tally and results certification to public verification, transparency logging, and long-term archive. The platform serves 22 million voters across 46,229 polling stations, 290 constituencies, and 47 counties, with an operational envelope that demands 99.99% availability for election-critical systems, HSM-backed cryptography, post-quantum hybrid signatures, a single monolithic Merkle transparency log published via five channels, and reproducible builds down to bit-identical binaries.

The tender that commissioned this work (IEBC/OIT/01/2026-2027) specified 69 modules grouped into 12 functional areas: voter registration (including special workflows for prisoners, KDF, IDPs, hospital patients), biometric matching and dedup, polling-day identification, results transmission from kits to tally centres, result declaration and certification, public verification (10M+ concurrent verifiers on election night), transparency logging with Merkle trees, chain of custody for equipment and ballots, political party and candidate management, primaries oversight, symbol registry and nomination processes, campaign finance transparency, grievance channels, Independent Assurance Board (IAB) dashboards, election command centre, queue transparency, boundary management (PostGIS-backed), inventory management, RLA post-election audit, election-court IT interface, transmission audit, parliamentary and judiciary reporting, county and international observer coordination, media and observer accreditation, voter education, poll-worker training, medical first response, secrecy assurance testing, financial forensics, referendum mechanics, recall procedures, by-election operations, diaspora voting, voter ID card production, budget tracking, HR payroll for temporary staff, and AI-driven anomaly detection, speech integrity, and turnout forecasting. Every module listed in the RFP acceptance criteria is implemented; every edge case in the 118-row edge-case matrix has test coverage linked in `docs/plans/TRACEABILITY.yaml`.

The trust model is a single monolithic Merkle log per election, with domain-tagged leaves, HSM-signed Signed Tree Heads at 1-second, 60-second, and 5-minute cadences. The log is published on five independent channels: IEBC public website, GitHub mirror, IPFS pinned within Kenya, Kenya Gazette daily print anchor (hash only), and KBC Radio daily hash bulletin. Witness cosigning by the IAB begins day-1; external observers cosign day-2. A reproducibility bundle (M14) allows auditors to rebuild the entire tally from primary evidence and verify it reaches the same Merkle root.

Data is segregated into two tiers: T1 election-critical (voter register, biometric submissions, results transmission, transparency log) mandates 99.99% availability and synchronous DR replication; T2 cycle-critical (party registration, candidate nomination, grievance handling) mandates 99.9% availability and asynchronous replication. Regional edge clusters in Nairobi, Mombasa, Kisumu, Eldoret, and Garissa cache and distribute read traffic; anycast failover to the primary DR site (Anniversary Towers) provides the RTO/RPO bounds listed below.

## Operational envelope

| dimension | value |
|---|---|
| Registered voters (target) | 22M+ |
| Polling stations | ~46,229 |
| Constituencies | 290 |
| Counties | 47 |
| KIEMS kits | ~65,000 (of which ~14,100 legacy Android to reuse) |
| Temporary election staff | ~250,000 |
| Peak concurrent RTS sessions | 60,000 |
| Peak concurrent portal viewers on election night | 5-10M |
| Peak M13 verifier concurrency | 10M |
| Peak concurrent staff check-ins in a 90-minute window | 250,000 |
| Election-day write events (kits + RTS + agents + observers + queues) | 400-600M |
| Sustained write rate at peak hour | 30-50k events/sec |
| Extreme burst (evening RTS submission window) | 80-150k events/sec |
| Statutory audit retention | 15 years |
| Availability class, election-critical modules | 99.99% (T1) |
| Availability class, cycle-critical modules | 99.9% (T2) |
| Availability class, routine modules | 99.5% (T3) |
| Data residency | Kenya (both DR sites and every edge cluster IEBC-provided) |
| Voter-facing languages | 11 (English, Kiswahili + 9 regional) |
| Accessibility floor | WCAG 2.2 AA (AAA for M68 referendum explainer) |
| Security assurance floor | OWASP ASVS L3 on election-critical, L2 elsewhere |
| HSM assurance floor | FIPS 140-3 Level 3+ |

## The 18 Kubernetes services

The 69 modules are implemented by 18 K8s services. One service is deployed identically to both DR sites plus each of the five edge clusters where relevant. Full per-module pages with every feature, acceptance criterion, and edge case are in `docs/plans/modules/`.

| service | primary language | tier | modules | runbook |
|---|---|---|---|---|
| biometric-edge-svc | Rust + Flutter (kits) | T1 | M01 bvr-device, M05 bvi-device, M24 diaspora-voting | [docs/runbooks/biometric-edge-svc.md](docs/runbooks/biometric-edge-svc.md) |
| bvr-svc | Rust + Django admin | T1 | M02 bvr-backend, M35 cvr-continuous-voter-register, M36 voter-deregistration-appeal, M53 voter-id-card | [docs/runbooks/bvr-svc.md](docs/runbooks/bvr-svc.md) |
| abis-svc | Rust + Python + confidential compute | T1 | M03 abis | [docs/runbooks/abis-svc.md](docs/runbooks/abis-svc.md) |
| vprs-svc | Rust + Elixir LiveView | T2 | M04 vprs | [docs/runbooks/vprs-svc.md](docs/runbooks/vprs-svc.md) |
| bvi-svc | Rust + Elixir LiveView + Astro | T1 | M06 bvi-backend, M07 bvv | [docs/runbooks/bvi-svc.md](docs/runbooks/bvi-svc.md) |
| rts-svc | Rust event-sourced + Tauri desktop | T1 | M08 rts-device, M09 rts-ro, M10 rts-backend, M51 result-declaration-certification, M63 ballot-serial-chain | [docs/runbooks/rts-svc.md](docs/runbooks/rts-svc.md) |
| portal-svc | Astro + Elixir LiveView + Rust | T2 | M11 public-portal, M52 election-calendar-gazette, M64 voter-list-public-inspection, M68 referendum-explainer | [docs/runbooks/portal-svc.md](docs/runbooks/portal-svc.md) |
| crms-svc | Rust + Django + Elixir + Python | T2 | M12 crms, M45 party-registration-management, M46 party-primaries-oversight, M47 nomination-fee-payment, M67 party-symbol-colour-registry | [docs/runbooks/crms-svc.md](docs/runbooks/crms-svc.md) |
| transparency-svc | Rust + HSM integration | T1 | M14 reproducibility-bundle, M17 speech-integrity contributions, M22 campaign-finance-transparency, P05 transparency-log, P06 HSM-signer | [docs/runbooks/transparency-svc.md](docs/runbooks/transparency-svc.md) |
| ecc-svc | Elixir LiveView + Rust + Python | T2 | M15 iab-dashboard, M16 election-command-centre, M23 grievance-channels | [docs/runbooks/ecc-svc.md](docs/runbooks/ecc-svc.md) |
| boundary-svc | Rust + PostGIS + Astro | T2 | M18 boundary-transparency | [docs/runbooks/boundary-svc.md](docs/runbooks/boundary-svc.md) |
| queue-svc | Rust + Elixir LiveView + Python CV | T2 | M19 queue-transparency | [docs/runbooks/queue-svc.md](docs/runbooks/queue-svc.md) |
| inventory-svc | Rust event-sourced + Django + Flutter mobile | T2 | M20 inventory-management, M39 ballot-printing-chain, M65 diaspora-ballot-physical-logistics | [docs/runbooks/inventory-svc.md](docs/runbooks/inventory-svc.md) |
| governance-svc | Rust + Python + Elixir + Tauri | T2 | M21 rla-audit-tool, M40 election-court-it-interface, M41 post-election-review, M48 recall, M49 referendum-mechanics, M50 by-election-operations, M56 county-national-coordination, M57 international-observer-coordination, M58 parliamentary-judiciary-reporting, M59 oag-interface, M62 transmission-legal-basis-audit, M66 election-financial-forensics | [docs/runbooks/governance-svc.md](docs/runbooks/governance-svc.md) |
| ai-platform-svc | Python (ONNX) + Rust gateway + Flutter kit plugin | T2 | M17 speech-integrity, M26 anomaly-fraud-detection, M28 ai-monitoring-stack, M44 turnout-forecasting-analytics, M61 secrecy-assurance-testing | [docs/runbooks/ai-platform-svc.md](docs/runbooks/ai-platform-svc.md) |
| field-mobile-svc | Rust BFF + Flutter | T1/T2 | M13 public-verifier-app, M29 mobile-citizen-verify, M30 mobile-party-agent, M31 mobile-poll-clerk, M32 mobile-field-support, M33 mobile-personnel-dispatch, M34 mobile-observer | [docs/runbooks/field-mobile-svc.md](docs/runbooks/field-mobile-svc.md) |
| training-svc | Django + Flutter offline | T3 | M37 voter-education-cms, M38 poll-worker-training-lms, M42 media-accreditation-portal, M43 observer-accreditation-portal, M60 medical-first-response | [docs/runbooks/training-svc.md](docs/runbooks/training-svc.md) |
| support-admin-svc | Django + Rust + Python | T2 | M25 special-voter-workflows, M27 intent-preservation, M54 budget-expenditure, M55 hr-payroll, M69 long-term-archive | [docs/runbooks/support-admin-svc.md](docs/runbooks/support-admin-svc.md) |

## The 43 shared packages

Cross-language libraries under `packages/` provide auth, secrets, events, persistence, observability, and resilience foundations.

| package | languages | purpose |
|---|---|---|
| eel-auth | Rust, Python, Elixir | JWT verification, SpiceDB client, audit trail, OpenTelemetry integration |
| eel-secrets | Rust, Python, Elixir | OpenBao dynamic credentials, transit encryption, key rotation |
| eel-events | Rust, Python, Elixir | Event envelope, producer/consumer, hybrid logical clock, message signing |
| eel-outbox | Rust, Python, Elixir | Transactional outbox pattern, Kafka publisher, idempotency |
| eel-eventstore | Rust, Python, Elixir | Event aggregate, chain history, snapshots, replay |
| eel-pii | Rust, Python, Elixir | Typed PII wrappers (Restricted, Sensitive, Public), masking |
| eel-hlc | Rust, Python, Elixir | Hybrid logical clock, causality tracking, merge semantics |
| eel-resilience | Rust, Python, Elixir | Circuit breakers, bulkheads, idempotency middleware, graceful degradation |
| eel-observability | Rust, Python, Elixir | OpenTelemetry spans, SLO metrics, audit shadow, structured logging |
| eel-loadshedding | Rust, Python, Elixir | Priority-based load shedding, adaptive admission control |
| eel-crypto | Rust | PKCS#11 HSM bindings, Ed25519 signatures, post-quantum hybrid (ML-KEM + ML-DSA), cosign, Merkle trees |
| eel-api-server | Rust | Opinionated axum application builder, middleware stack, health checks |
| eel-web | Python | Django base application, admin UI, schema migrations |
| eel-web | Astro | Static site base, design-system components, i18n setup |
| verifier | Rust (Dart FFI + WASM) | Merkle tree inclusion proof verification, Signed Tree Head verification (libeel-verifier) |
| kit-sdk | Flutter | Shared kit firmware shell, secure enclave storage, device attestation, offline event queue |
| service-template | Rust, Python, Elixir | Scaffolding for new services, CI/CD hooks, test skeleton |
| design-system | Astro, Svelte, Django | Component library, tokens, accessibility guidelines |
| openapi-contracts | OpenAPI 3.1 | Canonical API specifications, contract testing |
| event-schemas | JSON Schema | Event message specifications, evolution rules |
| authz-schema | SpiceDB .zed | Domain authorization policies, permission model definitions |
| data-dictionary | YAML | Column classification schema, PII types, retention policies |
| api-clients | Dart, TypeScript, Python, Rust | Auto-generated client libraries for all services |
| i18n | Message catalogs | 11-language translations (English, Kiswahili + 9 regional languages) |

## Module feature map

Mapping of all 69 modules to owning services:

| M01-M10 | M11-M20 | M21-M30 |
|---|---|---|
| M01 (biometric-edge-svc) | M11 (portal-svc) | M21 (governance-svc) |
| M02 (bvr-svc) | M12 (crms-svc) | M22 (transparency-svc) |
| M03 (abis-svc) | M13 (field-mobile-svc) | M23 (ecc-svc) |
| M04 (vprs-svc) | M14 (transparency-svc) | M24 (biometric-edge-svc) |
| M05 (biometric-edge-svc) | M15 (ecc-svc) | M25 (support-admin-svc) |
| M06 (bvi-svc) | M16 (ecc-svc) | M26 (ai-platform-svc) |
| M07 (bvi-svc) | M17 (transparency-svc, ai-platform-svc) | M27 (support-admin-svc) |
| M08 (rts-svc) | M18 (boundary-svc) | M28 (ai-platform-svc) |
| M09 (rts-svc) | M19 (queue-svc) | M29 (field-mobile-svc) |
| M10 (rts-svc) | M20 (inventory-svc) | M30 (field-mobile-svc) |

| M31-M40 | M41-M50 | M51-M60 | M61-M69 |
|---|---|---|---|
| M31 (field-mobile-svc) | M41 (governance-svc) | M51 (rts-svc) | M61 (ai-platform-svc) |
| M32 (field-mobile-svc) | M42 (training-svc) | M52 (portal-svc) | M62 (governance-svc) |
| M33 (field-mobile-svc) | M43 (training-svc) | M53 (bvr-svc) | M63 (rts-svc) |
| M34 (field-mobile-svc) | M44 (ai-platform-svc) | M54 (support-admin-svc) | M64 (portal-svc) |
| M35 (bvr-svc) | M45 (crms-svc) | M55 (support-admin-svc) | M65 (inventory-svc) |
| M36 (bvr-svc) | M46 (crms-svc) | M56 (governance-svc) | M66 (governance-svc) |
| M37 (training-svc) | M47 (crms-svc) | M57 (governance-svc) | M67 (crms-svc) |
| M38 (training-svc) | M48 (governance-svc) | M58 (governance-svc) | M68 (portal-svc) |
| M39 (inventory-svc) | M49 (governance-svc) | M59 (governance-svc) | M69 (support-admin-svc) |
| M40 (governance-svc) | M50 (governance-svc) | M60 (training-svc) | - |

## Technology stack

| component | version/role | purpose |
|---|---|---|
| **Languages** | | |
| Rust | nightly (stable fallback) | T1 services, HSM bindings, cryptography, performance-critical paths |
| Python | 3.12 + uv | ML/AI, adapters, batch jobs, Django admin surfaces |
| Elixir + OTP | 1.17 + OTP 27 | LiveView real-time dashboards, messaging, soft-real-time event processing |
| Dart + Flutter | 3.2.x | Kit firmware, mobile apps (Android, iOS, PWA), Tauri desktop |
| TypeScript + Node | 22 + pnpm | Astro static sites, API client generation, browser-side verification |
| Bash + Perl | POSIX | Scripting, migrations, data transforms |
| **Databases** | | |
| PostgreSQL | 17 | T1-voter (registration, biometrics), T1-consensus (results), T1-audit, T2-shared |
| Kafka + KRaft | latest (3.7.x) | Event bus, transactional outbox CDC, 90-day hot retention, MirrorMaker 2 for DR sync |
| Karapace | latest | JSON Schema registry, Kafka topics schema validation |
| Valkey | 8 | Cache, session store, idempotency keys, rate limits |
| OpenSearch | 2.x | Full-text and geo-spatial search (M07, M18, M22, M52) |
| MinIO | RELEASE | S3-compatible object store, tiered Kafka storage, snapshot exports |
| Rook-Ceph | latest | Persistent block/object/file storage for DB, MinIO, Kafka |
| **Identity + Crypto** | | |
| Keycloak | 24 | OAuth2/OIDC, SAML, WebAuthn, hardware key (YubiKey, FIDO2) |
| SPIFFE/SPIRE | 1.9.x | Workload identity, pod/kit/mobile SVIDs, HSM-sealed root |
| SpiceDB | 1.31.x | Google Zanzibar implementation, relational permissions |
| OpenBao | 2.x | Secrets engine, dynamic DB creds, PKI, transit encryption, Vault-compatible |
| Hardware HSM | PKCS#11 + KMIP | Thales Luna 7 or Utimaco CryptoServer, FIPS 140-3 L3+, STH signing |
| SoftHSM | 2.x | Development/test environment substitute |
| **Mesh + Networking** | | |
| Kubernetes | 1.31 | Container orchestration, StatefulSets for data tier |
| Cilium | 1.15.x | CNI, service mesh, NetworkPolicy, Hubble observability, eBPF dataplane |
| Envoy | latest | L7 proxy, load balancing, traffic shaping |
| Kong OSS | latest | API gateway, rate limiting, request logging |
| cert-manager | latest | TLS certificate lifecycle, auto-renewal |
| **Platform + Observability** | | |
| ArgoCD | 2.11.x | GitOps, ApplicationSets per service x site, blue-green deployments |
| Prometheus | latest | Metrics scraping, SLO alerting, time-series DB |
| Loki | latest | Structured logging aggregation, log-stream search |
| Tempo | latest | Distributed tracing backend, trace correlation |
| Grafana | latest | Visualization, dashboards, SLO tracking |
| Mimir | latest | Long-term metrics retention |
| OpenTelemetry | latest | Instrumentation SDK, span/metric/log emissions |
| Vector | latest | Log/metric/trace shipping, node-level collection |
| **Security + Scanning** | | |
| Falco | latest | Runtime threat detection, syscall rules |
| Tetragon | latest | eBPF-based threat detection, process tracking |
| Wazuh | latest | SIEM, log analysis, compliance reporting |
| Sigma rules | community | Detection-as-code format, engine-agnostic threat rules |
| Canary tokens | proprietary | Early breach detection, honeypot tokens |
| Semgrep | latest | SAST, Python/Rust/JavaScript/Go rules |
| Bandit | latest | Python static security analysis |
| Sobelow | latest | Elixir/Phoenix security checks |
| cargo-audit | latest | Rust dependency vulnerability scanner |
| cargo-deny | latest | Rust dependency policy enforcement |
| Trivy | latest | Container image and filesystem vulnerability scanning |
| Grype | latest | SBOM-based vulnerability matching |
| Nuclei | latest | DAST templates, API endpoint scanning |
| ZAP | latest | Web application penetration testing |
| MobSF | latest | Mobile app static analysis (Android + iOS) |
| **Testing** | | |
| cargo-nextest | latest | Rust test runner, flaky test detection, partitioning |
| pytest | 8.x | Python unit + integration testing |
| hypothesis | latest | Python property-based testing |
| ExUnit + StreamData | latest | Elixir unit + property tests |
| flutter_test | latest | Dart/Flutter unit and widget tests |
| vitest | latest | TypeScript/JavaScript unit tests |
| proptest | latest | Rust property-based testing |
| kani | latest | Rust bit-level formal verification |
| creusot | latest | Rust deductive verification (Merkle, HLC) |
| TLA+ / Apalache | latest | Temporal specification, election tally logic verification |
| Pact Broker | latest | Consumer-driven contract testing, can-i-deploy gates |
| testcontainers | latest | Ephemeral DB/Kafka/Redis for integration tests |
| k6 | latest | Load testing, RPS ramping, thresholds |
| Gatling | latest | Enterprise load testing scenarios |
| Playwright | latest | Browser E2E testing (portal, verifier) |
| Patrol | latest | Flutter mobile E2E testing |
| tauri-driver | latest | Desktop app (RTS) E2E testing |
| Chaos Mesh | latest | Failure injection, latency/network scenarios |
| **Build + Supply Chain** | | |
| Nix | flakes | Reproducible builds, bit-identical rebuilds, dev environments |
| Sonatype Nexus OSS | latest | PyPI, npm, cargo, Maven, apt, OCI, hex mirrors |
| Forgejo / Gitea | latest | On-prem git hosting, PR CI gateway |
| Woodpecker CI | latest | Event-driven CI/CD pipeline, plugin ecosystem |
| cosign | latest | Container image signing, keyless via SPIFFE |
| SLSA | L3 | Provenance attestations, supply-chain integrity |
| in-toto | latest | Link metadata, step attestations |
| syft | latest | SBOM generation (SPDX + CycloneDX) |
| RFC 3161 | timestamps | Artefact timestamp authority, audit trail |
| **Kyverno** | latest | Admission controller, policy enforcement (signed images, SVID attestation) |
| **OPA Gatekeeper** | latest | General policy enforcement, namespace isolation |

## Statutes and regulations satisfied

- Constitution of Kenya, Article 87: right of election petition
- Constitution of Kenya, Article 104: recall procedure
- Elections Act 2011 (amended): sections 6A (public inspection), 14A (extended hours), 30 (nomination), 39(1C) (transmission), 44 (identification), 44A (complementary mechanism), 53-55 (spoilt ballots), 69 (offences), 72 (secrecy), 75(1) (election courts), 78 (offences reporting), 80 (recount and scrutiny), 83-84 (declaration and certificate)
- Elections (Technology) Regulations 2017: KIEMS certification, transmission audit, independent security certifier
- Election Offences Act: criminal liability provisions
- Election Campaign Financing Act (ECFA): disclosure, reporting, limit enforcement
- Political Parties Act 2011 (amended 2022): registration, primaries, symbol management
- Kenya Data Protection Act 2019: sections 26 (minimisation), 40 (right to erasure), 41 (automated processing), 71 (cross-border transfer)
- ISO/IEC 27001:2022: information security management system (mandated by tender section V)
- ISO/IEC 17799: legacy information security reference standard
- Kenya Information and Communications Act (KICA): digital security requirements
- Data-sharing protocols with NLC, KNBS, IPRS, county governments (MoU-driven)

## How to run locally

Set up a development environment with all dependencies, then spin up a full stack on Docker/Kubernetes.

```bash
# Clone and enter the repository
git clone https://github.com/Datacraft-GaaS/eel && cd eel

# Load the Nix development environment (includes Rust, Python 3.12, Elixir/OTP, Dart, Node 22)
nix develop

# (Optional) Load only specific language environments
nix develop .#rust
nix develop .#python
nix develop .#elixir
nix develop .#dart
nix develop .#node
```

**Spin up the full local stack** (kind cluster + Kafka + PostgreSQL + Redis + MinIO + OpenSearch + Keycloak + SpiceDB + OpenBao + SPIRE + SoftHSM):

```bash
just up
```

This brings up a Kubernetes-in-Docker cluster on `127.0.0.1:6443` with the full data and identity plane. Individual services can then be run locally or deployed to the cluster.

**Generate deterministic synthetic test data** (voter register with no real PII):

```bash
just synth --scale 100k
```

**Validate contracts and schemas**:

```bash
just contracts
```

This runs Spectral OpenAPI linting, Karapace schema validation, and SpiceDB golden-test authorization rules.

**Run all tests**:

```bash
just test
```

This runs cargo-nextest for Rust, pytest for Python, ExUnit for Elixir, flutter_test for Dart, and vitest for TypeScript.

**Run CI locally**:

```bash
just ci
```

Full lint, test, contract, build, SBOM generation, and artifact signing.

**Load synthetic data into the cluster**:

```bash
just seed
```

**Tear down the stack**:

```bash
just down
```

### Local cluster topology

The `just up` command brings up a kind cluster with the following topology:

- Single master node (3 CPU, 8 GB RAM)
- PostgreSQL 17 with Patroni (simulated HA)
- Kafka KRaft (single broker, 90-day retention simulated)
- Karapace schema registry
- Valkey 8 (3-node setup)
- MinIO (single server, auto-replicated buckets)
- OpenSearch (single node)
- Rook-Ceph (emulated block storage)
- Keycloak (single realm for dev)
- SpiceDB (single node)
- OpenBao (unsealed, test-only)
- SPIRE (single root server + agent)
- SoftHSM (PKCS#11 substitute)
- Cilium + Hubble observability
- Prometheus + Loki + Grafana stack

### Troubleshooting

**Issue: `nix develop` fails with "unknown attribute"**

Check your Nixpkgs version and ensure you're on the stable channel. The flake pins to nixpkgs-unstable; if you need a specific version, edit `flake.lock`.

**Issue: `just up` times out**

The cluster bootstrap can take 3-5 minutes on first run. Check logs with `kubectl logs -n kube-system -l app=coredns`. If Kafka fails to start, ensure at least 6 GB free disk and 4 GB available RAM.

**Issue: Services cannot reach PostgreSQL**

Verify Patroni is running: `kubectl get pods -n postgres`. Check that the service DNS resolves: `kubectl run -it --rm debug --image=busybox --restart=Never -- nslookup postgres.default.svc.cluster.local`.

**Issue: Keycloak realm not loading**

Wait for the init Job to complete: `kubectl logs -n keycloak -f job/keycloak-init`. The realm bootstrap takes ~30 seconds on first run.

## Development waves and gates

Seven waves cover the full build plan from toolchain setup through system verification.

| wave | name | gate | status | commits |
|---|---|---|---|---|
| G0 | Toolchain, local profile, contracts, CI | PASS | PASS | - |
| G1 | 43 shared packages, templates, platform base | PASS | PASS | - |
| G2 | Trust core: transparency log, HSM signer, IAM, boundary, tier-A integrations | PASS | PASS | - |
| G3 | Election-night critical path: rts, bvr, abis, bvi, kits, portal, load harness | PASS | PASS | - |
| G4 | Operations, oversight, AI, mobile suite, register surfaces, candidates, edge, observability | PASS | PASS | - |
| G5 | Governance long tail, training, support-admin, diaspora, chaos/DR, runbooks, compliance, reproducibility, manifests, handover | PASS | PASS | - |
| G6 | Full-system verification: rehearsal, DR drill, chaos suite, 5x load, security harness, a11y/i18n sweep, completeness audit | PASS | PASS | 6c6901e, b36c1f7, d08e8a9 |

See full wave plans under `docs/plans/0N-wave-*.md`.

## Testing pyramid

Coverage strategy spans unit, property, integration, E2E, fuzzing, formal methods, load, chaos, security, and reproducibility.

| layer | frameworks | coverage target | status |
|---|---|---|---|
| **Unit** | cargo-nextest (Rust), pytest (Python), ExUnit (Elixir), flutter_test (Dart), vitest (TypeScript) | >= 80% per service | PASSING |
| **Property** | proptest (Rust), hypothesis (Python), StreamData (Elixir), fast-check (TypeScript) | >= 20 edge cases per invariant | PASSING |
| **Contract** | Pact Broker, Karapace, openapi-diff, buf (protobuf) breaking | 100% service pair coverage | PASSING |
| **Integration** | testcontainers (ephemeral DB, Kafka, Redis) | all read/write paths | PASSING |
| **E2E** | patrol (Flutter), Playwright (portal), tauri-driver (RTS desktop), kit-simulator (65k virtual kits) | all user flows | PASSING |
| **Fuzz** | cargo-fuzz (parsers, crypto), libFuzzer | every codec and protocol boundary | PASSING |
| **Formal** | TLA+ / Apalache (tally, run-off, threshold), kani (Merkle tree), creusot (HLC) | safety and liveness properties | PASSING |
| **Load** | k6 (scripted), Gatling (streaming), election-day simulator | 5x peak weekly, 80k ev/s bursts | PASSING |
| **Chaos** | Chaos Mesh (network, storage, compute failures) | monthly pre-prod drill | PASSING |
| **Security** | Semgrep, Bandit, cargo-audit, Trivy, Grype, ZAP, Nuclei, testssl.sh, ASVS L3/L2 checklist | ASVS L3 election-critical, L2 elsewhere | PASSING |
| **Accessibility** | axe, manual screen-reader scripts, 11-language pseudo-locale | WCAG 2.2 AA (AAA for M68) | PASSING |
| **Reproducibility** | Nix flakes, bit-identical rebuild, hash manifest | M14 make-reproduce v2 | PASSING |
| **Mutation** | cargo-mutants (Rust), mutmut (Python), Stryker (TypeScript) | 75%+ kill rate on election-critical | PASSING |

## Security and trust model

Election-critical systems (T1, rated 99.99%) are built to OWASP ASVS Level 3; others to Level 2. Cryptography uses FIPS 140-3 Level 3+ HSMs, post-quantum hybrid signatures (ML-DSA + ECDSA P-384), and MLKem-768 + X25519 for key encapsulation. TLS 1.3 only, HSTS, DANE, and ECH where supported.

The transparency model is a single monolithic Merkle log per election. Leaves are domain-tagged; Signed Tree Heads (STHs) are HSM-signed at 1-second, 60-second, and 5-minute cadences. STHs are published on five channels: IEBC website, GitHub mirror, IPFS pinned in Kenya, Kenya Gazette daily print, and KBC Radio hash bulletin. The Independent Assurance Board (IAB) cosigns day-1; external observers cosign day-2. Reproducibility bundle (M14) allows bit-identical rebuild.

For details, see `docs/architecture/cross-cutting/trust-model.md`.

## Reproducibility

Every build is reproducible. The M14 reproducibility bundle contains a Nix flake that can rebuild the entire system from source to a bit-identical binary.

```bash
cd reproducibility/
make reproduce v2
```

This downloads the original signed manifests, rebuilds every service, verifies the Merkle root matches, and generates a completeness audit. Builders, auditors, and courts can independently re-derive and verify the tally without access to the source code repository.

## Contributing

Contributions follow these rules:

- **No client names in code**: Use the `eel` prefix for packages, events, topics, headers, and trust domains. IEBC appears only in prose.
- **No em/en dashes**: Use commas, colons, or periods. This file and all documentation enforce this.
- **Commit conventions**: Follow conventional commits (feat, fix, docs, refactor, test, chore). Include module IDs and ticket refs where applicable.
- **DCO sign-off**: All commits must include `Signed-off-by: Your Name <your.email@example.com>`. Use `git commit -s`.
- **PR checklist**: Link relevant module pages, include acceptance criteria, declare breaking changes, add test IDs to the PR body.
- **Code of conduct**: Follow the [Contributor Covenant](https://www.contributor-covenant.org/) Code of Conduct.

See `CONTRIBUTING.md` for full details.

## Licence

Licensed under the Apache License 2.0. Copyright: Datacraft (builder) and Independent Electoral and Boundaries Commission of Kenya (commissioning agency).

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## Contact and support

Operational on-call rotations, break-glass custodians, and escalation contacts are in `docs/runbooks/break-glass.md`.

For module-specific ownership and wave assignment, see `docs/plans/module-owners.md`.

For general inquiries about the IEBC Electoral Technology Programme, contact the IEBC ICT directorate.
