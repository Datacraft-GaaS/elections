# eel

Integrated elections management platform, built for the Independent Electoral and Boundaries Commission of Kenya (IEBC) under tender IEBC/OIT/01/2026-2027. Software prime: Datacraft. Hardware prime: Mechrevo. Three-year build and steady-state operations, fifteen-year audit retention, Kenya-sovereign hosting on two IEBC-provided DR sites plus five regional edge clusters.

The identifier prefix `eel` is neutral by design: no class, package, event type, topic, header, or trust domain carries the client's name. The word IEBC appears only in prose meaning the Commission.

## Scope in one paragraph

Sixty-nine modules across kits, mobile apps, backend services, portals, and operator desktops cover the full election life cycle: voter registration and continuous roll maintenance, biometric identification and dedup, polling-day identification, results transmission and tally, declaration and certification, public verification, transparency logging, chain of custody for kits and ballots, party and candidate management, primaries oversight, symbol registry, nomination fee payment, election calendar and gazette notices, campaign finance transparency, grievance channels, IAB oversight, election command centre, queue transparency, boundary transparency, RLA post-election audit, election-court IT interface, transmission legal-basis audit, parliamentary and judiciary reporting, Auditor-General interface, county coordination, international observer coordination, media and observer accreditation, voter education CMS, poll-worker training LMS, medical first response, secrecy assurance testing, financial forensics, referendum mechanics, recall, by-election operations, diaspora voting and logistics, special-voter workflows (prisoners, hospital, KDF, IDPs), voter list public inspection, referendum explainer, long-term archive, budget and expenditure, HR and payroll for temporary staff, AI monitoring, anomaly and fraud detection, speech and disinformation integrity, turnout forecasting, intent preservation, voter ID card production. Everything the tender lists, and every capability the acceptance criteria demand.

## Operational envelope

| dimension | number |
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
| Availability class, election-critical modules | 99.99% |
| Availability class, cycle-critical modules | 99.9% |
| Availability class, routine modules | 99.5% |
| Data residency | Kenya (both DR sites and every edge cluster IEBC-provided) |
| Voter-facing languages | 11 (English, Kiswahili + 9 regional) |
| Accessibility floor | WCAG 2.2 AA (AAA for M68 referendum explainer) |
| Security assurance floor | OWASP ASVS L3 on election-critical, L2 elsewhere |
| HSM assurance floor | FIPS 140-3 Level 3+ |

## The 18 Kubernetes services (ADR-0020)

The 69 modules are implemented by 18 K8s services. One service is deployed identically to both DR sites plus each of the five edges where relevant.

| service | primary language | modules |
|---|---|---|
| biometric-edge-svc | Rust + Flutter (kits) | M01 bvr-device, M05 bvi-device, M24 diaspora-voting |
| bvr-svc | Rust + Django admin | M02 bvr-backend, M35 cvr-continuous-voter-register, M36 voter-deregistration-appeal, M53 voter-id-card |
| abis-svc | Rust + Python + confidential compute | M03 abis |
| vprs-svc | Rust + Elixir LiveView | M04 vprs |
| bvi-svc | Rust + Elixir LiveView + Astro | M06 bvi-backend, M07 bvv |
| rts-svc | Rust event-sourced + Tauri desktop | M08 rts-device, M09 rts-ro, M10 rts-backend, M51 result-declaration-certification, M63 ballot-serial-chain |
| portal-svc | Astro + Elixir LiveView + Rust | M11 public-portal, M52 election-calendar-gazette, M64 voter-list-public-inspection, M68 referendum-explainer |
| crms-svc | Rust + Django + Elixir + Python | M12 crms, M45 party-registration-management, M46 party-primaries-oversight, M47 nomination-fee-payment, M67 party-symbol-colour-registry |
| transparency-svc | Rust with HSM integration | M14 reproducibility-bundle, M17 speech-integrity contributions, M22 campaign-finance-transparency, transparency log (P05), HSM signer (P06) |
| ecc-svc | Elixir LiveView + Rust + Python | M15 iab-dashboard, M16 election-command-centre, M23 grievance-channels |
| boundary-svc | Rust + PostGIS + Astro | M18 boundary-transparency |
| queue-svc | Rust + Elixir LiveView + Python CV | M19 queue-transparency |
| inventory-svc | Rust event-sourced + Django + Flutter mobile | M20 inventory-management, M39 ballot-printing-chain, M65 diaspora-ballot-physical-logistics |
| governance-svc | Rust + Python + Elixir + Tauri | M21 rla-audit-tool, M40 election-court-it-interface, M41 post-election-review, M48 recall, M49 referendum-mechanics, M50 by-election-operations, M56 county-national-coordination, M57 international-observer-coordination, M58 parliamentary-judiciary-reporting, M59 oag-interface, M62 transmission-legal-basis-audit, M66 election-financial-forensics |
| ai-platform-svc | Python (ONNX) + Rust gateway + Flutter kit plugin | M17 speech-integrity, M26 anomaly-fraud-detection, M28 ai-monitoring-stack, M44 turnout-forecasting-analytics, M61 secrecy-assurance-testing |
| field-mobile-svc | Rust BFF + Flutter | M13 public-verifier-app, M29 mobile-citizen-verify, M30 mobile-party-agent, M31 mobile-poll-clerk, M32 mobile-field-support, M33 mobile-personnel-dispatch, M34 mobile-observer |
| training-svc | Django + Flutter offline | M37 voter-education-cms, M38 poll-worker-training-lms, M42 media-accreditation-portal, M43 observer-accreditation-portal, M60 medical-first-response |
| support-admin-svc | Django + Rust + Python | M25 special-voter-workflows, M27 intent-preservation, M54 budget-expenditure, M55 hr-payroll, M69 long-term-archive |

Full per-module pages with every feature, acceptance criterion, and edge case are in `docs/plans/modules/`.

## Client surfaces

| surface | technology | where |
|---|---|---|
| KIEMS kit firmware | Flutter/Dart on Android (new + legacy) | `services/<svc>/apps/<name>-kit/` |
| Legacy Android kit shim | Flutter with parity matrix | `services/biometric-edge-svc/apps/legacy-android/` |
| Public voter mobile app | Flutter (Android + iOS + PWA) | `services/field-mobile-svc/apps/citizen-verify/` |
| Accredited-role mobile apps | Flutter with MDM (Android Enterprise, Apple Business Manager) | `services/field-mobile-svc/apps/{party-agent,poll-clerk,field-support,personnel,observer}/` |
| Static public portals | Astro 5 + Svelte islands | `services/*/astro/` |
| Real-time public dashboards | Phoenix LiveView (Elixir) | `services/*/ex/` |
| Internal admin portals | Django 5 + HTMX + Alpine | `services/*/django/` |
| Operator desktop | Tauri 2 (Rust core + web UI) | `services/*/apps/*-desktop/` |
| SMS + USSD + WhatsApp + IVR | telco gateways + BFF | `services/field-mobile-svc/py/{ussd-menu,wa-bot,ivr-hook}/` |
| Public API (5 tiers per ADR-0011) | Rust (axum + utoipa + tonic) | `services/*/crates/*-api/` |
| Kiosk mode | Flutter kiosk overlay of M07 bvv | `services/bvi-svc/apps/bvv-kiosk/` |

## Data infrastructure (ADR-0003, ADR-0012)

| store | role | topology |
|---|---|---|
| PostgreSQL 17 | T1-voter, T1-consensus, T1-audit, T2-shared (schema-per-service) | Patroni HA on 2 DR sites + read replicas at each edge |
| Kafka Kraft + Debezium + Karapace | event bus, transactional-outbox CDC, JSON Schema registry | 3 brokers per DR, MirrorMaker 2 for site-to-site; 90-day hot retention + MinIO Tiered Storage |
| MinIO | object store: signed snapshots, bundle exports, blobs, tiered Kafka storage | 2 sites, active-active replication |
| Valkey 8 | cache, session store, idempotency cache, rate-limit buckets | 3 nodes per site + at each edge |
| OpenSearch | full-text and geo search for M07, M18, M22, M52 | 1 cluster per DR |
| PostgreSQL replicas + materialised views | analytics day-1 (ClickHouse deferred) | dedicated read-only clusters |
| Rook-Ceph | block/object/file storage under MinIO and DBs | per DR |

## Identity, authorization, secrets (ADR-0007)

- **Human AuthN**: Keycloak realms (staff, party, observer, media, public, county). OIDC + SAML + WebAuthn + hardware key (YubiKey / FIDO2) mandatory for IAB, Chair, declaration, ECC. National-ID federation to IPRS / e-Citizen when live.
- **Workload identity**: SPIFFE/SPIRE with HSM-sealed root. Every pod, kit, and mobile app carries a short-lived SVID (X.509 or JWT); rotation hourly.
- **Domain authorization**: SpiceDB (Google Zanzibar impl). Election permissions are relational; schema in `packages/authz-schema/*.zed` with golden tests per persona.
- **Admission + config policy**: Kyverno + OPA Gatekeeper. Only signed images, only SVID-attested workloads, only mesh-member connections.
- **Secrets**: OpenBao (Vault fork). HSM-sealed. Dynamic DB credentials, PKI, transit encryption. JIT admin creds capped at 4 hours with session recording.
- **Break-glass**: Shamir 3-of-5 across three named custodians (IEBC Chair rep, CJ rep, AG rep, plus two operational alternates). Every ceremony operation appears as a transparency-log leaf.

## Crypto (ADR-0005)

- HSM: PKCS#11 + KMIP abstraction. FIPS 140-3 L3+. Vendor-swappable (Thales Luna 7 or Utimaco CryptoServer).
- TLS: 1.3 only, HSTS preload, DANE where present, ECH where clients support.
- Post-quantum hybrid: **ML-KEM-768 + X25519** for KEM; **ML-DSA + ECDSA P-384** hybrid signatures; **SLH-DSA (FIPS 205)** for long-tail archive re-signing.
- At-rest: LUKS + HSM-sealed keys; column-level for Tier-A PII (ADR-0016).
- Artifact signing: cosign, keyless via SPIFFE for CI-built, keyed via HSM for release.
- Per-election-cycle KEK hierarchy escrowed to OAG post-election.

## Transparency (ADR-0017)

Single monolithic Merkle log per election, domain-tagged leaves, HSM-signed Signed Tree Heads at 1s / 60s / 5min cadences. Five-channel publication: IEBC public website, GitHub mirror, IPFS pinned in Kenya, Kenya Gazette daily print anchor, KBC Radio daily hash bulletin. Witness cosigning by IAB day-1; external observers day-2. Reproducibility bundle (M14) rebuilds tally to the same root; auditors can re-derive from primary evidence.

## Platform (ADR-0004, ADR-0019, ADR-0022)

- Kubernetes 1.31 on Ubuntu 24.04 LTS. Kubeadm-provisioned. Cilium as CNI + service mesh + NetworkPolicy + Hubble observability (one component doing four jobs).
- Ingress: Envoy Gateway with Kong control plane. Coraza WAF + Anubis PoW at each edge.
- GitOps: ArgoCD with ApplicationSets fanning per service × site. Blue-green with 15-minute hot rollback for stateful services; canary for public portals; rolling for stateless.
- Registry: Harbor with cosign verification enforced by Kyverno admission.
- Observability: OpenTelemetry Collector into Prometheus + Loki + Tempo + Grafana + Mimir + Vector node agent.
- Detection: Falco + Tetragon (eBPF), Wazuh SIEM, Sigma rules, canary tokens.
- Sites: 2 DR (Anniversary Towers primary + Bomas mirror, per RFP; sync-replicated on T1, async on T2). 5 regional edges: Nairobi, Mombasa, Kisumu, Eldoret, Garissa.
- DR SLOs: rts-svc RTO ≤30 min / RPO ≤5 min; other T1 ≤60 min / ≤15 min; T2 ≤4 h / ≤1 h; edge ≤15 min (anycast failover).

## Build and supply chain (ADR-0006)

- Reproducible builds: Nix flakes for every service and every image. Bit-identical rebuilds required (M14 `make reproduce`).
- Air-gapped path: Sonatype Nexus OSS mirrors PyPI + npm + cargo + Maven + apt + OCI + hex. No upstream fetch on the build path.
- Forge + CI: Gitea/Forgejo + Woodpecker CI, all on-prem.
- Provenance: cosign signatures, SLSA L3 attestations, in-toto steps, syft SBOMs in SPDX and CycloneDX, RFC 3161 timestamps for release artefacts.
- SAST + SCA: Semgrep, Bandit, Sobelow, MobSF, cargo-audit, cargo-deny, cargo-vet, pip-audit, pnpm audit, Trivy, Grype.
- Contract tests: Pact Broker with `can-i-deploy` gate between every service pair.
- Formal methods: TLA+/Apalache for tally, referendum threshold, run-off/tie, register-freeze; kani/creusot for Merkle and HLC primitives.
- Freeze: 30-day pre-election merge lock enforced by CI gate.

## External integrations

Government and telco integrations are the surface where the platform meets the country. Each adapter lives under `integrations/<name>/` with a Pact contract test and a stand-in double for dev.

IPRS national ID validation, MRZ parser, telco IPsec VPN, Power BI push, legacy ABIS adapter, legacy Android device shim, HSM PKCS#11 vendor profiles, SMS + USSD gateway (Africa's Talking, Safaricom SMPP), WhatsApp Business API, IVR provider, KRA iTax, EACC, IFMIS, PPRA, OAG, Auditor-General, Central Bank of Kenya (CBK), Financial Reporting Centre (FRC), Kenya Bureau of Standards (KEBS), Office of the Data Protection Commissioner (ODPC) 72-hour breach reporting, Government Printer (Gazette), KBC Radio SFTP, Kenya National Archives and Documentation Service (KNADS), DataCite DOI, Kenya Judiciary eCourt, Political Parties Disputes Tribunal (PPDT), Media Council of Kenya (MCK), NCIC + DCI + ODPP evidence endpoints, Kenya Meteorological Department (KMD), Kenya Airways Cargo + IATA Cargo2000, KRA e-Clearance, MFA Kenya for diaspora logistics, Pantone colour registry, social platforms (X, Facebook, TikTok, YouTube monitoring), LiteLLM tenant + NLLB for translations, CRS (Civil Registration Services), NRB (National Registration Bureau), Kenya Prisons, KDF/GSU, UNHCR ProGres v4, NDOC, Red Cross, IPFS pinning in Kenya, satellite fallback (Starlink/VSAT), M-Pesa Daraja C2B/B2C, NSSF, NHIF/SHIF, printer MES, card manufacturer, NIST Randomness Beacon.

## Statutes and regulations the platform enforces

- Constitution of Kenya, Articles 87 (right of election petition) and 104 (recall).
- Elections Act (2011, amended): §6A public inspection, §14A extended hours, §30 nomination, §39 transmission, §44 identification, §44A complementary mechanism, §§53-55 spoilt ballots, §69 offences, §72 secrecy, §75 (elections courts timing), §78 offences reporting, §80 recount and scrutiny, §§83-84 declaration and certificate.
- Elections (Technology) Regulations 2017: KIEMS certification, transmission audit, independent security certifier.
- Election Offences Act.
- Election Campaign Financing Act (ECFA).
- Political Parties Act (2011, amended 2022).
- Kenya Data Protection Act 2019 (DPA), especially §26 minimisation, §40 right to erasure, §41 automated processing, §71 cross-border transfer.
- ISO/IEC 27001:2022 (mandated by tender §V).
- ISO/IEC 17799 (legacy reference).
- Kenya Information and Communications Act (KICA).
- Data-sharing protocols with NLC, KNBS, IPRS, and county governments (MoU-driven).

## Reliability, edge cases, testing

`docs/plans/08-edge-case-matrix.md` lists 118 universal edge cases spanning connectivity (offline 7 days, 2G/3G, satellite, partition, flapping, captive portal, dual-SIM failover, IPsec drop, ISR shrink), time (clock skew, NTP loss, DST, HLC merge, leap second, s.14A extension, T-60 freeze race, EAT boundary), identity (kit cert revoked mid-session, JWT expiry mid-request, SVID rotation, attestation fail, rooted device, clone app, SIM swap, duress finger, dual-control revocation, SpiceDB down, JWKS stale), data (idempotent replay, out-of-order, poison payload, schema v1/v2 coexist, 30M-row ops, Unicode names, zero-results, arithmetic invalid, Debezium lag, oversized event, IPRS duplicate person, legacy shim envelope), security (signature mismatch, tamper, HSM unavailable, KEK rotation mid-cycle, injection, SSRF, path traversal, XXE, timing side-channels, cache poisoning, split-view, stolen kit honey mode, LiveView hijack, secrets in logs, PQ negotiation), human factors (worn fingerprints, amputee, gloves, low light, assisted voting, screen readers, KSL, 11 languages, colour-blind, illiterate USSD, Braille, clerk fatigue), legal (candidate withdrawal, court postponement, s.44A, recount, run-off, tie, referendum threshold off-by-one, petition-window boundary, T-60 auto-referral race, Daraja duplicate, correction after declaration), operations (kit battery mid-session, kit swap, seal broken, dual-control absent officer, power outage tally centre, flood/curfew relocation, staff no-show, disk full, wrong degradation mode, PITR during petition), scale (5M/hr flash crowd, thundering herd on STH, hot partition, consumer lag, sequencer stall, tiering lag, 250k check-ins in 90 min, Valkey eviction), ML (drift, adversarial, degraded forms, group bias, PII leak, unsigned model, alert flood), mobile (OS kill, storage full, 1 GB RAM device, Doze, push token rotation, update mid-election, GPS spoof, device time wrong, biometric sensor broken), kit (GNSS jam/spoof, legacy parity, OTA interrupted, mode-switch not propagated). Every module page under `docs/plans/modules/` maps its applicable edge cases to test IDs; `docs/plans/TRACEABILITY.yaml` seeds the union (4,444 rows).

Test pyramid (`docs/plans/09-test-pyramid.md`): unit (cargo-nextest / pytest / ExUnit / flutter_test / vitest), property (proptest / hypothesis / StreamData / fast-check), contract (Pact + Karapace compat + openapi-diff + buf breaking), integration (testcontainers), e2e (patrol + Playwright + tauri-driver + custom kit-simulator up to 65k virtual kits), fuzz (cargo-fuzz on every parser and protocol boundary), formal (TLA+ / kani / creusot), load (k6 + Gatling + election-day simulator, 5x peak weekly), chaos (Chaos Mesh scenarios monthly at pre-prod), security (Semgrep + Bandit + cargo-audit + Trivy + Grype + ZAP + Nuclei DAST + testssl.sh + ASVS L3/L2 checklist), accessibility (axe + manual screen-reader scripts), i18n (pseudo-locale + 11 real locales), reproducibility (bit-identical rebuild), mutation (cargo-mutants + mutmut + Stryker thresholds).

## Repository layout

```
services/           18 K8s services grouping the 69 modules
  <svc>/
    crates/         Rust workspace crates
    py/             Python packages (ML, adapters, batch)
    ex/             Elixir Phoenix + LiveView apps
    django/         Django admin apps
    astro/          Astro 5 static sites
    apps/           Flutter kit and mobile + Tauri desktop bundles
    migrations/     schema migrations
    deploy/         Kustomize base + per-site overlays
    tests/          integration and E2E harnesses
    loadtest/       service-specific load scenarios
    docs/           service README, EVENTS.md, THREAT-MODEL.md
    data_dictionary.yaml
modules/            69 module design and spec homes
packages/           shared cross-language libraries
  eel-auth-{rs,py,ex}         JWT verify + SpiceDB + audit + OTel
  eel-secrets-{rs,py,ex}      OpenBao dynamic creds + transit
  eel-events-{rs,py,ex}       producer/consumer, envelope, HLC, signing
  eel-outbox-{rs,py,ex}       transactional outbox (only path to Kafka)
  eel-eventstore-{rs,py,ex}   Aggregate + prev_hash chain + snapshots + replay
  eel-pii-{rs,py,ex}          typed wrappers (Restricted/Sensitive/Public)
  eel-hlc-{rs,py,ex}          hybrid logical clock
  eel-resilience-{rs,py,ex}   circuit breakers, bulkheads, idempotency middleware
  eel-observability-{rs,py,ex}  OTel + SLO metrics + audit-shadow
  eel-loadshedding-{rs,py,ex} priority-based load shedding
  eel-crypto-rs               PKCS#11 + Ed25519 + PQ hybrid + cosign + Merkle
  eel-api-server-rs           opinionated axum builder
  eel-web-py                  Django base app
  eel-web-astro               Astro base site
  verifier-rs (libeel-verifier)  Merkle inclusion + STH verify (Dart FFI + WASM)
  kit-sdk-flutter             shared shell: secure store, attestation, offline queue
  service-template-{rs,py,ex}
  design-system/{astro,svelte,django}
  openapi-contracts/          OpenAPI 3.1 specs
  event-schemas/              JSON Schemas
  authz-schema/               SpiceDB .zed
  data-dictionary/            column classification schema
  api-clients/{dart,ts,py,rs} auto-generated
  i18n/                       11-language message catalogues
platform/           K8s + mesh + identity + secrets + registry + observability manifests
edge/               5 regional edge cluster configs + WAF + rate limits + origin shield
integrations/       external-system adapters with doubles and Pact contracts
operations/         Kenya-based Tier 1-3 ops tooling (MDM, helpdesk, e-learning, warranty, dispatch, data migration)
compliance/         DPA toolkit, ISO 27001 evidence, Elections Tech Regs test harness, IPR handover
chaos/              Chaos Mesh scenarios and monthly drill schedule
loadtest/           k6 + Gatling + election-day simulator + local profile
detection/          Falco + Tetragon + Sigma rules + Wazuh config + canary tokens
reproducibility/    M14 make-reproduce toolkit, Nix flakes, hash manifests
air-gapped-build/   Nexus mirror config, build-image definition
environments/       dev / staging / pre-prod / prod ArgoCD overlays
tests/              cross-service e2e + integration
docs/               plans, ADRs, spec alignment, decision log, event conventions, runbooks, module index
tools/              scaffolding, synthetic data, completeness scanner, DR failover harness, appset generator
```

## Getting started

```bash
git clone https://github.com/Datacraft-GaaS/elect eel && cd eel
nix develop                # or: nix develop .#rust .#python .#elixir .#dart .#node
just up                    # kind + Kafka + Karapace + PG + Valkey + MinIO + OpenSearch + Keycloak + SpiceDB + OpenBao + SPIRE + SoftHSM
just synth --scale 100k    # deterministic synthetic register (no real PII in dev)
just contracts             # spectral + Karapace + zed goldens
just test                  # nextest + pytest + mix test + flutter test
just ci                    # lint + test + contracts + build + sbom + sign
just seed                  # load synthetic data into local kind
just down                  # tear down
```

## How the build proceeds

Seven waves. Each work package (WP) is a self-contained agent brief with paths, contracts, tests, and a hard exit bar (`docs/plans/10-definition-of-done.md`, 25 items).

- **Wave 0** — Toolchain, local profile, contracts, CI. `docs/plans/01-wave-0-toolchain.md`
- **Wave 1** — 43 shared packages + templates + platform base. `docs/plans/02-wave-1-packages.md`
- **Wave 2** — Trust core: transparency log, HSM signer, IAM, boundary, tier-A integrations. `docs/plans/03-wave-2-trust-core.md`
- **Wave 3** — Election-night critical path: rts, bvr, abis, bvi, kits, portal, load harness. `docs/plans/04-wave-3-critical-path.md`
- **Wave 4** — Operations, oversight, AI, mobile suite, register surfaces, candidates, edge, observability. `docs/plans/05-wave-4-operations.md`
- **Wave 5** — Governance long tail, training, support-admin, diaspora, chaos/DR, runbooks, compliance, reproducibility, manifests, handover. `docs/plans/06-wave-5-governance.md`
- **Wave 6** — Full-system verification: rehearsal, DR drill, chaos suite, 5x load, security harness, a11y/i18n sweep, completeness audit. `docs/plans/07-wave-6-verification.md`

The completeness audit (`docs/plans/11-completeness-audit.md`) fails the build if any module × acceptance criterion × edge case in `docs/plans/TRACEABILITY.yaml` maps to zero passing tagged tests. Nothing ships stubbed.

## Decision history

`docs/adr/README.md` indexes 24 architecture decision records (0001-0024). The narrative of how they were arrived at, and every alternative weighed, is in `docs/architecture/decision-log.md`. Divergences between the earlier `sw/` bid drafts and the ADRs are listed in `docs/spec-alignment/` with the exact edits to apply to those documents. IEBC pre-bid clarifications outstanding are in `docs/iebc-clarifications-outstanding.md`; internal bid-document backlog in `docs/datacraft-internal-backlog.md`.

## Contributor rules

- No identifier carries the client name. Packages, events, topics, headers, envelope fields, trust domains: `eel` prefix. IEBC appears only in prose meaning the Commission.
- No em dashes or en dashes in any prose file. Commas, colons, periods.
- Every new module page under `docs/plans/modules/` must map to at least one work package.
- Every new event needs a schema in `packages/event-schemas/`, a topic in `platform/kafka/topics/`, a Pact idempotency test, and an entry in the owning module's `docs/EVENTS.md`. Only `eel-outbox-*` publishes.
- Every new endpoint needs an OpenAPI 3.1 spec in `packages/openapi-contracts/`, regenerated clients in all four languages, contract tests both sides, and ASVS L3/L2 compliance.
- Every new column needs a `data_dictionary.yaml` classification. PII types (`PIIRestricted`, `PIISensitive`, `PIIPublic`) are the only path to Tier A/B data.
- Every write endpoint is idempotent by `Idempotency-Key`.
- Every service ships normal / degraded / read-only modes and a `docs/runbooks/<svc>-degradation.md`.
- Every PR runs `just ci` clean.

## Contact and ownership

Owners are recorded per module in `docs/modules/README.md`. Wave and WP owners are recorded per file under `docs/plans/`. Operational on-call rotations, break-glass custodians, and the Independent Assurance Board contact list are in `docs/runbooks/break-glass.md`.

## Status

Design phase complete. 24 ADRs committed. 69 module pages plus full build plan plus 118-row edge-case matrix plus 4,444-row traceability seed in `docs/plans/`. Wave 0 execution in progress. See `docs/audit/gate-G<n>.md` files as they land for wave sign-offs.
