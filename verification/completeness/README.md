# Completeness audit (WP-6.7)

Final audit per docs/plans/11-completeness-audit.md, sections 9.1 through 9.9.
Runs as a blocking CI job after Wave 6 deliverables land, and on-demand locally.

## Steps

| Step | Script | Checks |
|---|---|---|
| 9.1 | steps/9.1-stub-scan.sh | tools/completeness/scan --strict; zero hits outside allowlist |
| 9.2 | steps/9.2-surface-inventory.sh | Every Appendix-A surface has a deployable artefact |
| 9.3 | steps/9.3-traceability.py | Every TRACEABILITY.yaml row has >= 1 mapped passing test |
| 9.4 | steps/9.4-contract-closure.sh | Pact can-i-deploy passes; every Karapace schema has producer + consumer |
| 9.5 | steps/9.5-endpoint-closure.py | Every OpenAPI path+method: contract, idempotency, DAST, 4-language smoke |
| 9.6 | steps/9.6-runbook-closure.sh | Every service has runbooks + chaos run within 30 days |
| 9.7 | steps/9.7-sla-closure.py | Every Appendix-A SLA measured and met |
| 9.8 | steps/9.8-legal-closure.py | Every statute citation has a test tag |
| 9.9 | steps/9.9-report.py | Renders completeness-report.md + completeness-attestation.json |

## Running

```bash
# Full audit (requires cluster)
bash verification/completeness/run-audit.sh

# Dry run against fixtures (no cluster required)
EEL_AUDIT_MODE=dry bash verification/completeness/run-audit.sh
```

## Dry-run mode

Set `EEL_AUDIT_MODE=dry` to run every step against `fixtures/mini-audit/` synthetic data.
All steps exit 0. Used for CI pipeline validation and local development.

## Output files

After a successful run:
- `completeness-report.json` -- machine-readable aggregate
- `completeness-report.md` -- human-readable report
- `completeness-attestation.json` -- signed attestation for the transparency log

## Attestation

After all 9 steps pass, `steps/9.9-report.py` hashes `completeness-report.json` and submits
it as a `meta-events` leaf on the transparency log. The returned inclusion proof is stored in
`completeness-attestation.json` and becomes the Gate G6 sign-off artefact.

See `completeness-attestation.md` for the full tlog flow.

## Allowlist

`.completeness-allowlist.yaml` at the repo root governs which stub/skip findings are permitted.
Currently only the ClickHouse deferral is allowlisted (ADR-0021, expiry: 2026-12-31).
