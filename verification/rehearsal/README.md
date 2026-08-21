<title>Election-Day Rehearsal Harness</title>

# Election-Day Rehearsal Harness

Full end-to-end simulation of election day: voter verification, results transmission, transparency
log writes, and public portal tally display. Covers the entire M08-M11 critical path (22M voters,
46,229 polling stations, 80 k events/sec peak).

## Purpose

WP-6.1. Produces signed evidence that the system can run a complete election cycle without manual
intervention. Required for gate G6.

## Modes

### dry-mode

Validates all fixtures, service connectivity, and config without committing any results to
databases or the transparency log. Runs in CI on every merge to `main`.

```bash
just rehearsal dry
# or
cd verification/rehearsal && ./orchestrator.sh --mode dry
```

Duration: ~3 min. No external side-effects.

### live-mode

Exercises real services with synthetic election fixtures at pre-prod scale. Writes to the
pre-prod transparency log. Requires Change Advisory Board approval.

```bash
just rehearsal live ENV=preprod ELECTION_ID=rehearsal-2026-08
# or
cd verification/rehearsal && ./orchestrator.sh --mode live --env preprod --election rehearsal-2026-08
```

Duration: ~4 h (simulated election day compressed 6:1). Requires maintenance window.

## Fixtures

Located in `verification/rehearsal/fixtures/`. Contents:

| Fixture file | Description |
|---|---|
| `election-config.yaml` | Election meta: 47 counties, 290 constituencies, 1,450 wards |
| `polling-stations.csv` | 46,229 stations with coordinates and kit serial numbers |
| `voter-roll-sample.csv` | 500 k synthetic voters (subset of 22M for rehearsal) |
| `candidates.yaml` | Synthetic candidates per position |
| `result-forms.json` | Pre-signed Form 34A/34B templates |
| `biometric-probes.bin` | Synthetic fingerprint probes for ABIS pipeline |

## Expected artefacts

After a successful live-mode run, the following artefacts are written to
`verification/reports/rehearsal-<date>/`:

- `sla-results.json` -- pass/fail per SLO in `sla-manifest.yaml`
- `transparency-log-checkpoint.json` -- Merkle root at end of rehearsal
- `witness-crosscheck-report.json` -- output of `witness-crosscheck.sh`
- `report.html` -- human-readable summary from `report-template.md.tmpl`
- `gate-g6-rehearsal.sig` -- Ed25519 signature over SHA-256 of `sla-results.json`

## Pass criteria

- All SLOs in `sla-manifest.yaml` green
- Zero unsigned transparency log entries
- Witness cross-check: 3 of 3 witnesses agree on final Merkle root
- rts-svc p99 submission latency below 500 ms at peak
- Portal tally visible within 30 s of results transmission

## Extending fixtures

```bash
# Regenerate synthetic voter roll
uv run python verification/rehearsal/fixtures/gen_voters.py --count 500000 --seed 42

# Regenerate biometric probes
cargo run -p eel-biometric-fixtures -- --count 10000 --out fixtures/biometric-probes.bin
```

## Dependencies

- All 18 services deployed in target environment
- `eel-observability` stack
- Valid signing key in OpenBao at path `pki/election/rehearsal`
- Chaos Mesh NOT running during rehearsal (run chaos-runner separately)

## Related docs

- `verification/chaos-runner/README.md`
- `verification/dr-drill/README.md`
- `verification/reports/README.md`
- `docs/runbooks/rts-svc-election-day.md`
