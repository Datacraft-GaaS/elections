# IEBC IEMS — session design log

Date: 2026-08-21
Status: living document; reflects the state at end of the initial design session
Scope: captures the evolution of every load-bearing design decision made during the session that scaffolded this repository and set the platform direction for the Kenya IEBC IEMS 2026-2027 tender (IEBC/OIT/01/2026-2027).

This is a narrative log, not a specification. Each section records the state of the question, the alternatives weighed, the decision, and the evidence or constraint that drove it. Downstream ADRs 0001-0018 will formalise the final decisions. This document explains how we got here and why we did not go elsewhere.

---

## 1. Session summary

Starting point: an empty working directory (`/Users/nyimbiodero/src/pjs/eel`) with no code, no requirements, and a vague brief of "30 modules across desktop, mobile, server."

Ending point: a committed scaffold at commit `4d7e5c6` on branch `main`, an OSS-first design across ~15 architectural areas, seventeen planned ADRs, and a shared understanding that the project targets ~69 modules and growing under the constraints of election-critical integrity, sovereign hosting, air-gapped builds, 15-year audit retention, and survival under nation-scale adversarial load on election night.

---

## 2. Working set as of end of session

- **System**: Kenya IEBC Integrated Elections Management System (IEMS), tendered as IEBC/OIT/01/2026-2027. Datacraft is software prime; Mechrevo supplies hardware.
- **Scale**: ~69 modules on disk with slug gaps up to M65+ suggesting active drafting; design must accommodate ~100 modules over the 3-year contract.
- **Deployment**: three-site on-prem K8s (Anniversary Towers primary, Bomas of Kenya mirror sync-replicated, escrow site async).
- **Operational envelope**: 22M voters, 46k polling stations, ~65% turnout, 400-600M write events per election day, 5-10M concurrent public viewers.
- **Regulatory anchors**: Kenya DPA 2019, Elections Act §32 (15-year retention), Election Offences Act, ECFA, ISO 27001, and audit exposure to the Independent Assurance Board plus international observers.

---

## 3. Chronological decision log

### Phase 1 — Empty scaffold

**Question**: Give the repository a shape before requirements are read.

**Initial assumption**: 30 numbered modules, each with a shared/server/desktop/mobile quadrant.

**Decision**: Committed a monorepo scaffold at `4d7e5c6` with `apps/{desktop,mobile,server}`, `packages/{shared,sdk,ui,config}`, `modules/mod-01..mod-30/{shared,server,desktop,mobile,docs}`, `docs/{architecture,adr,research,api,modules,guides,runbooks}`, `infra/`, `tests/`, `scripts/`, `tools/`, `assets/`, `config/`, plus `README.md`, `.gitignore`, `.editorconfig`, `docs/architecture/overview.md`, ADR-0001 (monorepo layout, accepted) and ADR-0002 (app stacks, proposed).

**Justification recorded in ADR-0001**: monorepo with per-module vertical slices lets a single team own end-to-end module delivery, keeps contracts colocated with implementations, and makes cross-platform refactors a single PR at the cost of requiring disciplined boundary enforcement.

**Left open**: language and framework choices, the actual module identities, and everything downstream that depends on requirements.

---

### Phase 2 — Discovery of the actual requirements

**Trigger**: User pointed at `/Users/nyimbiodero/src/pjs/work_docs/01-Projects/IEBC/sw` — a 250KB+ requirements corpus including PRD, ARCHITECTURE, DATA-MODEL, SECURITY, API-CATALOG, ACCEPTANCE-CRITERIA, DATACRAFT_SOFTWARE_SCOPE (127KB), plus a `modules/` directory of individual module specs.

**Method**: Forked a reader (agent id `a15be2448ea981ce3`) with instructions to skim the master scope, deep-read only where it resolved specific questions, index all module specs from filenames plus first paragraph, and return a synthesized brief under 800 words. Rationale: keep raw specs out of the main context; only the synthesis returns.

**Findings that reshaped the initial scaffold**:

1. Module count is not 30. `README.md` said 12, `PRD.md` and `expect.md` said 28 (M01-M28), the `modules/` directory held 34 filenames (adding M29-M34 mobile apps). Later in the session a recount showed 53 files on disk numbered up to M65+ with gaps, and the user confirmed 69 modules and growing.
2. The scaffold's `{shared,server,desktop,mobile}` quadrant per module is wrong for approximately 26 of the modules. Actual surface distribution: 6 device-firmware (KIEMS kits), 6 backend-only, 7 web-only, 3 desktop workstation, 7 mobile-only, 5 multi-surface. Firmware and web were missing categories; mobile only applied to a minority.
3. Stack anchors are largely fixed by ARCHITECTURE.md: PostgreSQL 17, ClickHouse, Kafka, MinIO, Redis, Elasticsearch, Kubernetes, SPIFFE/SPIRE, cosign admission, TLS 1.3 + PQ hybrid (ML-KEM + X25519), FIPS-graded HSM, air-gapped Nix/Bazel reproducible builds, 15-year immutable audit, GitOps, 30-day pre-election software freeze.
4. Contradictions worth resolving: HSM FIPS 140-2 vs 140-3 (ARCHITECTURE vs expect.md), Linkerd vs Istio (ARCHITECTURE ambiguous, expect.md commits to Linkerd 2), PRD §8.3 module file links (five broken), `MOBILE-INTEGRATION-NOTES.md` referenced but absent from the corpus.

**Decision recorded**: treat the requirements as living, not final. Do not reshape the scaffold until the stack is agreed. Discussion mode.

---

### Phase 3 — Stack ideation with an OSS-first constraint

**User constraint stated**: "Use as much open source software as possible to obviate any unnecessary licensing/integration costs and complexity. Use the highest available level of tooling + software. Clean slugs."

**Method**: Presented a tier-by-tier stack proposal covering languages, data, platform, build, identity, crypto, and flagged seven decisions for user input.

**Notable OSS-hygiene picks (proposed at this stage)**:

- **OpenSearch** replaces Elasticsearch (SSPL avoided).
- **Valkey 8** replaces Redis 7.4+ (BSL avoided).
- **OpenBao** replaces HashiCorp Vault (BSL avoided).
- **Apache Kafka** kept over Redpanda Community (Redpanda's Source-Available BSL avoided).
- **Cilium** proposed as a candidate to replace Linkerd 2, consolidating CNI + mesh + network policy + observability.
- **Gitea/Forgejo + Woodpecker** proposed over GitLab CE for a lighter forge.
- **Talos Linux** or Ubuntu 24.04 proposed as node OS options.

**Initial recommendation on languages**: keep Python 3.12 async as house default (per `pjs/CLAUDE.md`), extract Rust for latency- and security-critical services (HSM broker, cert validator, transparency log server, results ingest, ABIS orchestration).

---

### Phase 4 — Language reversal: Rust-default, Python fallback

**User challenge**: "I am not averse to rust as a default and python as a fallback, does that change the analysis above? What will give us the fastest most secure development?"

**Analysis rendered**:

- Distinguished time-to-first-prototype (Python wins) from velocity at month 18 with 5 teams and 100 services (Rust wins decisively).
- Named three categories where pure-Rust is a net loss: (1) ML/data-science (M17, M26, M28, M21 SHANGRLA — Python owns the reference implementations), (2) vendor-SDK-bound integrations (IPRS, KRA, telco USSD/SMS gateways — SDKs ship in Java/Python), (3) ops glue and one-off tools.
- Named where Rust-default is straight-win: security-boundary services (HSM broker, cert validator, PKI issuer, transparency log, results ingest, ABIS orchestrator, ceremony orchestrator, audit-log writer, mTLS gateway, kit provisioning API).
- Committed a specific Rust stack: axum + tower + sqlx + tokio + utoipa + tonic + tracing + opentelemetry-rust + Ecto-equivalent absent so sqlx with compile-time query checks + serde + garde + rdkafka + fred + opensearch-rs + cryptoki + oqs + sigstore-rs + cargo-nextest + testcontainers-rs + cargo-fuzz.
- Named the team risk honestly: Kenya's Rust hiring pool is thin; mitigate with 2-3 senior anchors, 6-8 week Rust immersion for existing Python/Java devs, one-stack-one-template discipline, AI-assisted coding.

**User decision**: "Don't push pure rust — be pragmatic — you are going to develop all the software as quickly and as robustly as you can."

**Resolution**: Tiered policy. Rust-default with a small principled exception list (ML/data-science → Python; vendor-SDK-bound adapters → Python or Java behind a Rust facade; ops glue → Python). Locked in as ADR-0002.

---

### Phase 5 — AuthN, AuthZ, Secrets: Keycloak + SpiceDB + OpenBao

**User question**: "For authn + authz should we use keycloak + spicedb + openbao?"

**Analysis rendered**:

- Yes to the trio. All three are Apache 2.0. All three already run on the user's fleet (auth.lindela.io Keycloak+SpiceDB, ml/vault OpenBao). Operational familiarity is not a rounding error over 3 years.
- The trio is not enough. Two adjacent pieces are required: SPIFFE/SPIRE for workload identity (Keycloak issues human tokens, not per-workload SVIDs for 65k kits and 1000+ pods) and OPA+Kyverno for cluster admission and config policy (SpiceDB is domain authz, not admission).
- Rejected alternatives: Zitadel and Authentik (less battle-tested for gov), OpenFGA (SpiceDB has stronger consistency and tooling), HashiCorp Vault (BSL avoided).
- Composed the full request path: Human → Keycloak OIDC → short-lived JWT → Service verifies JWT → SpiceDB CheckPermission → OpenBao secret fetch → audit → response. Service-to-service uses SPIFFE SVIDs over mesh mTLS.
- Committed to a BFF pattern for portals: browsers never hold refresh tokens, never talk to SpiceDB/OpenBao directly.
- Named the SpiceDB schema as a load-bearing artefact requiring one canonical `.zed` per bounded context, versioned in `packages/authz-schema/`, PR-reviewed, golden-tested.

**Locked in as ADR-0007**: Keycloak + SpiceDB + OpenBao + SPIFFE/SPIRE + OPA/Kyverno.

**Left open**: IPRS / e-Citizen federation timing (Phase 1 or Phase 2).

---

### Phase 6 — Portal framework rethink: dropping Next.js

**User challenge**: "Nextjs has been prone to attack lately, we should consider alternative mature, secure options."

**Evidence cited**: CVE-2025-29927 (Next.js middleware auth bypass, critical, March 2025), CVE-2024-46982 (cache poisoning), CVE-2024-34351 (SSRF via server actions), and the broader pattern of middleware being treated as an auth boundary when the framework does not enforce it. Also Vercel-driven direction, weak on-prem story.

**Analysis rendered**:

- The portals are not one category. Three shapes emerged: (1) static-first public read surfaces, (2) internal admin CRUD, (3) real-time operator dashboards.
- Proposed Path A: Astro 5 (static) + Django 5 + HTMX (admin) + SvelteKit 2 (dashboards) + Tauri (desktop) + Flutter (mobile/kit). Three portal frameworks, no new languages beyond the accepted set.
- Rejected alternatives at this stage: Remix (React ecosystem risks, RR7 merger churn), Nuxt 3 (fine but no decisive win), Phoenix LiveView (dismissed on hiring/language-count grounds), SolidStart (dismissed as ecosystem-thin).

---

### Phase 7 — Re-examining LiveView and SolidJS

**User challenge**: "Examine Phoenix LiveView (Elixir, Apache 2.0) and SolidJS again, why are we not choosing these if they are technically excellent?"

**Admission**: The Phase 6 dismissals were too fast. Reasons given were secondary (hiring, ecosystem breadth) rather than technical.

**Honest re-analysis of Phoenix LiveView**:

- BEAM's per-connection process model = one process per browser session, supervised, crash-only. Under adversarial pressure this is worth more than any WAF rule; a hostile session crashes itself, blast radius is one process.
- Push semantics native. LiveView is designed for millions of live connections (Discord ~5M concurrent per node). Node/Python frameworks need heavy engineering to approximate this.
- Server-authoritative state. No JWT-in-browser, no localStorage exfiltration risk, no XSS-to-session-takeover chain. Compromise a browser, you get one session.
- Ecto: arguably the best-designed database library in any language; compile-time query composition; SQL injection effectively impossible if used correctly.
- Broadway (backpressure-aware pipelines) and Oban (background jobs) exceed Celery/Sidekiq for our workload shape.
- Rustler NIFs let us wrap our Rust crypto/HSM/verifier crates natively in Elixir with near-zero overhead.
- CVE track record for the Phoenix/Elixir stack is remarkably quiet vs the React/Next ecosystem.
- Cost: adds a fourth language (Rust + Python + TS + Elixir), needs 2-3 senior Elixir anchors + 4-6 upskilled devs, BEAM ops model needs a 2-3 month ramp-up.

**Honest re-analysis of SolidJS + SolidStart**:

- Fine-grained reactivity via signals is the technically most efficient reactive core in JS; Solid literally invented the model that Angular/Preact/Vue are now copying.
- Bundle size smaller than SvelteKit.
- JSX syntax means React devs migrate in a day.
- SolidStart v1 (April 2024) is younger than SvelteKit; framework maturity is the remaining honest concern for a 3-year contract.
- Tie-breakers: SvelteKit wins on maturity and prescriptive discipline; SolidStart wins on bundle size and React-dev migration; roughly equal on real-time ergonomics.

**Three revised paths presented**:

- **Path A**: original — Astro + Django + SvelteKit + Tauri + Flutter.
- **Path B**: LiveView-anchored — Astro + Django + LiveView (for M11, M13, M15, M16, M19, M23, M26) + Tauri + Flutter. Adds Elixir.
- **Path C**: LiveView-maximal — LiveView for most web, Astro only for pure-static, Tauri, Flutter. Drops Django and SvelteKit.

**Recommendation given**: Path B. LiveView owns the surfaces it is best-in-class for; Django keeps form-heavy admin in the boring, secure, Python-tier stack; Astro keeps pure-static simple.

---

### Phase 8 — Robustness escalation

**User constraint escalation**: "Robustness, reliability, performance and security are the key issues we need to address. There will be millions of viewers on election day — our systems will be challenged from every quarter."

**Framework decision confirmed**: Path B, with LiveView owning the exposed surfaces. Under this constraint LiveView is not a preference; it is the right substrate for the surfaces where the storm lands.

**Named the load shape**: Kenya has ~55M people; plausibly 5-10M concurrent viewers on election night. Threat surface includes L3/L4 volumetric DDoS, L7 application-layer DDoS, credential stuffing on admin/observer/agent portals, exploit attempts by state actors, insider threat, supply-chain compromise, and disinformation attacking system credibility.

**Twelve areas of architectural response committed**:

1. Edge tier absorbing 95% of load: multi-tier CDN inside Kenya, multiple ISP POPs (Safaricom, Airtel, Jamii, iColo, Konza), anycast + BGP, origin shield with nginx + Varnish, signed static tarballs, static-first for everything cacheable, QUIC/HTTP3 at the edge.
2. DDoS defence in layers: upstream ISP scrubbing, Coraza WAF, Anubis PoW, per-IP/per-token/per-endpoint rate limiting, slow-loris timeouts, DNSSEC + CAA + multi-provider DNS, TLS 1.3 + HSTS + DANE + ECH.
3. Graceful degradation everywhere: normal / degraded / read-only-lockdown modes, feature-flagged, one-command switchable, documented per module, rehearsed monthly.
4. Idempotency mandatory on every write API.
5. Backpressure and bulkheading: bounded upstream calls, bounded queues, isolated pools, circuit breakers on every external call.
6. Data tier under storm: read replicas per site, PgBouncer transaction pooling, statement timeouts, materialised views, ClickHouse offloads audit, Redis hot counters.
7. Detection: eBPF host sensors (Falco, Tetragon), Wazuh SIEM, honeypots, canary tokens, Certificate Transparency monitoring, brand/typosquat monitoring.
8. Chaos engineering rehearsed weekly.
9. Load testing at 5x expected peak.
10. Supply chain: cosign, SLSA L3, vendored dependencies, air-gapped registries, Renovate in a separate environment.
11. Public verifiability as active defence: on-chain/Gazette-anchored Merkle roots, reproducibility bundle.
12. Incident response with named humans, pre-drafted legal-approved comms, quarterly tabletops, external red team 8 weeks before election.

**Budget conversation flagged**: multi-ISP peering agreements, external red team, 24/7 SOC during election window, four language teams — none of this is optional under the stated constraint.

---

### Phase 9 — Public API surface strategy

**User question**: "How will we expose APIs to the public? from django?"

**Answer**: No, not from Django. Django serves internal admin only. Public APIs are a separate tier with a different technology mix and different threat model.

**Five-tier public API surface committed**:

- **Tier 1** — Signed static snapshots (~90% of load). Rust producer → cosign-signed JSON/CBOR/GeoJSON tarballs → MinIO → nginx+Varnish → ISP edges.
- **Tier 2** — Live per-user reads. Rust axum + utoipa behind Envoy Gateway + WAF.
- **Tier 3** — Accredited / authenticated APIs. Rust axum with Keycloak JWT + SpiceDB check + audit log.
- **Tier 4** — Bulk data feeds. Nightly signed Parquet+CSV to MinIO with checksum + Merkle proof.
- **Tier 5** — Real-time streams. SSE for unauthenticated public, WebSockets for authenticated, Kafka spine, LiveView Channels for the portals.

**Conventions**: OpenAPI 3.1 spec-first, generated Dart/TS/Python/Rust clients, signed responses with cosign signature + Merkle inclusion proof + snapshot ID in headers, strong ETag + Cache-Control, RFC 9457 problem+json errors, mandatory Idempotency-Key on writes, JSON default + CBOR alternate for mobile, URL-path versioning.

**Explicitly rejected**: DRF/Django Ninja for public, GraphQL for public (malicious query construction, N+1 at scale), live-DB-query API for bulk consumers, LiveView-exposed HTTP JSON for machines.

**Signing model**: every consensus-relevant response bears cosign signature + Merkle inclusion proof + snapshot ID reference; client SDKs verify automatically.

**User decisions**: (1) keep the 5-tier surface, (2) everything signed and verifiable, (3) CBOR alongside JSON for mobile. Locked in as ADR-0011.

---

### Phase 10 — Data & schema governance

**Decision to design A (data & schema) and B (event bus) before writing ADRs**: schema governance is load-bearing across every service; retrofitting across 69 services later is prohibitive.

**Six areas addressed**:

1. **Database topology** — tiered proposal: 7-8 dedicated PG clusters for consensus-critical event-sourced modules, 2-3 shared clusters (schema-per-service) for operational transactional, one ClickHouse for analytics, one OpenSearch for search, Valkey per site.
2. **Event sourcing scope** — proposed 8 modules event-sourced (M02, M06, M08/M09/M10, M20, M21, M27, M51, M63), rest state + immutable audit shadow.
3. **Event store shape** — PostgreSQL-backed event table with prev_hash chain, HLC, UUIDv7 keys, snapshots, outbox.
4. **Schema evolution** — expand-migrate-contract, zero-downtime required, no breaking DDL, tested rollback per migration, 30-day pre-election freeze, CI gates via atlas/squawk.
5. **Cross-service consistency** — transactional outbox pattern, Debezium CDC, exactly-once effective delivery with idempotent consumers, no dual-write, no 2PC. Temporal only for the small number of genuinely orchestrated workflows.
6. **PII classification & retention** — five tiers (A voter PII, B voter operational, C public, D operational metadata, E ephemeral) with encryption / access / retention per tier. Initially proposed cryptographic erasure for right-to-erasure.
7. **Transparency log** — CT-style Merkle log per election, HSM-signed Signed Tree Heads, split-view prevention via witnesses, reproducibility-bundle-verifiable.

**Six A-decisions posed to user**.

---

### Phase 11 — Write-scale reality check

**User challenge**: "Think millions of writes on election day into the db."

**First-principles estimate committed to shared understanding**:

- ~22M registered voters, ~46k polling stations, ~65% turnout = ~14M voters cast ballots.
- Per-voter events: 4-8 depending on how we count (check-in, turnout, ballot issued × ~7 races, spoilage, cast × 7 races).
- Non-voter events: kit heartbeats, RTS Form 34A/B captures, party agent attestations, observer reports, dispatch pings, session/auth, real-time results updates, verifier query audit.
- **Total: 400-600M write events per election day.**
- **Sustained rates**: daily average ~15k/sec, peak hour 30-50k/sec, extreme burst 80-150k/sec for 2-3 hours in the evening RTS submission window.

**Real bottlenecks identified**:

- Not PostgreSQL. PG 17 on serious hardware handles 30-50k writes/sec per primary; batched, partitioned, async-committed it goes higher.
- Not Kafka. Kafka does 10 GB/sec routinely; our peak is 100 MB/sec.
- **HSM signing throughput**: Luna 7 at ~5k EC sigs/sec means per-event HSM signing is architecturally impossible; must batch via Merkle.
- **Transparency log sequencer**: Trillian benchmarks 15-30k leaves/sec; naive impl dies at 50k+.
- Cross-site sync replication needs dedicated 10GbE+ links; measured and rehearsed, not assumed.

**Seven adjustments proposed**:

1. Shard T1-voter cluster with Citus (Apache 2.0, PG extension), distribute by county_id (47 shards).
2. HSM signs Signed Tree Heads only; producers sign events with SPIFFE-issued keys (short-lived, software Ed25519, per-workload).
3. Transparency log sharded by domain (5-7 logs) with a merged super-root every 60s.
4. Producer-side durable outbox — kits never block on log availability; events replay from Kafka when log recovers.
5. PostgreSQL tuning specifics for T1 write path (shared_buffers, checkpoint, WAL, fillfactor, BRIN indexes, PgBouncer transaction mode).
6. Load test as first-class deliverable — 46k virtual kits, bimodal traffic curve, 5x peak simulation, weekly from month 2.
7. Election-day operational mode — migrations frozen, autovacuum aggressive, statement timeouts tightened, circuit breakers hair-triggered, no admin queries on T1 primaries.

**Four write-scale decisions posed to user**.

---

### Phase 12 — A-decisions consolidated

**User answers** (batch):

1. Consolidate T1 to 3 clusters. → Restructured to T1-voter (turnout, register, custody), T1-consensus (results, credentials, transparency super-root), T1-audit (RLA, cross-service audit, per-domain logs). ×3 sites = 9 cluster instances, template-managed.
2. Narrower event-sourcing scope. → Trimmed to 4 modules: M08/M09/M10 RTS, M20 custody, M63 ballot serial, M51 declaration. M02, M06, M21, M27 demoted to state + immutable audit shadow. Cascading benefits: fewer projections, fewer replay operations, simpler day-1 delivery.
3. Cryptographic erasure — user pushback: "when will this ever be necessary — should we not just have soft delete?"
4. Transparency log cadence: 1s hot / 60s default / 5min batch. → Confirmed.
5. Third-party witnesses: defer. → Internal IAB witness stands as day-1 model; external + on-chain witnesses deferred to a later diplomatic conversation.
6. Synthetic voter register generator as first-class deliverable: yes. → Built week 1, all non-prod environments use synthetic data only.

**Honest response to #3**: user was right to push back. Cryptographic erasure was over-engineering for this domain.

Analysis: DPA 2019 §40 (right to erasure) has explicit exceptions for legal obligation, public interest, and legal claims; Elections Act §32 statutorily requires 15-year retention of the Tier A data most people picture as "PII to erase." Where erasure requests actually land — grievance intake, voter-ed CMS accounts, helpdesk logs, non-accredited media applications, withdrawn VPRS/diaspora registrations — soft-delete + scheduled purge + rotating backup encryption keys is legally sufficient and regulator-defensible.

**Revised policy**: soft delete + scheduled purge is default; cryptographic erasure infrastructure dropped from scope. Saves per-subject DEK management, per-column encryption complexity, months of build time. Retained option: if a specific system later demands stronger erasure guarantees, layer it on then.

---

### Phase 14 — RFP-alignment reconciliation (post-session)

**Trigger**: user reported the `sw/` corpus had been updated (~2-3x growth) and asked for a conflict review against the ADRs just committed.

**Method**: forked reader compared updated `sw/` against ADRs 0001-0018 and produced a conflict inventory. A second forked reader then read the actual RFP source (`work_docs/01-Projects/IEBC/tender.txt` + `PRE_BID_CLARIFICATIONS.md`) to split conflicts into RFP-binding vs Datacraft-drafted.

**Key finding**: the RFP mandates almost nothing at framework level. Where the spec doc pins Next.js / Cedar / Vault / Linkerd / Kotlin / Avro, those are Datacraft-drafted, not RFP-derived. Per user policy ("unless derived from the actual RFP, we can make the choice that is optimal"), the defensive posture is defend our optimal picks; concede only RFP-binding constraints and Datacraft-drafted extensions that are genuinely sound.

**RFP-binding items that changed our plan**:
- Escrow explicitly NOT required (tender.txt L9556, L11377). Dropped from scaffold.
- Two IEBC-provided DR sites (primary + mirror), not three. Adjusted ADR-0004/0012/0022.
- Ninety-day online log retention + permanent archive (tender.txt Sec V + PRE_BID D13). Adjusted ADR-0015 hot retention.
- Sixty thousand concurrent RTS sessions (tender.txt Sec V). Added to ADR-0018 SLA.
- Kotlin+Compose for kits is NOT RFP-derived; Flutter reversal confirmed since RFP-binding ~14,100 legacy Android devices argue for Flutter's better legacy support.

**Decisions locked in this phase**:
- Rust default holds.
- Ubuntu 24.04 LTS holds.
- Kong Gateway OSS accepted.
- Cilium defended over Linkerd 2.
- SpiceDB defended over Cedar.
- OpenBao / Valkey / Karapace defended over Vault / Redis / Confluent SR.
- Path B portals defended over Next.js 14.
- Kenyan-sovereign edge defended over Cloudflare / Akamai (spec self-contradicts anyway).
- Two-site DR posture (align to RFP).
- Flutter for both kits and mobile.

**Artefacts produced**:
- **ADR-0019** (regional edge topology, 5 clusters).
- **ADR-0020** (service topology, 18 K8s services grouping 69 modules).
- **ADR-0021** (mobile security posture, Play Integrity + App Attest + RASP).
- **ADR-0022** (release + DR, blue-green + 15-min rollback + RTO/RPO SLAs).
- **ADR-0023** (RFP-alignment reconciliation, amendments to 0004/0005/0006/0007/0008/0012/0015/0016/0017/0018).
- **Change-request pack** (`docs/spec-alignment/`): CR-001 OSS hygiene, CR-002 portal stack, CR-003 sovereign edge, CR-005 SpiceDB vs Cedar, CR-006 Cilium vs Linkerd, CR-007 event bus JSON vs Avro. (CR-004 dropped — user policy holds Rust-default without renegotiation.)
- **Blockers doc** (`docs/iebc-clarifications-outstanding.md and docs/datacraft-internal-backlog.md`): scope gaps (M29-M69 not costed, no OAT/FAT for M35-M69, no API-CATALOG for M29-M69), legal/retention questions (voter register 5y vs 15y, digital signature standard pending, PRE_BID D9-D13 outstanding), housekeeping (kit language, HSM level, CI in air-gap, Cloudflare vs sovereignty, escrow, three-vs-two site, OPA/Cedar ambiguity).

**Left open**: three DC hardware provisioning question (PRE_BID C1), independent security certifier (PRE_BID D11), ODPC registration timing (PRE_BID D10), Kenya data residency for production (PRE_BID D9), digital signature standard for Form 34A (PRE_BID C13).

---

### Phase 13 — Simplification pass

**User instruction**: "Make the choices that are defensible, simplest and can function under extreme pressure and load. Minimize complexity."

**Four write-scale decisions taken decisively**:

1. **No Citus day 1.** Modern PG 17 on serious hardware with declarative hourly partitioning + batched inserts + async commit + PgBouncer transaction mode holds 50-100k writes/sec (Instagram, Discord, Notion at this scale on vanilla PG). Citus adds coordinator layer, cross-shard transaction semantics, extension-version coupling, rebalancing operations — each a class of incident that can't happen without it. Ship vanilla PG; turn on Citus only if 5x-load tests prove necessity.
2. **HSM signs Signed Tree Heads only; producers sign leaves with SPIFFE keys.** Only viable pattern at scale; matches Trillian / Sigstore Rekor design. Locked.
3. **Single monolithic transparency log with domain tags on leaves.** Simpler to operate, simpler to explain to auditors, simpler to verify. Internal partitioning of underlying storage for headroom; external contract stays "one canonical log."
4. **Load-test SLA: 50k writes/sec sustained for 4 hours per cluster, P99 25ms end-to-end (kit-to-ACK).** Non-negotiable release gate.

**Complexity cuts applied across the earlier design**:

- **Temporal removed from day-1 stack.** Ceremony workflows can be state machines in the owning service backed by PG row + Oban / arq / tokio task; adding Temporal means running Cassandra/PG for Temporal plus the server plus operator training for 3-5 workflows.
- **ClickHouse deferred to Phase 2.** PostgreSQL 17 with hourly partitions + BRIN indexes + read replicas + materialised views handles analytics well past election day. ClickHouse pays off past 100M hot analytics rows with sub-second aggregation requirements — a Phase 2 problem. Modules that later demand columnar performance can request it in their module ADR.
- **OpenSearch confined to modules that actually search.** M07 register inspection, M18 boundary search, M22 candidate search, M52 gazette search need full-text + fuzzy + geo. Every other module uses Postgres full-text (pg_trgm + tsvector). One shared OpenSearch cluster for the four; no OpenSearch anywhere else.

---

## 4. Locked-in stack (end of session)

| layer | pick |
|---|---|
| Backend default | Rust (axum + sqlx + tokio + utoipa + tonic + tracing + serde + garde + rdkafka + fred + opensearch-rs + cryptoki + oqs + sigstore-rs) |
| Backend fallback | Python 3.12 async (FastAPI + Pydantic v2 + SQLAlchemy 2 + uv + ruff + pyright) for ML/adapters/glue |
| Real-time / high-fanout web | Phoenix LiveView (Elixir + Ecto + Broadway + Oban + Rustler for Rust interop) |
| Admin / backoffice web | Django 5 + HTMX + Alpine |
| Static public web | Astro 5 + Svelte islands |
| Operator desktop | Tauri 2 (Rust + web UI) |
| Kits + mobile | Flutter/Dart |
| OLTP | PostgreSQL 17, three T1 clusters per site (voter / consensus / audit), single-primary, hourly-partitioned hot tables, Patroni HA, pgBackRest → MinIO, PgBouncer transaction mode |
| Analytics | PostgreSQL read replicas + materialised views day 1; ClickHouse deferred to Phase 2 |
| Search | OpenSearch for four named modules only; Postgres pg_trgm + tsvector everywhere else |
| Cache / hot state | Valkey per site |
| Event bus | Apache Kafka 3.7 Kraft mode, no ZooKeeper |
| CDC | Debezium on every T1 cluster; transactional outbox is only path to Kafka |
| Object store | MinIO |
| Workflow orchestration | Oban / arq / tokio state machines day 1; Temporal only if a module ADR justifies |
| Container platform | Kubernetes 1.31, kubeadm-provisioned, one cluster per site |
| Node OS | Ubuntu 24.04 LTS (ops-team familiarity over Talos's image-based elegance for 3-year contract) |
| CNI + mesh | Cilium (CNI + service mesh + network policy + Hubble observability); no Linkerd |
| Ingress | Envoy Gateway |
| Human identity | Keycloak |
| Workload identity | SPIFFE/SPIRE, HSM-anchored root |
| Authorization | SpiceDB (domain ReBAC), OPA + Kyverno (admission) |
| Secrets | OpenBao (HSM-sealed) |
| Crypto | HSM PKCS#11/KMIP, TLS 1.3 + PQ hybrid (ML-KEM + X25519), AES-256-GCM at rest, cosign for artifact signing |
| GitOps | ArgoCD with ApplicationSets |
| Container registry | Harbor with cosign verification enforced |
| Supply chain | Sigstore/cosign, in-toto, SLSA L3 attestations, Syft/Grype/Trivy, cargo-vet, `atlas`/`squawk` for schema |
| Observability | OpenTelemetry Collector → Prometheus + Loki + Tempo + Grafana; Vector as node agent; Mimir for long-retention metrics |
| Detection | Falco + Tetragon (eBPF), Wazuh SIEM, Sigma rules, honeypots, canary tokens |
| Reproducible builds | Nix flakes for every service, every image |
| Language build tools | uv (Python), cargo (Rust), pnpm (TS), mix (Elixir), `just` at root |
| Forge + CI | Gitea/Forgejo + Woodpecker CI + Nexus OSS (air-gapped dep mirror) |
| Storage | Rook-Ceph (block/object/file); MinIO layered on top for S3 API |
| Backup | Velero + Kopia + pgBackRest, WAL streaming to MinIO, PITR to any point in 30 days |
| Edge | nginx + Varnish + Coraza WAF + Anubis PoW; multi-ISP peering (Safaricom, Airtel, Jamii, iColo, Konza), anycast, DNSSEC + CAA + multi-provider DNS |
| Retention | Soft-delete + scheduled purge + rotating backup encryption keys; no cryptographic erasure infrastructure |
| Transparency log | Single monolithic Rust service with domain-tagged leaves, HSM-signed STHs at 1s hot / 60s default / 5min batch cadence |

**Portal allocation** (Path B):

- **Astro**: M04 VPRS, M07 BVV, M11 public portal, M18 boundary viewer, M22 finance viewer, M37 voter-ed CMS, M52 gazette, M64 voter-list inspection, plus other static pages.
- **Django**: M12 CRMS + party portal, M42/M43 accreditation admin, M45 party mgmt, M35/M36 voter register admin, internal admin surfaces.
- **LiveView**: M11 dynamic portions (with Astro shell), M15 IAB dashboard, M19 queue transparency, M23 grievance channels, M26 anomaly triage; M16 command centre runs LiveView inside a Tauri webview.
- **Tauri**: M09 RO tally, M16 Command Centre, M21 audit laptops.
- **Flutter**: M01 BVR device, M05 BVI device, M08 RTS device, M13 public verifier (all channels), M24 diaspora, M25 special-voter, M27 intent, M29 mobile citizen verify, M30 party agent, M31 poll clerk, M32 field support, M33 dispatch, M34 observer.

**Event-sourced modules (only these four)**: M08/M09/M10 RTS, M20 chain of custody, M63 ballot serial chain, M51 declaration.

**Database cluster shape**: T1-voter (turnout, register, custody), T1-consensus (results, credentials, transparency super-root), T1-audit (RLA, cross-service audit, per-domain logs). ×3 sites = 9 cluster instances.

---

## 5. Open decisions

Named so we don't forget:

1. **HSM FIPS level** — 140-2 L3 vs 140-3 L3+. Procurement question. Placeholder in ADR-0005; code against PKCS#11 abstractly so the vendor is swappable.
2. **IPRS / e-Citizen federation timing** — Phase 1 (day 1 for citizen-facing modules M04, M07, M13, M29, M35, M42, M43) or Phase 2 (mock IdP until federation lands).
3. **Elixir hiring plan** — 2-3 senior anchors + 4-6 upskilled devs; remote OK; timing depends on HR alignment.
4. **External red team + 24/7 SOC funding** — flagged as budget-shape questions.
5. **Third-party witness engagement for transparency log** — external observers + on-chain anchoring diplomatic engagement.
6. **Contradictions in source spec to reconcile with client** — module count drift across README/expect/PRD, PRD.md §8.3 broken filename links, missing `MOBILE-INTEGRATION-NOTES.md`, strategy sections §XIV/§XV/§XVIII/§XIX referenced but not in the sw corpus.

---

## 6. ADR roadmap

To be written as a batch, cross-referenced, each 200-400 words with Context / Decision / Consequences / Alternatives / Related:

| ADR | Subject | Status |
|---|---|---|
| 0001 | Monorepo layout | Accepted (committed at `4d7e5c6`) |
| 0002 | Languages: Rust default, Python fallback, Elixir for LiveView, Dart for kits+mobile, TS for Astro | Pending write |
| 0003 | Data: PostgreSQL 17 primary, OpenSearch scoped, Valkey, Kafka Kraft, MinIO, ClickHouse deferred | Pending write |
| 0004 | Platform: K8s 1.31 + Cilium + SPIFFE/SPIRE + ArgoCD + Harbor + cosign + Nix + Ubuntu 24.04 | Pending write |
| 0005 | Crypto and HSM: PKCS#11/KMIP, TLS 1.3 + PQ hybrid, M-of-N key ceremony, FIPS level deferred | Pending write |
| 0006 | Build and supply chain: Nix flakes, Gitea + Woodpecker + Nexus, cosign + SLSA L3, air-gapped mirror | Pending write |
| 0007 | AuthN + AuthZ + Secrets: Keycloak + SpiceDB + OpenBao + SPIFFE/SPIRE + OPA/Kyverno, BFF for portals | Pending write |
| 0008 | Portal stack: Astro + Django + LiveView + Tauri + Flutter (Path B), with Next.js CVE history as trigger | Pending write |
| 0009 | Resilience posture: degradation modes, circuit breakers, backpressure, idempotency, load shedding | Pending write |
| 0010 | Edge and DDoS: multi-ISP peering, Coraza WAF, Anubis, rate limits, DNSSEC + CAA | Pending write |
| 0011 | Public API surface: 5-tier, everything signed and verifiable, CBOR + JSON, RFC 9457 errors | Pending write |
| 0012 | Data topology: 3 T1 clusters (voter / consensus / audit), no Citus day 1, single primary + partitioning | Pending write |
| 0013 | Event-sourcing boundaries: only M08/M09/M10, M20, M63, M51; rest state + audit shadow | Pending write |
| 0014 | Schema evolution: expand-migrate-contract, zero-downtime, tested rollback, 30-day freeze | Pending write |
| 0015 | Cross-service consistency: transactional outbox + Debezium; no dual-write; no 2PC; choreography default | Pending write |
| 0016 | PII classification + retention: five tiers, soft-delete + scheduled purge, rotating backup keys, no cryptographic erasure | Pending write |
| 0017 | Transparency log: single monolithic, domain-tagged leaves, HSM-signed STH at 1s/60s/5min, IAB witness day 1 | Pending write |
| 0018 | Write-path performance targets: 50k/sec sustained per cluster, P99 25ms end-to-end, 5x load buffer | Pending write |

---

## 7. What comes next

1. **Write ADRs 0002-0018** as one pass with cross-references and citations to the source specs (`sw/ARCHITECTURE.md`, `sw/PRD.md`, `sw/expect.md`, individual module specs).
2. **Reshape the scaffold**:
    - Rename `modules/mod-01..mod-30` to clean slugs matching the actual module identities from the spec (~69 modules).
    - Drop the automatic `{shared,server,desktop,mobile}` quadrant; use on-demand surfaces per module.
    - Add top-level directories: `services/`, `platform/`, `integrations/`, `operations/`, `compliance/`, `edge/`, `chaos/`, `loadtest/`, `detection/`, `transparency-log/`, `reproducibility/`, `escrow/`, `air-gapped-build/`.
    - Populate `packages/` with shared crate/lib skeletons: `iebc-auth-{rs,py,ex}`, `iebc-secrets-{rs,py,ex}`, `iebc-eventstore-{rs,py,ex}`, `iebc-outbox-{rs,py,ex}`, `iebc-pii-{rs,py,ex}`, `iebc-hlc-{rs,py,ex}`, `iebc-resilience-{rs,py,ex}`, `iebc-observability-{rs,py,ex}`, `iebc-loadshedding-{rs,py,ex}`, `iebc-api-server-rs`, `iebc-web-py`, `iebc-web-svelte`, `iebc-web-astro`, `packages/authz-schema`, `packages/data-dictionary`, `packages/event-schemas`, `packages/openapi-contracts`, `packages/api-clients/{dart,ts,py,rs}`, `packages/verifier-rs`, `packages/kit-sdk`, `packages/design-system`.
    - Update `README.md`, `docs/README.md`, `docs/modules/README.md`, `docs/adr/README.md` index.
    - Add runbook skeletons under `docs/runbooks/`: pg-cluster-failover, backup-restore-drill, event-replay, transparency-log-witness-cross-check, break-glass, per-module degradation runbooks.
3. **Design B (event bus conventions on Kafka)**: envelope (CloudEvents 1.0 canonical JSON for signing, MessagePack/CBOR for volume topics), producer signing with SPIFFE keys, topic naming and partitioning, retention by class (event-source infinite → MinIO archive, operational 30d, analytics 7d, DLQ 90d), Buf Schema Registry, consumer patterns (idempotent default; exactly-once for tally/results via Kafka transactions + outbox), replay policy, LiveView fan-out backpressure.
4. **Design environments / CI/CD / release train**: dev, staging, pre-prod, prod, election-day-frozen-prod; release trains; blue/green + canary; 30-day pre-election freeze; migration approval gates.
5. **Write the first real service**: the transparency log itself. Foundational, self-contained, exercises the Rust stack, HSM integration, event-source shape, and the OpenAPI + signed-response conventions.

---

## 8. Working principles reinforced during the session

Captured because they shaped multiple decisions and should govern future ones:

- **OSS-first with strict licence hygiene**: Apache 2.0 / MIT / BSD / MPL 2.0 preferred; SSPL, BSL, and Source-Available avoided (Elasticsearch → OpenSearch, Redis → Valkey, Vault → OpenBao, Redpanda → Kafka).
- **Rust-default for backend with a small principled exception list**: velocity at month 18 with 5 teams beats velocity at week 1 with 1 team; memory-safety CVE class is genuinely eliminated for the security-boundary services; pragmatic Python fallback for ML, vendor SDKs, and ops glue.
- **Simplify aggressively**: prefer vanilla PG over Citus, prefer single log over sharded, prefer state + audit table over event sourcing wherever the event log is not itself the deliverable, prefer soft-delete + purge over cryptographic erasure, prefer Oban / arq over Temporal, defer ClickHouse until Postgres cannot cope, scope OpenSearch to modules that actually search.
- **Design for the storm, rehearse for it**: chaos engineering weekly, load tests at 5x peak weekly, blameless tabletop exercises quarterly, external red team 8 weeks before election, degraded modes documented and rehearsed per module.
- **Everything consensus-relevant is signed and verifiable**: cosign signatures, Merkle inclusion proofs, HSM-anchored STHs, reproducibility bundle so any third party can rebuild the answer from primary evidence.
- **Trust boundaries deserve dedicated infrastructure**: T1 clusters isolated by domain (voter / consensus / audit), transparency log is its own service, HSM interactions confined to a broker, secrets never in git or environment variables.
- **One canonical source per fact**: Rust services hold truth; LiveView renders it; Django admins it; snapshots publish it; mobile consumes it. No parallel implementations of business logic.
- **Client operational familiarity is not a rounding error**: Keycloak + SpiceDB + OpenBao + MinIO chosen partly because they already run on the client's fleet; Ubuntu 24.04 over Talos because ops-team comfort matters over a 3-year contract.
- **Requirements are living, not final**: the 12 → 28 → 34 → 53 → 69-and-growing module count is a feature of the tender-drafting process, not a defect; the design accommodates ~100 modules without structural change.
