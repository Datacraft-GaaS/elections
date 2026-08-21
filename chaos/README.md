<title>Chaos Engineering Programme</title>

# Chaos Engineering Programme

Systematic fault injection for the Kenya elections platform (22M+ voters, 46,229 polling stations,
18 Kubernetes services). Goal: prove that every service meets its SLA under realistic failure
conditions before election day.

## Purpose and wave context

Wave 5 (WP-5.4) mandates a chaos and DR programme covering all tier-A and tier-B services. This
directory contains:

- Chaos Mesh experiment YAML definitions (`mesh-experiments/`)
- Scenario catalogue with expected outcomes (`scenarios/`)
- CI integration scripts run weekly in staging and monthly at pre-prod scale

Gate G5 requires evidence of at least one successful run of each composite experiment plus passing
witness cross-checks.

## Architecture references

| ADR | Decision |
|-----|----------|
| ADR-0004 | Multi-site active-active topology (primary + mirror + edge) |
| ADR-0009 | Observability and SLO definitions |
| ADR-0019 | Resilience tiers and degradation modes |
| ADR-0022 | Chaos and DR rehearsal cadence |
| ADR-0023 | Transparency log integrity requirements |

## Directory layout

```
chaos/
  mesh-experiments/     Chaos Mesh CRD YAML files (NetworkChaos, PodChaos, StressChaos, ...)
  scenarios/            Scenario catalogue with expected results and runbooks
  README.md             This file
```

## Tooling

- **Chaos Mesh 2.x** running in the `chaos-testing` namespace
- **Chaos Mesh Dashboard** at `https://chaos.internal.eel/dashboard`
- `kubectl` + `just` recipes for running individual experiments
- `chaos-runner` in `verification/chaos-runner/` for orchestrated multi-scenario runs

## Running a single experiment

```bash
# Apply one experiment
kubectl apply -f chaos/mesh-experiments/eel-network-loss-30pct-2s-latency.yaml

# Watch status
kubectl get networkchaos -n chaos-testing -w

# Delete after observation
kubectl delete -f chaos/mesh-experiments/eel-network-loss-30pct-2s-latency.yaml
```

## Running composite experiments (staging)

```bash
just chaos-composite staging eel-election-day-composite.yaml
```

## Running full pre-prod monthly rehearsal

```bash
just chaos-rehearsal preprod  # triggers chaos-runner in verification/chaos-runner/
```

## Cadence

| Environment | Frequency | Trigger |
|-------------|-----------|---------|
| Staging | Weekly (Monday 02:00 EAT) | CI cron |
| Pre-prod | Monthly | Manual approval + CI |
| Production shadow | Post-G5 gate only | Change Advisory Board |

## Dependencies

- Chaos Mesh CRDs installed: `NetworkChaos`, `PodChaos`, `StressChaos`, `IOChaos`, `TimeChaos`,
  `KernelChaos`, `HTTPChaos`, `AWSChaos`
- `eel-observability` stack (Prometheus, Grafana, Loki, Tempo) for SLO measurement
- Patroni, MinIO, Valkey, Kafka, SPIRE all deployed and healthy before running experiments

## Related docs

- `chaos/scenarios/README.md` -- full scenario catalogue
- `chaos/mesh-experiments/README.md` -- YAML library reference
- `verification/chaos-runner/README.md` -- automated orchestration
- `verification/dr-drill/README.md` -- DR drill automation
- `docs/runbooks/` -- per-service degradation runbooks
- ADR-0022 -- rehearsal cadence rationale
