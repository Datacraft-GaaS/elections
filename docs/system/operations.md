# Operations Reference

Operational reference for building, running, testing, deploying, and recovering the
eel election platform. Grounded in code with citations; ground truth documents:
`docs/architecture/cross-cutting/deployment-topology.md`, ADRs 0004/0009/0017/0019/
0020/0022 (index: `docs/adr/README.md`), and `verification/README.md`.

---

## 1. Build System

### 1.1 Cargo workspace

Root manifest `Cargo.toml` defines one resolver-2 workspace:

- Members (globs so new crates are picked up automatically, `Cargo.toml:3-10`):
  `packages/eel-*-rs`, `packages/verifier-rs`, `packages/service-template-rs`,
  `services/*/crates/*`, `services/abis-svc/tools/*`, `tools/*`.
- Excludes (`Cargo.toml:11-30`): placeholder-only tools `tools/softhsm`,
  `tools/spectral`; nested workspaces that own their crates:
  `services/support-admin-svc/crates/{budget,payroll,archive,special-voter,intent}`,
  `services/ai-platform-svc/crates/telemetry-analytics`; plus
  `services/bvr-svc/crates/{bvr-api,bvr-domain,bvr-batch}` and
  `services/rts-svc/crates/rts-integration-tests` per inline comments.
- Shared versions pinned in `[workspace.dependencies]` (`Cargo.toml:38+`); edition
  2021, rust-version 1.83 (`Cargo.toml:33-42`).
- Several services additionally own nested service-level workspaces
  (`services/*/Cargo.toml`); the root justfile enumerates them dynamically
  (`justfile:17-19`).

### 1.2 Elixir mix projects

There is no umbrella mix project; each Elixir app is standalone under a service's
`ex/` directory. Present apps (each with its own `mix.exs`):

| App | Path |
|---|---|
| bvv_web (BVV lookup LiveView) | `services/bvi-svc/ex/bvv_web/mix.exs` |
| portal_live | `services/portal-svc/ex/portal_live/mix.exs` |
| ecc_live, iab_live | `services/ecc-svc/ex/ecc_live/mix.exs`, `services/ecc-svc/ex/iab_live/mix.exs` |
| queue_map | `services/queue-svc/ex/queue_map/mix.exs` |
| observer_workspace | `services/governance-svc/ex/observer_workspace/mix.exs` |
| speech_analyst | `services/ai-platform-svc/ex/speech_analyst/mix.exs` |
| hello_app | `services/hello-svc/ex/hello_app/mix.exs` |

Shared Elixir libraries live as their own mix projects in
`packages/eel-{secrets,eventstore,loadshedding,events,hlc,auth,pii}-ex/`.

### 1.3 Python projects

Per-package `pyproject.toml` + uv (e.g. `packages/eel-pii-py/pyproject.toml`,
`packages/eel-secrets-py/pyproject.toml`, `tools/pii-in-events/pyproject.toml` with
`uv.lock`). The root justfile drives Python tests via
`uv run pytest -q tests/ci` in every project containing a `tests/ci` directory
(`justfile:73-78`). Woodpecker pipelines use `uv pip install -e .` /
`uv run` for tooling (`.woodpecker/pii-lint.yaml`).

### 1.4 Reproducibility pipeline

M14 make-reproduce toolkit (`reproducibility/make-reproduce/`), purpose and flow in
`reproducibility/README.md`: fetch release bundle -> Nix rebuild from pinned flake ->
diffoscope compare -> tally replay -> transparency-log signature verification.

Makefile targets (`reproducibility/make-reproduce/README.md`):

| Target | Effect |
|---|---|
| `make reproduce BUNDLE=<path>` | full pipeline |
| `make verify-images IMAGES="..."` | rebuild named Nix attrs; compare sha256 |
| `make replay-tally BUNDLE=<path>` | replay `events.jsonl`, re-derive Merkle root |
| `make diff-artefact A=<p> B=<p>` | diffoscope HTML+JSON report |
| `make clean-repro` | drop `.repro-reports/` |

Supporting assets: driver scripts `scripts/reproduce.sh`, `replay-tally.sh`,
`multi-arch-verify.sh`; Rust replay engine `bin/replay-tally-rs`; per-language Nix
flake fragments under `reproducibility/nix-flakes/`; hash-manifest schema and
validation under `reproducibility/hash-manifests/`. Supply-chain context: ADR-0006.

---

## 2. Running Locally and Testing

### 2.1 Per-stack commands

Root orchestrator is `just` (`justfile`):

| Command | What it runs |
|---|---|
| `just up` / `just down` | local kind cluster + compose stack from `environments/dev/compose/docker-compose.yml` (`justfile:27-45`) |
| `just test-unit` | cargo nextest (fallback `cargo test`) `--lib --bins` across every workspace root, then `uv run pytest tests/ci` per Python project, then `mix test` per Elixir app (`justfile:58-85`) |
| `just test-integration` | `cargo nextest run --test '*'` per workspace root; requires cluster up (`justfile:89-101`) |
| `just verify-all dry|live` | run all Wave-6 harnesses in the chosen mode (`verification/README.md`) |

Direct equivalents: `cargo test -p <crate>`, `mix test --no-color` inside an app dir,
`uv run pytest` inside a Python project.

Minimal end-to-end smoke target: `services/hello-svc/` is the scaffold reference —
`hello-api` exposes a single `GET /` returning a JSON health payload
(`services/hello-svc/crates/hello-api/src/lib.rs:9,14`) and runs standalone with
`cargo run -p hello-api`.

### 2.2 Hermetic test patterns

- Store abstraction: services depend on traits, not concrete DBs — e.g.
  `trait BviStore` (`services/bvi-svc/crates/bvi-api/src/store.rs:51`) with the single
  production impl `PgStore` (`store.rs:177`), letting route handlers be exercised
  against any store.
- In-memory stores where the shape allows: e.g.
  `services/ai-platform-svc/crates/ai-platform-api/src/store/memory.rs`,
  `services/ecc-svc/crates/ecc-api/src/store.rs`.
- testcontainers for integration realism: dev-dependencies in
  `services/bvi-svc/Cargo.toml:50-51` (postgres+kafka modules),
  `services/boundary-svc/Cargo.toml:57-58` (postgres),
  `packages/eel-eventstore-rs/Cargo.toml:34-35`,
  `packages/eel-outbox-rs/Cargo.toml:32-33`.
- Deterministic crypto properties: proptest suites, e.g. constant-time equivalence
  property (`packages/eel-crypto-rs/src/constant_time.rs:80-87`).

### 2.3 Verification suites

`verification/README.md` maps Wave-6 concerns to harnesses: rehearsal (election-day
readiness), dr-drill (RTO/RPO), chaos-runner, load-5x, security (ASVS L3 +
pentest-clean), a11y-i18n (WCAG 2.2 AA + 3 languages), completeness (69 modules),
reports. Every harness supports dry-mode (fixtures only, CI-safe) and live-mode
(CAB-approved, pre-prod) (`verification/README.md`, "Verification modes").

Completeness audit steps (`verification/completeness/steps/`):

| Step | Check |
|---|---|
| `9.1-stub-scan.sh` | `eel-completeness scan --strict`; zero findings outside allowlist; JSON mode consumed here |
| `9.2-surface-inventory.sh` | API surface inventory |
| `9.3-traceability.py` | every `docs/plans/TRACEABILITY.yaml` row has a passing mapped test (parses nextest/pytest/ExUnit/Flutter JUnit output) |
| `9.4-contract-closure.sh` | Pact can-i-deploy + every Karapace schema has producer and consumer tests |
| `9.5-endpoint-closure.py` | endpoint vs contract closure |
| `9.6-runbook-closure.sh` | degradation runbooks exist |
| `9.7-sla-closure.py` | SLO definitions closed |
| `9.8-legal-closure.py` | statutory requirements closed |
| `9.9-report.py` | aggregate report generation |

Reproduce-challenge bounty programme: independent teams recompute the nightly tally
from published bundles and submit within an 8-hour window; automated verifier
`verify-submission.sh` diffs submissions and publishes signed results
(`verification/reproduce-challenge/README.md`; submission format in
`submit-endpoint.md`; schedule in `schedule.yaml`).

Security harnesses: DAST via ZAP baseline + nuclei
(`verification/security/dast/zap-baseline.sh`, `nuclei-templates.yaml`), fuzzing on a
24-hour schedule (`verification/security/fuzz/run-corpus.sh`, `24h-schedule.yaml`),
pentest engagement scope and findings tracking (`verification/security/pentest/`),
evidence collectors `asvs-evidence-collector.sh` and `sast-sca-container.sh`.

---

## 3. Deployment Topology

Two DR sites plus five regional edges (ADR-0019 edge regions, ADR-0020 service
grouping, ADR-0022 release/DR):

- Primary DC (Nairobi): full k8s cluster, T1 PostgreSQL primary, 3-broker Kafka,
  ArgoCD. Mirror DC: hot standby PG, Kafka MirrorMaker 2, full service set. Five edge
  clusters host kit-proximate BVI/RTS write acceptance and CDN origin; field kits run
  offline biometric-edge firmware (`deployment-topology.md:5-48`).
- Replication: PG streaming replication primary->mirror; MirrorMaker 2 for `eel.*`
  topics; ArgoCD ApplicationSets generate one Application per service per site from
  `platform/argocd/applicationsets/` (`deployment-topology.md:32-66`).
- Failover targets (ADR-0022 summary at `deployment-topology.md:50-55`): RTO 15 min;
  RPO zero for T1 event-sourced data, up to 1 h T3 analytics; election-day mode
  freezes migrations; blue-green for stateful services, canary for portal/vprs.
- Edge regions per `docs/adr/0019-edge-topology.md:16-20` and `edge/README.md:8-14`:
  Nairobi, Mombasa, Kisumu, **Eldoret**, Garissa. (The cross-cutting diagram names
  Nakuru instead of Eldoret — see Discrepancies.)
- Per-service placement descriptor `services/<svc>/deploy/service.yaml` declares
  image, sites (empty = all seven sites: primary, mirror, five edges) and strategy
  per site — e.g. `services/bvi-svc/deploy/service.yaml` uses blue-green on
  primary/mirror, rolling on edges, replicas 3/3/2x5.
- Representative kustomize layout: `services/rts-svc/deploy/base/` holds separate
  deployments for rts-api, rts-tally, rts-publisher, projections, each with secretRef
  for DB credentials and readiness/liveness probes on `/health`
  (`rts-api-deployment.yaml:25-44`).
- Environment roots: `environments/{dev,staging,pre-prod,prod}/kustomization.yaml`;
  four-env promotion model in ADR-0022 (dev nightly -> staging weekly green ->
  pre-prod biweekly -> prod manual quorum).
- Election-day-mode gating via the `eel.election-day-mode` ConfigMap checked by Argo
  Rollout analysis templates (`deployment-topology.md:64-66`).

---

## 4. Configuration

### 4.1 Environment variable conventions

Rust services read plain env vars at startup; observed conventions:

| Variable | Used by | Citation |
|---|---|---|
| `HTTP_ADDR` | axum bind address (default `0.0.0.0:8080` or per-binary port) | `services/bvi-svc/crates/bvi-api/src/main.rs:18`; `services/rts-svc/crates/rts-device-api/src/main.rs:14` (8082); rts-ro-api (8083) |
| `DATABASE_URL` | sqlx Postgres URL (required) | `services/bvi-svc/crates/bvi-api/src/main.rs:19` |
| `DB_URL` | same convention in other services (k8s secret `rts-db-secret`, key `url`) | `services/rts-svc/crates/rts-api/src/main.rs:10`; `deploy/base/rts-api-deployment.yaml:26-29` |
| `KAFKA_BROKERS` | Kafka bootstrap | `services/rts-svc/crates/rts-tally/src/main.rs:10` |
| `TLOG_ENDPOINT` | transparency log gRPC endpoint | `services/rts-svc/crates/rts-tally/src/main.rs:11` |
| `PUBLICATION_INTERVAL_SECS` | publisher cadence | `services/rts-svc/crates/rts-publisher/src/main.rs:12` |
| `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` / `_METRICS_` | OTLP exporters (default localhost:4317) | `packages/eel-observability-rs/src/otel.rs:23-26` |
| `EEL_ABIS_SPIRE_REQUIRED` | toggles ABIS SVID enforcement | `services/abis-svc/crates/abis-api/src/middleware.rs:17-19` |
| `EEL_WCAG_STUB_DIR` | WCAG audit CI stub mode | `tools/wcag-audit/README.md` |
| `EEL_PROMETHEUS_URL`, `EEL_*_KUBECONFIG`, `EEL_DR_CONFIRM`, `EEL_RTO/RPO_TARGET_*` | DR tooling | `tools/dr-failover/README.md` |

### 4.2 Secrets

OpenBao (Vault-compatible) per ADR-0007:

- Mounts: KV v2, database dynamic creds, PKI, transit
  (`platform/openbao/mounts/{kv,database,pki,transit}.yaml`).
- Least-privilege policies per service: one HCL file per service in
  `platform/openbao/policies/` (e.g. `bvi-svc.hcl`, `transparency-svc.hcl`), plus
  `admin-jit.hcl` and `break-glass.hcl`.
- Client library `packages/eel-secrets-rs/src/lib.rs:1-8` provides:
  dynamic Postgres credentials `PgDynamicCred` (`dynamic_creds.rs:18`), transit
  encrypt/decrypt `Transit::new/with_mount` (`transit.rs:15,19`), JIT admin creds
  capped at 4 h (`issue_admin_cred`, `admin_creds.rs`), PKI leaf issuance
  (`issue_leaf_cert`, `pki.rs`). Python/Elixir equivalents:
  `packages/eel-secrets-py/`, `packages/eel-secrets-ex/`.
- In-cluster secrets still used for bootstrap values (e.g. `secretKeyRef` for DB URL
  above); break-glass procedure documented in `docs/runbooks/break-glass.md`.

---

## 5. Observability Ops

Package `packages/eel-observability-rs/`:

- `init_otel(&OtelConfig)` wires a tracing registry with: env-filter, OpenTelemetry
  layer exporting spans over OTLP/gRPC to the configured endpoint, a PII-scrubbing
  layer (`PiiScrubberLayer`, `src/scrubber.rs:11`), and JSON-formatted logs via
  `tracing_subscriber::fmt::layer().json()` (`src/otel.rs:66-79`). The returned
  `OtelGuard` shuts the provider down on drop (`otel.rs:36-45`).
- SLO definitions as data: `SloDefinition::new(service, indicator, target)`
  (`src/slo.rs:17`).
- Audit-shadow helper for stateful services: `audit_shadow_sql(table_name)`
  (`src/audit_shadow.rs:8`).

Health endpoints: axum services expose `/healthz` and `/readyz`
(`services/abis-svc/crates/abis-api/src/lib.rs:98-99`, handlers in
`routes/health.rs`); Kubernetes probes consume `/health` on port 8080 with staggered
initial delays (`services/rts-svc/deploy/base/rts-api-deployment.yaml:32-44`).
Kyverno enforces the posture fleet-wide: `platform/kyverno-policies/require-probes-and-limits.yaml`
and `require-otel-annotations.yaml`.

Metrics/traces pipeline: all 18 services emit OTLP to the OTel Collector, which routes
to Prometheus (scraping + alerting), Loki (structured JSON logs), Tempo (traces), with
Mimir for production long-term metrics and Grafana for dashboards
(`platform/observability/README.md`, "Stack components"). Provisioned assets:

- Dashboards: one Grafana JSON per service under `platform/observability/dashboards/`
  (e.g. `bvi-svc.json`, `transparency-svc.json`).
- Alerts: Prometheus rules per service domain in `platform/observability/alerts/`.
- SLOs: Sloth-compatible YAML per service in `platform/observability/slo/`, matching
  the `SloDefinition` type in code (`eel-observability-rs/src/slo.rs:17`).
- Local overlay: `platform/observability/local/` (single-node Grafana, no Mimir).

Load testing: profiles per environment in `loadtest/profiles/`
(`local.yaml`, `staging.yaml`, `pre-prod.yaml`, `prod-dry-run.yaml`), k6/Gatling
scenarios, and a kit simulator under `loadtest/kit-simulator/`. The 5x-peak gate runs
from `verification/load-5x/`.

---

## 6. Failure and Degradation

ADR-0009 (`docs/adr/0009-resilience-posture.md`) defines the discipline, implemented
in shared crates so no service can skip it:

- Three operating modes per service — normal / degraded / read-only-lockdown;
  feature-flagged, one-command switchable, documented per module in
  `docs/runbooks/<module>-degradation.md` and rehearsed monthly (ADR-0009 "Decision").
  Fleet-level runbook: `docs/runbooks/degradation-per-service.md`.
- Code surfaces: `DegradationMode` flag (`eel-resilience-rs/src/degradation.rs:8,25`);
  write-guard middleware rejecting writes during lockdown
  (`eel-api-server-rs/src/middleware/degradation_guard.rs:6`).
- Circuit breakers with half-open probe recovery (`circuit_breaker.rs:41`); bounded
  queues and per-upstream semaphores; retry budgets; graceful SIGTERM drain (ADR-0009).
- Load shedding by priority class returning `503 Retry-After`
  (CoDel shedder `eel-loadshedding-rs/src/shedder.rs:45`; priority extraction
  `priority.rs:27`); sequencer-side backpressure returns 429 + Retry-After when the
  append semaphore is exhausted
  (`tlog-sequencer/src/rate.rs:7,22`).
- Queue backpressure: Broadway/tokio flow control end-to-end; Kafka consumers pause
  rather than pile up (ADR-0009 "Backpressure end-to-end").
- Offline kit operation: kits buffer and transmit later; edge clusters accept writes
  kit-proximately (`deployment-topology.md:47`); transparency-log downtime never
  blocks the write path because producers buffer in outbox + Kafka
  (ADR-0017 "Availability").
- Chaos engineering as deliverable: weekly Chaos Mesh staging runs and monthly
  production-equivalent drills killing pods, partitioning sites, failing primary
  Postgres, saturating links, poisoning DNS, filling disks, expiring certs
  (ADR-0009; scenarios in `chaos/scenarios/`; runner harness `verification/chaos-runner/`).

---

## 7. Disaster Recovery

### 7.1 dr-failover tool

`tools/dr-failover/` (binary `eel-dr-failover`, WP-5.9). Subcommand model
(`tools/dr-failover/README.md`):

| Subcommand | Purpose |
|---|---|
| `measure` | compute current RTO/RPO from Prometheus/Loki against configurable targets |
| `plan` | emit sequenced failover plan without executing |
| `execute` | execute step-by-step **with interactive confirmation gates** before each destructive step |
| `status` | primary-up state, mirror lag, last backup age |
| `drill` | full DR drill in simulation mode (no real traffic) |

Failover sequence (abbreviated, README "Failover sequence"): verify mirror lag within
RPO -> quiesce primary writes via Envoy route weights to 0 -> promote mirror PG ->
flip MirrorMaker 2 direction -> move DNS (Cilium BGP / external DNS) -> re-enable
writes -> validate services -> record the failover event to the transparency log.
Route-weight phases match `platform/envoy-gateway/README.md` (100/0 -> 0/0 -> 0/100).

**Live execution is manual by design**: `execute` prompts for confirmation at each
step (auto-confirm only allowed for drills via `EEL_DR_CONFIRM=auto`), consistent
with ADR-0022's manual prod deploys and CAB-gated live verification modes.
Runbooks: `docs/runbooks/dr-failover-bvr-svc.md`, `dr-failback.md`,
`backup-restore-drill.md`. DR rehearsal harness: `verification/dr-drill/`.

### 7.2 Backup and archive

- Cluster backups: pgBackRest for PostgreSQL (`platform/pgbackrest/`), Velero for
  cluster state (`platform/velero/`), Kopia for filesystem/object snapshots
  (`platform/kopia/`), MinIO site-to-site replication (ADR-0022 DR posture).
- Long-term archive (module M69, owned by support-admin-svc): BagIt v1.0 (RFC 8493)
  collections under OAIS (ISO 14721) with PreservationAttestations (HSM-signed),
  DestructionCertificates (commissioner-signed, witnessed), DataCite DOIs, and KNADS
  handover under the Public Archives Act (`docs/features/long-term-archive.md`;
  aggregates table in section 2). Distinct from M14 reproducibility bundle:
  M14 = immediate post-election public package; M69 = decades-scale institutional
  memory (same doc, section 1).

---

## 8. CI Gates

CI is Woodpecker (ADR-0006), pipelines in `.woodpecker/`:

| Pipeline | Gate |
|---|---|
| `.woodpecker/completeness.yaml` | builds `eel-completeness`; PRs run `scan --fail-on-new --format junit` (only new non-allowlisted findings fail); pushes to main run `scan --strict` plus `trace` and `report` |
| `.woodpecker/pii-lint.yaml` | HARD-24 PII-in-events Semgrep lint on every PR/push via `eel-pii-lint --root ../..` |
| `.woodpecker/contracts.yaml` | OpenAPI/Pact contract checks (companion to step 9.4 contract closure) |
| `.woodpecker/test.yaml`, `lint.yaml`, `fuzz.yaml`, `mutation.yaml`, `sbom.yaml`, `sign.yaml` | tests, lints, fuzz corpus, mutation testing, SBOM generation, artifact signing (cosign) |
| `.woodpecker/tlc-nightly.yaml` | nightly transparency/reproduction checks |
| `.woodpecker/build.yaml`, `deploy-dev.yaml` | build and dev-environment GitOps deploy |

Trace coverage gate: `eel-completeness trace` maps `#[trace(...)]` /
`@pytest.mark.trace(...)` / ExUnit / Dart / Playwright tags against
`docs/plans/TRACEABILITY.yaml` (`tools/completeness/README.md`, "trace"). ASVS
coverage reporting is provided separately by `tools/asvs-check` against
`tools/asvs-check/asvs-manifest.yaml`.

Contract closure gate: `verification/completeness/steps/9.4-contract-closure.sh`
asserts Pact can-i-deploy passes (env `PACT_BROKER_URL`) and every Karapace schema
has at least one producer and one consumer test (env `EEL_KARAPACE_URL`).
Gate G6 (production certification) requires the full Wave-6 evidence package with
100% completeness for tier-A modules and >=95% for tier-B/C
(`verification/completeness/README.md`, "Purpose").

---

## Discrepancies Between Docs and Code

Found while writing this reference:

1. **Edge region naming.** `docs/architecture/cross-cutting/deployment-topology.md:24`
   lists an "Edge: Nakuru" cluster; ADR-0019 (`0019-edge-topology.md:19`),
   `edge/README.md`, and every `services/*/deploy/service.yaml` use **Eldoret**
   instead. The cross-cutting diagram is stale.
2. **No top-level `deploy/` directory.** Docs referencing `deploy/` manifests should
   point at `platform/`, `edge/`, `environments/`, or `services/*/deploy/`.
3. **`x-eel-tier` vocabulary drift.** `packages/openapi-contracts/README.md` says
   tiers are `A|B|C` data classes; actual contracts use `internal` / `2` / `3`
   (`packages/openapi-contracts/bvi/openapi.yaml:83,460,547`).
4. **Tier-model duplication.** `data-tiers.md`/`trust-model.md` describe storage tiers
   T0-T3, while ADR-0016 and the `eel-pii-*` packages enforce five field classes
   A-E. Both are real but the mapping between them is nowhere stated in docs.
5. **Edge shared-config directories are empty** (`edge/nginx`, `edge/varnish`,
   `edge/anubis`, `edge/rate-limits`, `edge/waf-rules` contain only README + .gitkeep);
   the config files tabulated in those READMEs are not yet committed.
6. **No in-memory store in bvi-svc.** The only `BviStore` impl is `PgStore`; hermetic
   testing there relies on testcontainers, unlike ai-platform/ecc which ship memory
   stores.
7. **Load-profile filename drift.** `verification/completeness/README.md` requires a
   load profile at `loadtest/profiles/preprod-5x.yaml`; the directory actually contains
   `pre-prod.yaml` (plus local/staging/prod-dry-run). Either the check or the file must
   be renamed.
8. **Security-relevant stubs** (detailed in `docs/system/security.md` section 10):
   transparency-log placeholder STH signatures (`pending-wp22`), unwired ABIS auth
   middleware, stub cosign response header, hardcoded wa-bot webhook token, stub kit
   attestation verification.
