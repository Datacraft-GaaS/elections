# Load test weekly drill

## Purpose

Run the load test suite against a staging cluster to verify that every Appendix-A numeric SLA still holds and that no performance regression has been introduced. The weekly drill is the early-warning mechanism for capacity and latency drift before it reaches production.

## Prerequisites

- A staging cluster (`eel-staging`) with the full service topology deployed.
- The load test harness installed: `loadtest/kit-simulator/`, `loadtest/k6/`, `loadtest/gatling/`.
- The local SLA scaling profile at `loadtest/profiles/local.yaml` (default 1/100 scale).
- Grafana dashboards for the services under test.
- An incident ticket open as `<TICKET>` (drill tickets are pre-approved).

## Commands

### Step 1: baseline the cluster

```bash
# record baseline latency and error rates:
curl -sf http://prometheus.eel-system.svc.cluster.local:9090/api/v1/query \
  --data-urlencode 'query=histogram_quantile(0.99, sum(rate(eel_http_request_duration_seconds_bucket[5m])) by (le, service))' \
  | python3 -m json.tool
```

### Step 2: run the kit simulator

```bash
# simulate the election-day kit load at the local scale factor:
cd loadtest/kit-simulator
./run --scale 1/100 --duration 30m --profile loadtest/profiles/local.yaml
# this drives BVI identification, RTS submission, and verifier traffic
```

### Step 3: run the k6 and Gatling suites

```bash
cd loadtest/k6 && k6 run scenarios/election-day.js
cd loadtest/gatling && ./run.sh scenarios/portal-viewers.scala
# these cover portal viewers, verifier, and API load
```

### Step 4: measure against SLA targets

```bash
# compare measured p95/p99 against the Appendix-A targets:
# BVI 1:1 p95 800ms / p99 1.5s @ 18k req/s
# RTS submit p95 500ms / p99 1s @ 60k concurrent
# portal p95 200ms / p99 500ms @ 5M/hr
# any target missed is a finding; open a ticket
```

### Step 5: check for regressions

```bash
# compare p99 against the previous drill's baseline:
# a >10% p99 regression is a finding even if the SLA is still met
```

### Step 6: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.loadtest-weekly-drill.completed.v1\",\"payload\":{\"ticket\":\"<TICKET>\",\"sla_missed\":\"<N>\",\"regressions\":\"<N>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. Every Appendix-A SLA target is met at the local scale factor.
2. No p99 regression greater than 10% versus the previous drill.
3. No service OOMs or crashes under load.
4. Findings are tracked as tickets.
5. Transparency log contains the drill-completed leaf.

## Rollback procedure

A load test that degrades the staging cluster is stopped immediately. Scale the load back to zero and let the cluster recover. Findings are not rolled back; they are tracked as tickets for capacity or code fixes. Production is never touched by this drill.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Performance Lead (contact via war room bridge) for SLA misses
- Service owners for any service that regresses

## Estimated duration

1 to 2 hours.

## Chaos-drill reference

This is a scheduled drill, not a chaos scenario. It is complemented by `docs/runbooks/chaos-monthly-drill.md` for resilience testing.
