# Contribution Guide

## Commit style

Conventional Commits: `<type>(<scope>): <subject>`. Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `perf`. Scope is the service or package slug (e.g. `rts`, `bvr`, `eel-crypto-rs`). Subject: imperative, lowercase, no period, 72-char limit on first line.

Examples:

```
feat(rts): add form34d transmission for diaspora stations
fix(bvr): correct voter merge on IPRS middle-name variation
test(abis): add enclave attestation refresh edge case
docs(architecture): add boundary-svc service page
```

## Branch naming

`<type>/<scope>-<short-description>`. Examples: `feat/rts-diaspora-form34d`, `fix/bvr-iprs-merge`.

## PR checklist (Definition of Done)

Before opening a PR, verify all of the following:

- [ ] All tests in `tests/ci/` pass (`just test`)
- [ ] No `pyright` or `cargo check` errors
- [ ] New modules have acceptance tests tagged with module ID (e.g. `@pytest.mark.M10`)
- [ ] New API endpoints are reflected in the OpenAPI contract; `just gen-clients` regenerated
- [ ] New events have schemas in `packages/event-schemas/`
- [ ] Migrations are additive-only (no column drops or renames without two-phase plan)
- [ ] No secrets or PII in test fixtures
- [ ] `docs/` updated if behaviour changes; `docs/adr/` updated if a decision changed
- [ ] PR description references the WP or module ID (e.g. `WP-3.6`, `M10`)
- [ ] TRACEABILITY.yaml updated if new acceptance criteria were added

## Boundary rules: sw/ vs docs/

Changes to software (`services/`, `packages/`, `platform/`, `edge/`, `integrations/`, `tools/`) and changes to documentation (`docs/`) have separate review paths. Never mix a software fix with an unrelated documentation change in the same PR. The tender.txt (RFP) is binding: do not change behaviour without tracing to a requirement.

## Test tagging conventions

Use pytest markers for Python; custom attributes for Rust nextest.

| Tag | Meaning |
|---|---|
| `@pytest.mark.M<N>` | Covers acceptance criterion for module N |
| `@pytest.mark.ci` | Included in `tests/ci/` auto-discovery |
| `@pytest.mark.slow` | Excluded from `just test` fast pass; runs in nightly |
| `@pytest.mark.pii_t1` | Handles T1 PII; reviewed for DPA compliance |

Rust: `#[cfg_attr(test, doc = "M10")]` convention; see `tools/completeness/` for extraction.

## Code style

Follow patterns in `services/service-template-{rs,py,ex}/`:

- Python: async, tabs, modern typing, Pydantic v2 with `ConfigDict(extra='forbid')`.
- Rust: Axum handlers, `thiserror`, structured logging via `tracing`, UUID7 for IDs.
- Elixir: GenServer per domain aggregate, Phoenix Channels for real-time, HLC timestamps.

## Sensitivity reminders

- The RFP (`tender.txt`) is binding. No functional change without a traceable requirement.
- Do not add `iebc` to identifiers; use `eel` prefix everywhere (see `docs/adr/README.md`).
- No em or en dashes in documentation or commit messages.
- PII: use `eel-pii-*` libraries; never log T0/T1 data in plaintext.
