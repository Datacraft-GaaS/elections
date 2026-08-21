<title>Verification Reports Index</title>

# Verification Reports Index

This directory holds generated evidence artefacts from Wave-6 verification harnesses. All files
here are outputs; do not edit them manually. Each signed report is appended to the transparency log
after gate G6 approval.

## Directory structure

```
reports/
  rehearsal-<date>/       Election-day rehearsal artefacts
  dr-drill-<date>/        DR drill RTO/RPO measurements
  chaos-runner-<date>/    Chaos scenario pass/fail evidence
  load-5x-<date>/         5x load SLO compliance
  security-<date>/        ASVS and pen-test reports
  a11y-i18n-<date>/       Accessibility and i18n evidence
  completeness-<date>/    Module completeness matrix
  gate-g6/                Aggregated G6 evidence package
```

## Gate G6 evidence package

After all harnesses pass, run:

```bash
just gate-g6-package  # aggregates all latest reports
just gate-g6-sign     # signs the manifest with the transparency-svc HSM key
```

This produces `reports/gate-g6/manifest.json` (SHA-256 hashes of all artefact files) and
`reports/gate-g6/manifest.sig` (Ed25519 signature). Both are appended to the transparency log and
become the permanent audit record.

## Viewing reports

```bash
# Open the latest rehearsal report
open reports/rehearsal-$(ls -t reports/ | grep rehearsal | head -1)/report.html

# Check overall G6 status
just gate-g6-status
```

## Report retention

Reports are retained for the life of the contract plus 7 years per Kenya National Archives and
Documentation Service Act. Velero backs up the pre-prod cluster nightly; the `reports/` directory
is included in the backup scope.

## Adding a new harness

If a new verification harness is added:

1. Write artefacts to a `reports/<harness-name>-<date>/` subdirectory.
2. Include a `gate-g6-<harness>.sig` file.
3. Register the harness in `just gate-g6-package` so its sig is included in the manifest.
4. Update the directory structure table above.

## Related docs

- `verification/README.md` -- Wave-6 programme overview
- `docs/audit/gate-G5.md` -- G5 evidence (predecessor gate)
- ADR-0023 -- transparency log integrity requirements
