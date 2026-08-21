<title>Automated Chaos Scenario Runner</title>

# Automated Chaos Scenario Runner

Orchestrates all 34 chaos scenarios from `chaos/scenarios/README.md` in sequence (or parallel
where safe), measures SLO compliance, and produces a signed evidence report.

## Purpose

WP-6.3. Runs as part of Wave-6 full-system verification. Also used for weekly staging rehearsals
and monthly pre-prod runs (Wave 5 cadence).

## Modes

### dry-mode

Validates that all Chaos Mesh CRDs are installed, all target pods are running, and all SLO
thresholds are parseable. Does not inject any faults.

```bash
just chaos-runner dry
```

### live-mode

Injects faults sequentially, waits for each scenario to stabilise, records metrics, then tears
down the experiment before starting the next.

```bash
just chaos-runner live ENV=preprod
# Run a subset by scenario number range
just chaos-runner live ENV=preprod SCENARIOS=01-10
```

## Scenario sequencing

Scenarios are grouped to avoid cascading blast radius:

1. Network scenarios (01-03, 11-13): run sequentially
2. Compute scenarios (04-05): run after network group recovers
3. Storage and identity (07-10, 17-18): sequential
4. Application-layer (19-26): sequential
5. Infrastructure (06, 14-16, 27-29): require quiescent system
6. Composites (34): run last; require all services healthy

## Pass/fail logic

Each scenario passes when:
1. The fault is injected and detected by the SLO dashboard within 30 s
2. Relevant alerts fire within the threshold in `sla-manifest.yaml`
3. The system returns to healthy state within the specified RTO
4. No data loss is detected (row counts, sequence numbers, Merkle roots compared pre/post)

## Expected artefacts

Written to `verification/reports/chaos-runner-<date>/`:

- `scenario-results.json` -- per-scenario pass/fail with timestamps and metrics
- `alert-timeline.json` -- Alertmanager alert history during run
- `slo-compliance.json` -- percentage of time each SLO was met
- `data-integrity.json` -- row count and hash comparisons pre/post
- `gate-g6-chaos.sig` -- signed evidence for G6

## Configuration

`sla-manifest.yaml` in `verification/rehearsal/` defines per-SLO thresholds. The chaos-runner
reads this file to determine pass/fail per scenario.

```bash
# Override SLA manifest for a custom run
just chaos-runner live ENV=staging SLA_MANIFEST=chaos/scenarios/sla-override.yaml
```

## Dependencies

- Chaos Mesh >= 2.6 installed with `chaos-testing` namespace
- All 18 services healthy before run starts (pre-flight check enforced)
- `eel-observability` stack with Prometheus remote-write enabled
- `jq`, `kubectl`, `curl` available in PATH

## Limitations

- Does not run composite scenarios concurrently with single-service scenarios
- Kafka-related scenarios require at least 3 broker replicas in the target cluster
- TimeChaos scenarios require node-level permissions (not available in shared cloud environments)

## Related docs

- `chaos/scenarios/README.md` -- scenario catalogue
- `chaos/mesh-experiments/README.md` -- YAML library
- `verification/rehearsal/README.md` -- election-day rehearsal
- `verification/reports/README.md` -- report index
