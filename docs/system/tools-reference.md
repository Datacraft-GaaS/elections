# Tools Reference (`tools/`)

Developer, security, and operator tooling under `tools/`. All Rust tools build from the workspace root: `cargo build --release -p <name>` ([tools/README.md](../../../tools/README.md)). Two tools are placeholder-only crates excluded from the workspace (see status column and the root [Cargo.toml](../../../Cargo.toml) `exclude` comment: *"softhsm, spectral: placeholder-only tools (`src/lib.rs` contains no implementation)"*).

Quick index:

| Tool | Language | Security role | Status |
|---|---|---|---|
| appset-gen | Rust | Generates NetworkPolicies + Kafka ACLs | Implemented |
| asvs-check | Rust | OWASP ASVS 4.0 L2/L3 coverage gate | Implemented |
| completeness | Rust | Stub/traceability/contract-closure gate | Implemented |
| dr-failover | Rust | DR drill planner/executer | Implemented |
| hsm-ceremony | Rust | Multi-custodian key ceremony | Implemented |
| hsm-init | Rust | HSM slot/key bootstrap | Implemented |
| iam-seed | Rust | Idempotent IAM bootstrap | Implemented |
| kit-simulator | Rust | Election-day load simulator | Implemented |
| pii-in-events | Python | Tier-A PII lint over events/sources | Implemented |
| softhsm | Bash (+ stub crate) | Local PKCS#11 token init | Bash implemented; crate is a stub |
| spectral | JS (+ stub crate) | OpenAPI lint ruleset | JS implemented; crate is a stub |
| spicedb | Bash (+ stub crate) | SpiceDB CLI wrapper | Wrapper implemented; crate is a stub |
| synth | Rust | Deterministic synthetic data | Implemented |
| wcag-audit | Rust | WCAG 2.2 AA accessibility gate | Implemented |

---

## appset-gen

**Purpose.** Generates ArgoCD ApplicationSets, Kustomize overlays, Cilium NetworkPolicies, and Kafka ACLs from per-service manifests ("Generate ArgoCD ApplicationSets and platform manifests from service descriptors", main.rs:13).

**Invocation.** Binary `eel-appset-gen`; clap subcommands defined at [tools/appset-gen/src/main.rs:17-41](../../../tools/appset-gen/src/main.rs):

- `service [--service NAME] [--all] [--out DIR]` — emit ApplicationSet + Kustomize overlays; default output `platform/argocd/applicationsets`
- `topics [--out DIR]` — emit Cilium NetworkPolicies and Kafka ACLs from `platform/kafka/topics/*.yaml`; default output `platform/cilium/generated`
- `check` — exit non-zero if any service lacks `deploy/service.yaml`
- `initial-services` — write template `deploy/service.yaml` for each of the 18 canonical services

**Key functions.** Dispatch in `main()` (main.rs:43); service discovery/loading via `model::load_service`; manifest emission in `gen::emit_appset` / `gen::emit_kustomize_overlays` ([src/gen.rs](../../../tools/appset-gen/src/gen.rs)); Kafka/Cilium generation in [src/kafka.rs](../../../tools/appset-gen/src/kafka.rs).

**Produces.** ApplicationSet YAML, Kustomize overlays, Cilium policies, Kafka ACLs.

**Security role.** The `topics` command is the sole generator of network segmentation policy and Kafka authorization ACLs — drift here directly changes the attack surface between services.

---

## asvs-check

**Purpose.** "Maps ASVS L2/L3 requirements to test trace tags and reports coverage gaps" ([tools/asvs-check/src/main.rs:8-11](../../../tools/asvs-check/src/main.rs)). Correlates `#[trace("...")]` markers (from `eel-trace-rs`) against an ASVS requirement manifest.

**Invocation.** Binary `eel-asvs-check`, subcommands (main.rs:27-47):

- `scan` — walk repo, collect test trace tags, print coverage per requirement
- `report [--format html|md|json] [--output FILE]`
- `check [--strict]` — CI gate: exit non-zero on uncovered L3 requirements (also L2 with `--strict`)

**Key functions.**

| Function | Location | Description |
|---|---|---|
| `scan_repo(root)` | src/scan.rs:32 | Harvests trace tags across test files |
| `map_requirements(...)` | src/scan.rs:90 | Joins tags to ASVS manifest entries |
| `gaps(results)` / `gaps_by_level` | src/scan.rs:131 / :135 | Coverage gap computation |
| `build_json` / `render_json` | src/report.rs:39 / :91 | Report serialization |

Manifest handling: [src/manifest.rs](../../../tools/asvs-check/src/manifest.rs), default path resolved by `manifest::default_manifest_path(&root)` (main.rs:55).

**Validates.** Every security-relevant requirement has at least one mapped test.

**Security role.** This is the platform's compliance gate for OWASP ASVS 4.0 Level 2/3; it converts untested security requirements into CI failures. Tests: `tests/scan_test.rs`.

---

## completeness

**Purpose.** Stub scanner, traceability mapper, and contract-closure verifier — prevents unfinished code and unimplemented contracts from silently shipping.

**Key functions.**

| Function | Location | Description |
|---|---|---|
| `scan::run(root, allowlist)` | tools/completeness/src/scan.rs:188 | Finds TODO/stub/unimplemented markers |
| `scan::emit(findings, format, fail_on_new, strict)` | src/scan.rs:333 | Report emission with fail-on-new semantics |
| `trace::run(root, traceability_path, fail_on_gap)` | src/trace.rs:54 | Maps tests to TRACEABILITY.yaml requirements |
| `contract_closure::run(root, fail)` | src/contract_closure.rs:37 | Verifies declared contracts are implemented |
| `allowlist::load(path)` | src/allowlist.rs:72 | `.completeness-allowlist.yaml` handling |

CLI entry `fn main()` at [tools/completeness/src/main.rs:65](../../../tools/completeness/src/main.rs) — parses a root path plus optional allowlist, defaulting to `.completeness-allowlist.yaml` at the repo root. Fixtures for self-testing live in `fixtures/should-trigger/` and `fixtures/no-trigger/`; integration test in `tests/integration.rs`. A helper script `seed_traceability.py` bootstraps TRACEABILITY files.

**Security role.** Indirect but important: blocks placeholder implementations of security controls from reaching production by treating stubs as release blockers.

---

## dr-failover

**Purpose.** Automated disaster-recovery failover assist: measures RTO/RPO from Prometheus, plans site/service failovers, executes them with confirmation gates, and reports on drills.

**Invocation.** Binary `eel-dr-failover`, subcommands ([tools/dr-failover/src/main.rs:44-75](../../../tools/dr-failover/src/main.rs)):

- `plan [--service S] [--target-site SITE]` — print playbook without executing
- `execute [--service S] ... --dry-run | --confirm` (execution requires one of these flags — enforced by test `execute_requires_dry_run_or_confirm`, main.rs:513)
- `report [--drill ID]` — summarize a drill

**Key functions.**

| Function | Location | Description |
|---|---|---|
| `site_failover_plan(target_site)` | src/main.rs:143 | Ordered failover steps for a full site |
| `service_failover_plan(svc, target_site)` | src/main.rs:197 | Per-service plan |
| `query_prometheus(prom_url, promql)` | src/main.rs:227 | RTO/RPO metric extraction |
| `cmd_plan` / `cmd_execute` / `cmd_report` | src/main.rs:267 / :289 / :341 | Subcommand handlers |

Entry: `async fn main()` (main.rs:446).

**Security role.** Operational resilience for election-day continuity; the dry-run/confirm gating prevents accidental destructive failovers.

---

## hsm-ceremony

**Purpose.** Orchestrates multi-custodian HSM key ceremonies with quorum approval for key generation/rotation, driving the remote HSM signer over gRPC (tonic `Channel::from_shared(cli.endpoint)`, [tools/hsm-ceremony/src/main.rs:161-167](../../../tools/hsm-ceremony/src/main.rs)).

**Invocation.** Subcommands (main.rs:59-70):

- `custodian <action>` — custodian management (register/list, via `CustodianCommands`, main.rs:73)
- `ceremony <action>` — ceremony lifecycle (via `CeremonyCommands`, main.rs:88)

Output formats selected by `OutputFormat` (main.rs:53); human-readable printer `print_human` (main.rs:339).

**Key functions.**

| Function | Location | Description |
|---|---|---|
| `handle_custodian(...)` | src/main.rs:187 | Custodian subcommand dispatch |
| `handle_ceremony(...)` | src/main.rs:226 | Ceremony execution against the signer |
| `enum RoleName { Sth, Declaration, KitCa, CredentialCa, CodeSigning, CycleKek }` | src/main.rs:134 | Mirrors `eel_crypto_rs::KeyRole` into proto roles (From impl :143) |

**Produces.** Signed ceremony records / key metadata (JSON or human format).

**Security role.** Critical: this tool is how the six ADR-0005 root keys are generated and rotated with M-of-N custodian quorum, so no single operator can mint signing keys. Runbook: `docs/runbooks/hsm-ceremony.md`.

---

## hsm-init

**Purpose.** Bootstraps PKCS#11 keys in SoftHSM slots for local development and CI, replicating the production HSM slot layout.

**Invocation.** Binary `hsm-init`; loads the module from `--module` (defaulting to `PKCS11_MODULE_PATH`) then initializes PKCS#11 ([tools/hsm-init/src/main.rs:150-156](../../../tools/hsm-init/src/main.rs)). Slot-per-role mapping in `slot_for_role(role)` (main.rs:56).

**Key functions.**

| Function | Location | Description |
|---|---|---|
| `find_slot(pkcs11, label)` | src/main.rs:195 | Locates a token slot by label |
| `generate_key(pkcs11, role)` | src/main.rs:209 | Creates the role's key pair |
| `list_keys(pkcs11)` | src/main.rs:284 | Enumerates provisioned keys |
| `sign_blob(pkcs11, role, data)` | src/main.rs:321 | Test-sign with a role key |
| `verify_blob(pkcs11, role, data, signature)` | src/main.rs:341 | Verify round-trip |

**Security role.** Dev-only counterpart to `hsm-ceremony`: ensures every service can exercise the same key-role model (eel-sth … eel-cycle-kek) locally. Pair with `bash tools/softhsm/init.sh` first.

---

## iam-seed

**Purpose.** "Idempotent IAM seeding CLI" ([tools/iam-seed/src/main.rs:14](../../../tools/iam-seed/src/main.rs)): Keycloak realms, SpiceDB schemas, SPIRE workload entries, OpenBao policies/mounts — all idempotent, diff-checked before apply.

**Invocation.** Subcommands (main.rs:20-90):

- `keycloak-apply --realm R --url U [--realm-file F] [--yes]`
- `spicedb-apply --endpoint E --token T (--env SPICEDB_TOKEN) [--schema-dir D] [--yes]` — schema dir defaults to `platform/spicedb/schema/`
- `spire-apply --server S [--entries-dir D] [--yes]`
- `openbao-apply --address A --token T [--policies-dir P] [--mounts-dir M] [--yes]`
- `apply-all --env ENV [--yes]` — runs all applies in sequence

**Key modules/functions.**

| Item | Location | Description |
|---|---|---|
| `compute_diff(current, desired)` | src/lib.rs:6 | Structural diff used for preview before apply |
| `keycloak.rs` / `spicedb.rs` / `spire.rs` / `openbao.rs` | src/ | Per-system clients |
| `verify.rs` | src/verify.rs | Post-apply verification |

Tests: dry-run suites `tests/spicedb_dry_run_test.rs`, `tests/openbao_dry_run_test.rs`, `tests/keycloak_dry_run_test.rs`, diff test `tests/diff_test.rs`.

**Security role.** Critical bootstrap of the entire identity fabric: realms, workload identities (SPIRE), authorization schema (SpiceDB), and secret-engine policy (OpenBao). The `--yes` prompt gate and diff preview prevent unintended privilege changes; tokens are supplied via env (`SPICEDB_TOKEN`) rather than argv history where supported.

---

## kit-simulator

**Purpose.** KIEMS election-day load simulator: drives up to 65k virtual BVI and RTS kits against a target cluster (or an in-process mock server) using scenario profiles.

**Invocation.** Binary `kit-simulator`, subcommands ([tools/kit-simulator/src/main.rs:23-72](../../../tools/kit-simulator/src/main.rs)):

- `run --profile local|staging|pre-prod|prod-dry-run --duration 60s[m|h] --scenario loadtest/scenarios/election-day.yaml [--target-url URL] [--reports-dir D]` — omitting target uses the built-in mock server
- `compare --baseline RUN --candidate RUN [--reports-dir D]` — enforces the p99 regression gate (10% threshold)
- `report --run RUN [--reports-dir D]`

**Key functions.**

| Function | Location | Description |
|---|---|---|
| `runner::run(...)` | src/runner.rs:25 | Load orchestration |
| `run_worker(...)` | src/runner.rs:153 | Per-kit request loop |
| `attestation_token(kit_id)` | src/runner.rs:256 | Fabricates device attestation tokens for requests |
| `mock_server.rs` | src/mock_server.rs | In-process target for dry runs |
| `regression.rs` / `report.rs` / `scenario.rs` / `config.rs` | src/ | Gate math, report I/O, scenario parsing |

**Security relevance.** Not a control-plane security tool, but its regression gate protects availability SLOs during elections; attestation token generation mirrors the shape real kits must present.

---

## pii-in-events

**Purpose.** Lint driver (HARD-24) that walks event schemas and service source trees applying Semgrep rules to detect raw Tier-A personal data in event payloads, cross-checks findings against an allowlist, and exits non-zero on un-allowlisted violations ([tools/pii-in-events/src/eel_pii_in_events/lint.py:1-12](../../../tools/pii-in-events/src/eel_pii_in_events/lint.py)).

**Invocation.**

```
python -m eel_pii_in_events.lint [--root /path/to/repo]
uv run eel-pii-lint [--root /path/to/repo]
```

**Key elements.**

| Item | Location | Description |
|---|---|---|
| `_TIER_A_RE` | lint.py:22 | Raw-PII field names always flagged: national_id, voter_id, passport_number, fingerprint_raw, iris_raw, face_image, dob_exact, etc. |
| `_TIER_B_RE` | lint.py:29 | Quasi-identifiers flagged as warnings (voter_id_hashed, ward_code, …) |
| `_SCAN_GLOBS` | lint.py:33 | Scans event-schemas plus Rust/Python/Elixir service sources |
| `load_allowlist(path)` | lint.py:40 | `{event_type: {fields}}` from `allowlist.yaml` |
| `Finding.is_allowlisted` | lint.py (Finding class) | Allowlist exemption check |
| Rules | src/eel_pii_in_events/rules/no-tier-a-in-event.yaml | Semgrep rule set |

Tests: `tests/test_lint.py`.

**Security role.** Enforces ADR-0016's cardinal rule that Tier-A PII never enters the event backbone; allowlist provides auditable exceptions.

---

## softhsm

**Status.** The usable artifact is [tools/softhsm/init.sh](../../../tools/softhsm/init.sh), a bash script that initializes SoftHSMv2 tokens matching production layout (six slots: eel-sth, eel-declaration, eel-kit-ca, eel-credential-ca, eel-code-signing, eel-cycle-kek — slot table at init.sh:15-22, SO/user PIN pairs embedded in the map). The Rust crate `tools/softhsm` is a **placeholder**: `src/lib.rs` contains only `// stub`, and the workspace excludes it (root Cargo.toml exclude comment cites missing implementation).

**Usage.**

```
bash tools/softhsm/init.sh
cargo run --release -p hsm-init -- init-slots
```

Env vars: `SOFTHSM2_CONF`, `SOFTHSM_TOKEN_DIR`, `PKCS11_MODULE_PATH` ([tools/softhsm/README.md](../../../tools/softhsm/README.md)).

**Security role.** Local-only PKCS#11 stand-in for the Thales Luna/Utimaco production HSMs so developer flows exercise real PKCS#11 paths. Never use its fixed dev PINs elsewhere.

---

## spectral

**Status.** The functional content lives in [tools/spectral/functions/](../../../tools/spectral/functions/) — two custom Spectral rule functions (`eel-has-content-types.js`, `eel-has-data-class.js`) referenced by the repo-root `.spectral.yaml` ruleset enforcing ADR-0011 (JSON+CBOR content types, data-class annotations, Idempotency-Key on POST/PUT, RFC 9457 error bodies, versioned paths). Run with `spectral lint packages/openapi-contracts/**/*.yaml --fail-severity error`. The Rust crate `tools/spectral` is again a **stub** (`src/lib.rs` = `// stub`), excluded from the workspace.

**Security role.** Contract-level enforcement that error responses use registered problem types and that classification/tier metadata cannot be omitted from APIs.

---

## spicedb

**Status.** Host-side wrapper [tools/spicedb/zed](../../../tools/spicedb/zed) resolving the SpiceDB `zed` CLI without colliding with the Zed editor: resolution order `SPICEDB_ZED_BIN` → Homebrew tap → `~/.local/bin/spicedb-zed` → `~/.cargo/bin/spicedb-zed` → Nix flake → PATH `zed` verified as the SpiceDB build ([tools/spicedb/README.md](../../../tools/spicedb/README.md)). Used by `just validate-authz`, `just test-authz`, `just deploy-authz`. The Rust crate `tools/spicedb` is a **stub** (`src/lib.rs` = `// stub`), excluded from the workspace.

**Security role.** The sanctioned way operators touch SpiceDB from hosts; avoids accidentally launching a code editor where a permission-store mutation was intended, and centralizes credential-bearing invocations.

---

## synth

**Purpose.** Deterministic synthetic data generator for voters, geography, forms, kits, personnel, embassies — seeded RNG so outputs are reproducible ("Deterministic synthetic data generator for the eel election-management system", [tools/synth/src/main.rs:16-19](../../../tools/synth/src/main.rs)).

**Invocation.** Binary `eel-synth`, subcommands (main.rs:24 onward): `voters --seed 42 --scale 100000 [--out PATH] [--format csv]`, `geo --seed 42 --stations 4600 [--format geojson|sql]`, `forms --seed 42 --scale 200 [--out DIR]` (Form 34A PNG images), plus further generators (kits, personnel, embassies — see `Commands` enum and modules [src/voters.rs](../../../tools/synth/src/voters.rs), [src/geo.rs](../../../tools/synth/src/geo.rs), [src/personnel.rs](../../../tools/synth/src/personnel.rs), [src/kits.rs](../../../tools/synth/src/kits.rs), [src/embassies.rs](../../../tools/synth/src/embassies.rs)); `verify` subcommand checks generated data consistency ([src/verify.rs](../../../tools/synth/src/verify.rs)).

**Key internals.** Reproducible RNG in [src/rng.rs](../../../tools/synth/src/rng.rs); determinism asserted by `tests/determinism.rs`; smoke test `tests/smoke.rs`.

**Security note.** Synthetic only — deliberately generates fake PII so realistic volumes can be tested without touching real citizen data.

---

## wcag-audit

**Purpose.** WCAG 2.2 AA (and AAA option) accessibility auditor running axe-core against every URL in a manifest, producing JSON/HTML reports and failing CI on violations.

**Invocation.** Binary `eel-wcag-audit`, subcommands ([tools/wcag-audit/src/main.rs:18-45](../../../tools/wcag-audit/src/main.rs)):

- `audit [--manifest tools/wcag-audit/manifest.yaml] [--output-dir wcag-report] [--format json|html|both] [--stub-dir DIR]` — run axe-core against manifest URLs; supports pre-recorded axe JSON results for offline/CI replay
- additional report/check verbs dispatched in `main()` (main.rs:62+)

**Key functions.**

| Function | Location | Description |
|---|---|---|
| `audit_entry(entry, stub_dir)` | tools/wcag-audit/src/audit.rs:30 | Audits a single manifest entry |
| `audit_all(entries, stub_dir)` | src/audit.rs:122 | Batch audit; `AuditResult::is_passing` (:24) |
| `axe::parse(json)` / `filter_violations(violations, level)` | src/axe.rs:60 / :48 | Axe result ingestion and severity filtering |

Support: `manifest.rs` (manifest model), `runner.rs` (headless browser driving), `report.rs`. Tests: `tests/audit_test.rs`.

**Security role.** Accessibility is treated as a statutory/compliance requirement (election surfaces must be usable by all voters); this is the enforcement gate alongside `asvs-check` (both G5/WP-5.12 per the wave table in [tools/README.md](../../../tools/README.md)).

---

## Cross-cutting notes

- **Stub crates pattern:** three tools ship Rust crate skeletons (`softhsm`, `spectral`, `spicedb`) whose actual logic is shell/JS. Root Cargo.toml excludes them because empty libs inherit edition config that breaks standalone `cargo check`.
- **Wave gates** (tools/README.md): G0 — softhsm, hsm-init, appset-gen, iam-seed, spicedb, synth, completeness, spectral; G2 — hsm-ceremony; G3 — kit-simulator; G5 — asvs-check, wcag-audit, dr-failover.
- **Shared env contract:** `EEL_HSM_SIGNER_ENDPOINT` (hsm-init, hsm-ceremony), `PKCS11_MODULE_PATH` (hsm-init, softhsm), `SOFTHSM2_CONF` (softhsm), `SPICEDB_ZED_BIN` (spicedb wrapper).
