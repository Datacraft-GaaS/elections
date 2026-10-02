<title>Module Completeness Checker</title>

# Module Completeness Checker

Verifies that all 69 modules specified in the tender (the tender specification) are implemented,
tested, documented, and traceable to requirements before gate G6.

## Purpose

WP-6.7. Produces a module-by-module completeness matrix. Gate G6 is blocked until completeness
score is 100% for tier-A modules and >= 95% for tier-B/C modules.

## What "complete" means per module

A module is complete when all of the following are evidenced:

1. Service code exists at the canonical path (`services/<svc>/`)
2. At least one `TRACEABILITY.yaml` entry maps the module to a functional requirement
3. `tests/ci/` contains tests with `@trace` annotations referencing the module code
4. `docs/README.md` exists for the service
5. `data_dictionary.yaml` exists and passes schema validation
6. OpenAPI spec exists in `packages/openapi-contracts/<ctx>/`
7. Kustomize deploy manifests exist for `primary`, `mirror`, and at least one `edge-*` overlay
8. Load profile entry exists in `loadtest/profiles/preprod-5x.yaml`
9. Degradation runbook exists at `docs/runbooks/<svc>-degradation.md`

## Modes

### dry-mode

Checks all nine completeness criteria against local file system. Produces a completeness report
without running any tests.

```bash
just completeness dry
```

### live-mode

Runs all nine checks plus executes the `tests/ci/` suite for each module and verifies CI pass.

```bash
just completeness live ENV=preprod
```

## Running a single module check

```bash
just completeness dry MODULE=rts-svc
```

## Expected artefacts

Written to `verification/reports/completeness-<date>/`:

- `module-matrix.json` -- per-module completeness score (0-9 criteria met)
- `module-matrix.html` -- colour-coded HTML matrix (green/amber/red)
- `missing-evidence.json` -- list of failing criteria with file paths expected
- `traceability-coverage.json` -- percentage of FR/AC codes with test traces
- `gate-g6-completeness.sig` -- signed evidence for G6

## Tier definitions

| Tier | Criteria | Modules |
|------|---------|---------|
| A | All 9 criteria, zero exceptions | M01-M11, M14, M18, M19, M22, M24, M51, M52, M63, M64 |
| B | Criteria 1-7 required; 8-9 best-effort | M15, M16, M20, M23, M35, M36, M39, M53, M65, M68 |
| C | Criteria 1-5 required | All remaining modules |

## Pass criteria

- Tier-A: 100% completeness (all 9 criteria met for all tier-A modules)
- Tier-B: >= 95% average completeness
- Tier-C: >= 90% average completeness
- Traceability coverage: >= 95% of functional requirements traced to at least one test

## Limitations

- Does not check subjective quality of implementations, only structural evidence
- TRACEABILITY.yaml format must match `docs/EVENT-CONVENTIONS.md` schema or check fails
- Manual evidence items (code reviews, design sign-offs) are out of scope; checked in G5 gate

## Related docs

- `docs/plans/00-principles.md` -- module-to-WP ownership table
- `packages/event-schemas/` -- event schema completeness (separate check)
- `verification/security/README.md` -- ASVS completeness
