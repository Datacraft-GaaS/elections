<title>Wave-6 Verification Programme</title>

# Wave-6 Verification Programme

Full-system verification for the Kenya elections platform. Wave 6 (WP-6.1 to WP-6.7) closes gate
G6 and is the final quality gate before production certification. No production deployment proceeds
without a signed G6 evidence package.

## What Wave 6 proves

| Concern | Harness |
|---------|---------|
| Election-day operational readiness | `rehearsal/` |
| Disaster-recovery RTO/RPO targets | `dr-drill/` |
| All chaos scenarios pass at full scale | `chaos-runner/` |
| System sustains 5x peak load | `load-5x/` |
| ASVS Level 3 + pen-test clean | `security/` |
| WCAG 2.2 AA + 3 languages complete | `a11y-i18n/` |
| All 69 modules implemented per spec | `completeness/` |
| Signed evidence reports | `reports/` |

## Verification modes

Every harness runs in two modes:

- **dry-mode**: validates fixtures, config, and connectivity without touching production data or
  triggering real side-effects. Safe to run in CI on every merge.
- **live-mode**: exercises real services at pre-prod scale with synthetic election fixtures.
  Requires Change Advisory Board approval and a maintenance window.

## Running all harnesses (dry-mode)

```bash
just verify-all dry
```

## Running all harnesses (live-mode, pre-prod)

```bash
just verify-all live ENV=preprod
```

## Gate G6 evidence package

After a successful live-mode run, reports are written to `verification/reports/` and a summary
manifest is signed and appended to the transparency log:

```bash
just gate-g6-sign  # requires transparency-svc running and HSM available
```

## Dependencies

- All 18 services deployed and healthy in the target environment
- `eel-observability` stack running (Prometheus, Grafana, Loki, Tempo)
- Chaos Mesh installed (for chaos-runner)
- Valid election fixtures in `verification/rehearsal/fixtures/`

## Related docs

- `docs/plans/00-principles.md` -- wave definitions and gate criteria
- `docs/audit/gate-G5.md` -- G5 evidence (prerequisite)
- `verification/reports/README.md` -- generated report index
