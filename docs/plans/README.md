# the Commission IEMS: Full Build Plan (index)

## Files

| file | contents |
|---|---|
| [00-principles.md](00-principles.md) | housekeeping, conventions every WP follows, wave overview, coverage check, risks |
| [01-wave-0-toolchain.md](01-wave-0-toolchain.md) | toolchain, local profile, synthetic data, contracts, CI |
| [02-wave-1-packages.md](02-wave-1-packages.md) | 43 shared packages, templates, platform base |
| [03-wave-2-trust-core.md](03-wave-2-trust-core.md) | transparency log, HSM signer, IAM, boundary, tier-A integrations |
| [04-wave-3-critical-path.md](04-wave-3-critical-path.md) | rts, bvr, abis, bvi, kits, portal, load harness |
| [05-wave-4-operations.md](05-wave-4-operations.md) | ECC, queue, inventory, AI, mobile suite, register surfaces, candidates, edge, observability |
| [06-wave-5-governance.md](06-wave-5-governance.md) | governance, training, support-admin, diaspora, chaos/DR, runbooks, compliance, reproducibility, manifests, handover |
| [07-wave-6-verification.md](07-wave-6-verification.md) | rehearsal, DR drill, chaos, 5x load, security, a11y/i18n, completeness audit |
| [08-edge-case-matrix.md](08-edge-case-matrix.md) | 118 universal edge cases with stable EC-IDs |
| [09-test-pyramid.md](09-test-pyramid.md) | mandatory test layers and harnesses per language |
| [10-definition-of-done.md](10-definition-of-done.md) | 25-item exit bar for every WP |
| [11-completeness-audit.md](11-completeness-audit.md) | how Wave 6 proves nothing is stubbed; end-to-end verification |
| [12-appendix-module-inventory.md](12-appendix-module-inventory.md) | condensed 69-module inventory, integrations, SLAs |
| [modules/](modules/) | one page per module: purpose, surfaces, entities, APIs, events, integrations, ML, deps, ACs, EC rows, owning WP |
| [TRACEABILITY.yaml](TRACEABILITY.yaml) | seed matrix: module x acceptance criterion x EC row to test IDs (empty until built) |

## How to run a work package

1. Open the wave file, copy the WP row and its done-criteria into the agent brief together with `00-principles.md`, `10-definition-of-done.md`, the relevant `modules/<slug>.md` pages, and the EC rows they list.
2. The agent works only inside the WP's paths, starts from the language template (`packages/service-template-*`), and tags every test with `EC-`, `FR-`, `AC-` IDs.
3. On completion it appends test IDs to `TRACEABILITY.yaml`, updates the WP's runbooks and docs, and opens a PR. `just ci` plus `tools/completeness/scan` must be green.
4. The wave gate reviewer signs `docs/audit/gate-G<n>.md` once every WP in the wave meets `10-definition-of-done.md`.

Portal stack is Path B (Astro, Django, Phoenix LiveView, Tauri, Flutter). Next.js is excluded from this programme.
