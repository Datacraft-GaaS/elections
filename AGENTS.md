# Agent guide

## Workflow

How work moves through this repository. Written from what the history actually
shows, not from an ideal process.

- **Verify with a command, not an assertion.** Every claim in a change should be
  backed by something that was run. A summary that says a suite is green is worth
  nothing without the count.
- **One logical group per commit.** Mixed changes make a revert a bisect. Fix the
  bug, then note the bug, then fix the doc — separately.
- **Build artifacts never get committed.** `.gitignore` has no effect on
  already-tracked paths; a file force-added before its ignore rule existed stays
  tracked forever. Check `git ls-files | rg '\.gradle/|/build/|\.apk$'` after
  adding an ignore rule.
- **Watch the disk.** `target/` reached 37G and filled the volume mid-build. Run
  `cargo clean` after a large verification, not when something fails.
- **Push in batches.** Long verification runs end in a commit, not in an
  unverified working tree.

### When a build fails, read the error before retrying

Most failures here have been real defects wearing a trivial costume. Examples
found by actually reading them: a `u128` accumulator that overflowed and made
audit sampling select every station; a `date` item whose value was never parsed,
so `"1990-13-45"` passed validation; a `min_age_years` bound that compiled and
then enforced nothing. Retrying or working around any of those would have left a
silent correctness hole in an election system.

---

## Codebase

Orientation, for an agent about to edit something it has not read.

- **Layout.** `services/<svc>/crates/*` Rust, `packages/*` shared libraries in
  Rust/Python/Elixir, `platform/` cluster config, `integrations/` external
  adapters, `tools/` dev tooling, `docs/` documentation, `formal/` TLA+ and kani.
- **Naming.** `eel` prefix only. The client name appears in prose, never in an
  identifier, package, header, or event name (ADR-0026).
- **Nothing is `target/` or `node_modules/`.** Both are large and regenerable.
- **Search before assuming.** `codebase-memory` graph tools for structure,
  `zvec-grep` for content across files, `rg` only for a known literal.

### Lints that fail the build, and why

`.cargo/config.toml` sets `--deny=unused-variables`, `--deny=unused-mut`, and
`--deny=unsafe-code` repo-wide. The first two are the reason an unused test
binding is a build failure, not a warning.

The third has exactly four deliberate exceptions, and **every one of them is an
FFI boundary**, where dereferencing caller-supplied pointers is the job:

| Crate | Boundary |
|---|---|
| `packages/verifier-rs` | Dart FFI, Merkle proof verification |
| `services/biometric-edge-svc/crates/nearform-nbis` | C FFI to the vendored NBIS bozorth3/mindtct |
| `services/biometric-edge-svc/apps/bvr-desktop/bridge` | C ABI for the Qt client |
| `services/biometric-edge-svc/apps/bvr-desktop/bridge/tests` | Same ABI, called as C++ would |

A crate-level `#![allow(unsafe_code)]` does override the command-line deny, so
the blanket policy stands everywhere else. Adding a fifth is possible but should
be argued on the grounds that it is an ABI boundary, not convenience — the check
is `rg -l "allow\(unsafe_code\)" --glob "*.rs"`, and a result that is not a
boundary is a finding.

`#![warn(missing_docs)]` on public crates is why new public items need a doc
comment. Warnings are tolerated on `missing_docs`; they are not tolerated on
unused code.

### Conventions worth matching

- Errors are `thiserror` enums with `Display`; never parse an error message.
- New public behaviour gets a test that would fail without it.
- A test that cannot fail is worse than no test. Where a test guards a document
  or a contract, prove it by breaking the thing and watching it fail.

---

## Planning

- **Spec-first (ADR-0011).** Contracts live in `packages/openapi-contracts/` and
  services implement against them. A handler without a contract is a guess.
- **Decisions get an ADR.** `docs/architecture/decisions/` continues numbering
  from 0025; `docs/adr/` holds 0001-0025 with an index. ADRs are immutable after
  acceptance: amendments are new ADRs, and numbering collisions are recorded
  rather than renumbered, because renumbering invalidates every citation.
- **Sequential M rules (§11.0).** M4 follows the M2/M3 gates, M5 follows M4.
  Do not run a later milestone's work early to look productive.
- **Scope honesty.** When something is blocked on infrastructure this repository
  does not have, write down the blocker and the exact evidence. `status.md`
  items 16 and 17 are the worked examples.
- **Prefer the declarative core.** When a feature is "make this configurable",
  build the document model and its compiler first, and let the UI be generated.
  That ordering is why adding a registration field is now a document edit.

---

## Research

- **Verify, do not recall.** Standard numbers, licence terms, and vendor
  capabilities are exactly the things to look up. `abis-modality-gait` cited
  ISO/IEC 19794-6, which is the *iris* standard; the binding one is ISO/IEC
  39794-17. That was found by searching, and is now pinned by a test.
- **Prefer a primary source.** An ADR in this repository, or the standard
  publisher, beats a blog post. Where no standard governs something, say so
  explicitly rather than naming a plausible one; citing a standard the data
  does not conform to is worse than citing none in a tender deliverable.
- **A design idea is worth taking; its 2009 realisation is not.** MBARK's
  contribution was describing biometric acquisition *formally* and compiling it
  (BiCCL, NIST IR 7531). Its XML pipeline, .NET implementation, and plug-in
  mechanism are not the idea. ADR-0027 records which parts were taken and why.
- **Negative results are results.** "Tauri is a webview, not a native GUI" and
  "genuinely native splits by platform" were both worth saying out loud rather
  than quietly building the wrong thing.

---

## Dependency & Compilation Rules (M5 chain, ADR-0022/0023)

### Lockfile Discipline
- **Commit Cargo.lock** — freeze exact dependency graph; never .gitignore it.
- **Locked builds** — `cargo build --locked`, `test --locked`, `check --locked`; prevents accidental updates.
- **Frozen builds** — `cargo build --frozen` / `test --frozen`; fully offline, locked, no network.
- **Update one at a time** — `cargo update -p crate_name --precise X.Y.Z`; never bulk `cargo update`.
- **Vet crate age** — check crates.io release date; choose version ≥30 days old (stability gate).
- **Pin Rust compiler** — `rust-toolchain.toml` with exact version (e.g., 1.82.0); no floating.
- **Audit before commit** — `cargo audit` + `cargo deny check`; fix all findings.

### Service Verification (all 20 services)
- Compile all workspace-root services (`biometric-edge-svc`, `boundary-svc`, `bvi-svc`, `bvr-svc`, `field-mobile-svc`, `queue-svc`, `rts-svc`, `support-admin-svc`, `vprs-svc`) + individual child crates (`abis-svc`, `ai-platform-svc`, `crms-svc`, `ecc-svc`, `governance-svc`, `hello-svc`, `inventory-svc`, `portal-svc`, `training-svc`, `transparency-svc`, `workforce-svc`).
- **Field-mobile-svc**: green. `mobile-bff`, `support-api`, `dispatch-api`, and `eel-field-mobile-telemetry-ingest` all compile clean under `cargo check --locked --all-targets`; 91 tests pass. The earlier E0382/E0061/E0599/E0308 errors in `mobile-bff` are resolved.
- **Training-svc** (`training-api`): verified 22.27s, 298 `missing_docs` warnings only — acceptable per `#![warn(missing_docs)]`.
- **No stubs**: 2 defensive `unreachable!()` kept; no `UnimplementedError`; no empty endpoints.
- **Audit reference preserved** in every doc: 5-axis (where/when/who/why/how) + `proofHash` + `signature` + `auditId` (UUID7).
- **Sequential M rules** (§11.0): M4 after M2/M3 gates; M5 after M4.
