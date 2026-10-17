# Production readiness: what remains

Measured 2026-10-17. Every number below came from a command run against this
tree, and the command is named beside it. Where a figure is a count of something
a gate produced, the gate's own exit status was read from a redirect rather than
through a pipe — `| tail` returns tail's status and has already masked a build
failure in this repository once.

Scope: the 69 modules, 19 Django apps, 18 Kubernetes services, 121 SQL
migrations, 245 Cargo manifests and 23 Flutter packages under `services/`,
`packages/` and `tools/`.

---

## Where it stands

| gate | command | result |
|---|---|---|
| Cargo manifests | `tools/cargo_manifests.py` | 245 checked, **0 broken** |
| Migrations apply | `tools/check-migrations.sh` | **121 applied clean** |
| Django schema drift | `tools/check-django-schema-drift.py` | **12 tables drifted, 4 apps** |
| Handler fidelity | `tools/rust_handler_fidelity.py --all-routes` | 639 judged, **8 fabricated** |
| Fabrication triage | `tools/fabrication_triage.py` | 8 total: **1 implementable**, 1 needs build, 6 blocked |
| Flutter analyze | `tools/flutter_health.py` | 23 packages, **0 errors / 0 warnings / 0 infos** |
| Flutter tests | `tools/flutter_tests.py` | 23 packages, 22 green, **1 failing** (376 passed, 1 failed) |
| Python lint | `ruff check services packages tools` | **18 errors** |
| Python format | `ruff format --check services packages tools` | **37 files would be reformatted** |

Two of these were regressions introduced during the schema-reconciliation work
and are fixed in this session: the root Cargo workspace was unloadable
(`tools/lib/` matched the `tools/*` member glob), and the drift count fell from
29 to 12.

---

## 1. Deployability — the largest gap, and nothing else closes it

**None of the 18 services has a `deploy/overlays/` directory.** ArgoCD's 19
ApplicationSets each point at `services/<svc>/deploy/overlays/{{.site}}`:

```
platform/argocd/applicationsets/governance-svc.yaml:65
        path: services/governance-svc/deploy/overlays/{{.site}}
```

`ls services/*/deploy/overlays/` returns nothing for every service. Each
ApplicationSet expands over seven sites — `primary`, `mirror`, `edge-nairobi`,
`edge-mombasa`, `edge-kisumu`, `edge-eldoret`, `edge-garissa` — so the missing
directory is **19 × 7 = 133 Application objects**, every one of which ArgoCD
would fail to render.

What exists instead is a flat `deploy/` holding one `service.yaml` per service
(plus, in a few, an `hpa-*.yaml`, `pdb.yaml`, `networkpolicy.yaml`). There is no
base/overlay split, so the per-site differences the ApplicationSets assume
(cluster, namespace, replica count, resource class) have nowhere to live.

The pipeline that would consume this cannot run either:

```
.woodpecker/deploy-dev.yaml:22  - name: argocd-sync
```

It shells out to `argocd appset get` against `argocd.dev.internal` with a token
from a secret. There is no `kustomize build` step and no manifest validation
anywhere in CI, so a broken overlay would be discovered by ArgoCD at sync time
rather than by a pipeline.

**No Django app has a Dockerfile.** 19 apps under `services/*/django/*/`, zero
`Dockerfile`s. Nothing containerises them, so even a correct overlay would have
no image to reference.

**Not verifiable here.** The user's own assessment stands: *"Large scope, and I
can't verify any of it without a cluster."* What can be built without a cluster
is the base/overlay structure and the Dockerfiles; what cannot is any evidence
that they are correct.

## 2. Django schema drift — 12 tables across 4 apps

```
abis-svc/django/abis-console      eel_reviewers
training-svc/django/cms           4 tables
training-svc/django/lms           4 tables
training-svc/django/medical       3 tables
```

The rule agreed for this work was *SQL wins where the app shares a database with
a Rust service, models win where the app owns its database outright.* Every one
of these four apps belongs to `training-svc` or `abis-svc` and shares a database
with a Rust crate, so **SQL wins for all 12** — the models are wrong and the
migrations are the schema.

The drift is not cosmetic. `training_lms_modules` is missing seven columns the
migration creates (`content_markdown`, `language`, `version`, `max_retakes`,
`expiry_date`, `asset_ids`, `case_study_ids`); `training_medical_units` is
missing nine of its own key columns including `unit_id`, `county`, `gps_lat`,
`gps_lon` and `locked`. `abis-console.eel_reviewers` names `id`,
`specialisations` and `active_case_count` where the table has none of them.

Two patterns already found elsewhere in this work will likely recur:

- A **derived value modelled as a column** (the RFID `cover_state` case — the
  state was the last row of a transition chain, not a field on the tag).
- **`managed = False` models naming tables no migration creates**, which the ORM
  only reports when a query runs. The gate was widened to catch these; three
  such models were found in `ecc-admin`.

## 3. Handler fidelity — 8 handlers, 1 implementable here

`tools/rust_handler_fidelity.py --all-routes`: 639 registered, 8 fabricated.

| handler | triage | blocker |
|---|---|---|
| `verifier-api::lookup_voter_registration` | implementable | needs the roll as a data source |
| `mobile-bff::verify_voter` | needs build | attestation trust anchor |
| `edge-api::distress` | blocked | transactional outbox |
| `rts-device-api::dual_sim_resubmit` | blocked | transactional outbox |
| `rts-device-api::transmission_attempt` | blocked | transactional outbox |
| `rfid-cover-custody::post_ingest_read` | blocked | HSM signature verification, Valkey |
| `gazette::sign_gazette_notice` | blocked | HSM signature verification |
| `cvr::family_notify` | blocked | M02 BVR snapshot client |

Six are blocked on infrastructure this repository does not contain. The one
implementable handler needs a decision rather than code: `lookup_voter_registration`
must consult the register, and which store that is belongs to whoever owns the
M02 schema. It is currently pinned in the classifier self-test with its
`roll_not_published` path documented, which is why the pin and not a 501 is the
honest state.

**Refusals.** `rg -o "StatusCode::NOT_IMPLEMENTED" --glob='*.rs' services/`
returns **44 occurrences across 32 files**. That counts occurrences, not
handlers — one handler can refuse on several branches — so it is an upper bound
and not the figure to track. The previous report recorded the handler count as
"28 → 21", and that number is carried forward here as its claim rather than
re-derived, because the fidelity scanner does not report it and no command in
this session measured it.

## 4. Flutter — one failing package, and it is a toolchain-drift finding

`tools/flutter_tests.py` exits **1**: `packages/design-system/flutter` reports
22 passed, 1 failed. The failure is the golden test `core widgets light + dark`,
and the detail is the finding:

```
Golden "goldens/eel_banner_info.light.png": Pixel test failed, 0.00%, 4px diff
Golden "goldens/eel_result_card.light.png": Pixel test failed, 0.00%, 1px diff
```

**14 of 42 goldens fail, every one at 0.00% with a 1–4 pixel diff.** A
percentage that rounds to zero with a non-zero pixel count is antialiasing, not
a layout change. The goldens were last written 2026-09-05; the widget library
changed 2026-09-29 and the local toolchain is Flutter 3.47.7, installed nine
days ago.

**Nothing in the repository pins the Flutter version.** There is no
`.fvmrc`, no `.tool-versions`, no `flutter-version` in any workflow. So a golden
test — which is a byte comparison of a rasterised render, and therefore a
function of the engine — is being asked to hold across an unpinned toolchain.
That is the defect, and regenerating the goldens would hide it until the next
engine bump.

Compounding it: **`design-system/flutter` is not in the mobile CI matrix.**
`.woodpecker/mobile.yaml` lists 17 apps and design-system is not among them, so
these goldens have never run in CI at all.

The run also left **104 untracked golden-failure PNGs** under
`test/failures/` (`git ls-files --others`), where **8 are already tracked** —
a diagnostic output directory committed to the repository.

## 5. Python lint and format

`ruff check services packages tools` → **18 errors**. None are in the code
reconciled this session (`ruff check` on `inventory-admin` is clean). They
concentrate in `tools/`: `rust_handler_fidelity.py` (2 × F841 unused variable),
`cargo_manifests.py` (2 × E741 ambiguous `l`), plus `flutter_health.py`,
`fabrication_triage.py`, `prune_unused_imports.py`, and single findings in
`training-svc`, `support-admin-svc`, `biometric-edge-svc`, `ai-platform-svc`,
`abis-svc` and two shared packages.

`ruff format --check` → **37 files would be reformatted**, 573 already formatted.

Both are wired into CI (`lint.yaml` step `python-ruff` runs exactly these two
commands), so **the lint pipeline is currently red**.

## 6. Traceability — the largest honest number

Trace tags exist in **30 Rust files out of 1,509** (`rg -l` for the tag pattern
against `services/`). The requirement corpus is larger than that by orders of
magnitude. The previous report measured "25 of 4,444 rows have passing tagged
tests"; nothing in this session moved it, and no command here reproduced the
4,444 figure, so it is carried forward as that report's claim.

This is a test-writing programme across all 69 modules, not a code fix. The
tagging harness works; the burn-down is per-module and should be tracked with
`eel-completeness trace` rather than estimated.

**Unbound OpenAPI paths.** The previous report recorded 287. No command in this
session reproduced that number — the fidelity scanner reports contract-bound and
contract-unbound *fabrications* (1 and 7), which is a different question from
how many declared paths have no handler at all. Counting the latter needs a
spec-versus-router diff that does not exist yet; until it does, the 287 stands
as the prior report's measurement and should be re-derived before it is used to
size work.

## 7. Blocked on inputs this repository does not have

These are recorded rather than started, because no amount of work here closes
them.

- **Live Wave-6 execution**: rehearsal at scale, DR failover, chaos-during-load,
  measured 5× load, 24h fuzz/DAST, screen-reader sweeps. Harnesses are dry-run
  green; they need pre-prod clusters.
- **ONNX face detection**: needs the deployed model's input/output contract
  (anchor decode, NMS) shipped with the bundle.
- **`ecc` s44a_live and `bvi` lookup_live**: no S44A module exists in `ecc-api`
  and `bvi-svc` declares three crates that are absent.
- **`field-mobile-verify`** has a migration, README and TRACEABILITY and no
  implementation. Whether the skeleton is worth keeping is a product decision.
- **`bvr-api` idempotency** (FR-M02-003): no durable idempotency store exists,
  so this is a feature gap and not taggable as a test.

## 8. Environment hazards found during this work

Recorded because each cost time and will cost it again.

- **`tools/*` is a Cargo member glob.** Any non-Rust directory added under
  `tools/` breaks the entire root workspace, and cargo reports only the missing
  manifest. `tools/__pycache__` and now `tools/lib` are excluded; a new Python
  helper directory needs a new entry.
- **`eel-migrate` tracks `(service, name)` with no checksum.** Editing an
  applied migration's body never re-applies it. Schema fixes must be new
  revisions — which is why the append-only guard repair is V055 and V053 rather
  than an edit to V051 and V052.
- **`| tail` on a gate masks its exit code.** Reading `flutter_tests.py` through
  a pipe reported success while the gate was failing.
- **Five scratch Postgres instances are running** on ports 55993, 55493, 55433,
  50440 and 50425, plus a stale `/tmp/eel-pg-django` data directory. They are
  not part of this repository and should be stopped and removed when the
  reconciliation work finishes.

---

## Order of work

1. **Deployability** — base/overlay structure for 18 services, Dockerfiles for
   19 Django apps. Largest gap; needs a cluster to verify.
2. **Django drift** — 12 tables in 4 apps, SQL wins in all 12.
3. **Python lint** — 18 errors and 37 format violations; the CI lint pipeline is
   red until they are fixed.
4. **Flutter golden drift** — pin the Flutter version, add `design-system` to
   the CI matrix, then decide whether to regenerate or widen the comparator.
   Regenerating first would hide the cause.
5. **`lookup_voter_registration`** — one handler, blocked on a data-source
   decision rather than on effort.
6. **Traceability and OpenAPI binding** — long programmes, tracked as burn-down.
