# Chaos Scenarios — Technical Explanation

## Role
Chaos engineering is the discipline of intentionally injecting controlled faults into a distributed system to validate resilience, observe degradation modes, and confirm recovery procedures. For EEL, chaos scenarios verify that the M1-M5 pipeline (biometric capture → verification → transcription → audit → transparency) survives realistic infrastructure failures without corrupting election integrity.

## Purpose
Every scenario maps to one or more SLO thresholds from ADR-0009 (Service Level Objectives) and a degradation mode from ADR-0019 (Degradation Modes). The catalogue ensures that no service operates without a verified failure mode and recovery path before election-day deployment.

## Functionality
- **Application**: `kubectl apply -f` against `chaos-testing` namespace; Chaos Mesh v2.7+ injects faults via `PodChaos`, `NetworkChaos`, `IOChaos`, `StressChaos`, `TimeChaos`, `HTTPChaos`.
- **Observation**: Grafana SLO dashboard (`grafana.internal.eel/d/eel-slo`); Alertmanager; Hubble (Cilium policy); Prometheus metrics from `field-mobile-svc` telemetry.
- **Audit**: Each scenario YAML includes M5 audit chain (`auditId` UUID7, `proofHash` SHA-256, `correlation_key`, `M3_device_id`) linking to detection rules (`falco-rules/`, `wazuh-config/`, `tetragon-policies/`).
- **Recovery**: Per-service runbooks in `runbooks/` and playbooks in `playbooks/`; `chaos-runner` automates execution and validates SLO recovery windows.

## Configuration
- **Namespace**: `chaos-testing` (isolated from `eel-prod`).
- **Tool/CRD**: `chaos-mesh.org/v1alpha1` resources; `action` (`pod-kill`, `disk-fill`, `abort`, etc.); `mode` (`one`, `fixed-percent`, `all`); `selector` by `app` label.
- **Labels**: `eel.io/scenario`, `eel.io/target`, `eel.io/wave` (e.g., `5.9`), `eel.io/adrs` (ADR references), `eel.io/audit`.
- **Schedules**: Most run once (`scheduler` commented) or via `chaos-runner` orchestration (`mesh-experiments/`).
- **Composite**: `eel-site-partition-primary-mirror.yaml` (scenario 34 / 68) tests split-brain and quorum restoration.

## What's in this folder
- `README.md`: Scenario catalogue (68 scenarios, 1-68) with table mapping to service targets, tools, expected results.
- `scenarios/*.yaml`: 68 chaos scenario definitions (34 original + 34 new) — each annotated with M5 audit chain, ADR references, role/purpose/function/config.
- `mesh-experiments/`: Composite YAML files (`eel-stress-chaos-cpu-mem.yaml`, `eel-infra-stress-composite.yaml`, etc.).
- `.gitkeep`: Placeholder for mesh-experiment results.
- `_pod-kill-template.sh`: Template script for manual pod-kill execution.

## Usage
```bash
# Single scenario (example 11 — Cilium misconfig)
kubectl apply -f chaos/scenarios/eel-network-loss-30pct-2s-latency.yaml
# Watch recovery
watch -n 2 'kubectl -n chaos-testing get podchaos'
# Remove
kubectl delete -f chaos/scenarios/eel-network-loss-30pct-2s-latency.yaml
```

## Audit Reference (5-axis)
- **Where**: `chaos-testing` namespace; `chaos-mesh.org/v1alpha1` CRD
- **When**: Before election day (wave 5.9) and during M5 audit drill
- **Who**: Chaos engineering team + SRE (operational); audit by governance-svc observer
- **Why**: Validate ADR-0009 SLOs; verify ADR-0019 degradation modes; prove M5 audit chain integrity
- **How**: Inject via Chaos Mesh → observe Grafana → confirm recovery → log audit event with `proofHash` + `signature`

## References
- ADR 0004 (M3 hardware integration), ADR 0006 (threshold tuning), ADR 0009 (SLO), ADR 0010 (degradation), ADR 0019 (degradation modes)
- Chaos Mesh v2.7 docs: https://chaos-mesh.org
- `docs/runbooks/chaos-recovery/` (per-service recovery steps)
- `platform/argocd/` (deployment of chaos test environments)
