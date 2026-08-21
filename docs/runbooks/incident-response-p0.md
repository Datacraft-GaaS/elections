# P0 incident response: war room protocol

## Purpose

Govern the first 60 minutes of a P0 incident: page hierarchy, bridge setup, 5-minute cadence updates, and post-incident tlog attestation of all key decisions. A P0 is declared when one or more of the following conditions hold: data loss confirmed or suspected, election integrity event (tally corruption, transparency log divergence), platform-wide write unavailability, HSM compromise, or any event requiring dual-officer sign-off.

This runbook is distinct from `incident-response.md`, which covers general incidents. Use this one only for P0s.

## Prerequisites

- PagerDuty P0 policy is active (verify at `https://pagerduty.eel.io`).
- War room bridge URL is pinned in the `#eel-ops-p0` channel description.
- Dual-officer authentication is available (required for any production action during P0).
- An incident ticket is open; record as `<TICKET>`.

## Page hierarchy

| Minute | Action |
|--------|--------|
| 0 | Incident Commander (IC) declares P0; opens ticket; starts war room bridge |
| 2 | IC pages Site Reliability Lead and Security Officer via PagerDuty |
| 5 | IC pages DR Coordinator and relevant service lead |
| 10 | IC sends first stakeholder update to `#eel-ops-p0` channel |
| 15 | Escalate to executive sponsor if customer impact confirmed |

## Commands

### Step 1: declare P0 and open ticket

```bash
# create PagerDuty incident via API (or use the web UI):
curl -sf -X POST https://api.pagerduty.com/incidents \
  -H "Authorization: Token token=<PAGERDUTY_TOKEN>" \
  -H "Content-Type: application/json" \
  -d "{
    \"incident\": {
      \"type\": \"incident\",
      \"title\": \"P0: <ONE_LINE_DESCRIPTION>\",
      \"service\": {\"id\": \"<SERVICE_ID>\", \"type\": \"service_reference\"},
      \"urgency\": \"high\",
      \"body\": {\"type\": \"incident_body\", \"details\": \"Ticket: <TICKET>\"}
    }
  }"
```

### Step 2: freeze non-emergency changes

```bash
# pause all ArgoCD auto-sync:
argocd app list -o name | xargs -I{} argocd app set {} --sync-policy none
# or use the write-gate if write traffic must stop:
kubectl patch configmap eel-write-gate -n eel-system \
  --patch '{"data":{"state":"frozen","reason":"P0 incident <TICKET>"}}'
```

### Step 3: establish 5-minute cadence

```bash
# IC posts to Slack every 5 minutes using the template:
# [P0 <TICKET>] T+<MINUTES> | Status: <investigating|mitigating|resolved>
# Impact: <description> | Next update in 5m | Actions taken: <list>
```

### Step 4: record decisions to tlog (required for all production actions)

```bash
# before each significant action during the P0:
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.p0.decision.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"decision\":\"<ONE_LINE_DECISION>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Step 5: run relevant mitigation runbooks

```bash
# select and execute the appropriate runbook based on symptom:
# data loss or DB failure -> pg-primary-loss.md or pgbackrest-pitr.md
# Kafka issues -> kafka-isr-shrink.md
# HSM issues -> hsm-loss.md or break-glass.md
# tlog divergence -> witness-divergence-response.md
# site loss -> dr-failover-primary-to-mirror.md
```

### Step 6: declare resolution

```bash
# when impact is contained and services are stable:
# 1. unfreeze write-gate if frozen:
kubectl patch configmap eel-write-gate -n eel-system \
  --patch '{"data":{"state":"open","reason":"P0 <TICKET> resolved"}}'
# 2. re-enable ArgoCD auto-sync:
argocd app list -o name | xargs -I{} argocd app set {} --sync-policy automated
# 3. emit resolution leaf:
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.p0.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"duration_minutes\":\"<MINUTES>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
# 4. post resolution to #eel-ops-p0 channel
```

### Step 7: schedule post-mortem

```bash
# within 24 hours of resolution, open a post-mortem ticket and schedule
# a blameless review within 48 hours; attach all tlog leaf hashes from
# the incident as evidence
```

## Verification steps

1. All tlog decision leaves are present (query tlog for `meta-events.p0.decision.v1` with the ticket ID).
2. Services are healthy: `dr-failover smoke-test` or equivalent passes.
3. Write-gate is open and ArgoCD sync is re-enabled.
4. Stakeholder update posted to `#eel-ops-p0` confirming resolution.
5. Post-mortem ticket is open.

## Rollback procedure

There is no rollback for a P0 declaration. If a P0 was declared in error, close the PagerDuty incident, remove the write-gate freeze, and note the false positive in the ticket. Use the post-mortem to update detection thresholds.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- DR Coordinator (contact via war room bridge)
- Security Officer (contact via encrypted comms)
- Executive Sponsor (contact details in `docs/runbooks/incident-response.md`)

## Estimated duration

P0 declaration to initial mitigation: 15 to 30 minutes. Full resolution: varies by incident type.

## Chaos-drill reference

This runbook is exercised as the coordination layer for all major chaos scenarios. The closest load-based trigger is: `chaos/scenarios/eel-stress-chaos-cpu-mem.yaml`

War-room protocol is practised in the quarterly chaos drill per `docs/runbooks/chaos-monthly-drill.md`.
