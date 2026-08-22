# Chaos monthly drill: scheduled resilience exercise

## Purpose

Run the full chaos scenario suite against a staging cluster to verify that every degradation contract, runbook, and SLO burn alert behaves as designed. The monthly drill is the primary mechanism for keeping runbooks honest: a scenario that no longer matches production behaviour is a finding, not a pass.

## Prerequisites

- A staging cluster (`eel-staging`) with the full service topology deployed.
- Chaos Mesh installed and the scenario manifests present under `chaos/scenarios/`.
- Grafana dashboards and PrometheusRule SLO burn alerts deployed.
- An incident ticket open as `<TICKET>` (drill tickets are pre-approved).
- Site Reliability Lead and at least one service owner present to witness.

## Commands

### Step 1: baseline the cluster

```bash
# record baseline SLO burn rates and error budgets:
curl -sf http://prometheus.eel-system.svc.cluster.local:9090/api/v1/query \
  --data-urlencode 'query=sum(rate(eel_http_requests_total{status=~"5.."}[5m]))' \
  | python3 -m json.tool
# confirm all services report healthy:
kubectl get pods -n eel-system | grep -v Running | grep -v Completed || echo "ALL RUNNING"
```

### Step 2: run the scenario suite

```bash
# run each scenario in chaos/scenarios/ against the staging cluster:
for f in chaos/scenarios/eel-*.yaml; do
  echo "=== $f ==="
  kubectl apply -f "$f" -n eel-system
  # wait for the scenario to complete (duration is defined in the manifest)
  sleep 60
  kubectl delete -f "$f" -n eel-system
done
```

### Step 3: verify each degradation contract

```bash
# for each service, confirm it entered the expected degradation mode:
for svc in bvr-svc rts-svc bvi-svc abis-svc; do
  echo -n "$svc mode: "
  kubectl exec -n eel-system deploy/$svc -- \
    wget -qO- http://localhost:9090/metrics \
    | grep 'eel_degradation_mode'
done
# confirm no SLO breach beyond the degradation contract:
curl -sf http://prometheus.eel-system.svc.cluster.local:9090/api/v1/query \
  --data-urlencode 'query=count(eel_slo_burn_rate > 1)' \
  | python3 -m json.tool
```

### Step 4: record findings

```bash
# any scenario that did not behave as its runbook describes is a finding:
# open a ticket per finding and link the runbook that needs updating
```

### Step 5: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.chaos-monthly-drill.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"scenarios_run\":\"<COUNT>\",\"findings\":\"<N>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Every scenario in `chaos/scenarios/` ran to completion.
2. Each service entered the degradation mode its runbook describes.
3. No SLO breach beyond the degradation contract.
4. Findings are tracked as tickets with linked runbook updates.
5. Transparency log contains the drill-completed leaf.

## Rollback procedure

Chaos scenarios are self-terminating and scoped to the staging cluster. If a scenario does not terminate, delete it explicitly with `kubectl delete -f <scenario> -n eel-system`. If a service fails to recover after a scenario, follow its degradation runbook and escalate to the service owner. Production is never touched by this drill.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Service owners for any service that fails to recover
- DR Coordinator (contact via war room bridge) if a scenario reveals a DR gap

## Estimated duration

2 to 4 hours.

## Chaos-drill reference

This drill runs the full suite: `chaos/scenarios/eel-*.yaml`. Each scenario is documented by its corresponding runbook in this directory.
