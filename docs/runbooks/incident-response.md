# Incident response: general procedure

## Purpose

Standard procedure for any production incident at P1 or below. This is the general incident response runbook; for a P0 (data loss, tally corruption, tlog divergence, HSM compromise, platform-wide write outage), use `incident-response-p0.md` instead. The goal is to contain, mitigate, and record every incident, with a clear owner and a post-incident review.

## Prerequisites

- An incident ticket open as `<TICKET>` in the ITSM system.
- On-call rotation contact details (PagerDuty).
- Access to the affected service's logs, metrics, and runbook.
- A war room bridge for any incident involving more than one service.

## Commands

### Step 1: declare and triage

```bash
# open the incident ticket and classify severity (P1/P2/P3):
# P1: election-critical service degraded or down
# P2: cycle-critical service degraded
# P3: routine service issue
# assign an incident commander and record the start time
```

### Step 2: contain

```bash
# stop the blast radius before diagnosing:
# - if a service is misbehaving, scale it down or route around it
# - if a write path is corrupting data, pause the affected consumer
# - if a credential is exposed, revoke it (see break-glass.md)
# record every containment action in the ticket
```

### Step 3: diagnose

```bash
# gather evidence from the affected service:
kubectl logs -n eel-system deploy/<SERVICE> --tail=200
kubectl get events -n eel-system | tail -20
# check SLO burn and error rates:
curl -sf http://prometheus.eel-system.svc.cluster.local:9090/api/v1/query \
  --data-urlencode 'query=sum(rate(eel_http_requests_total{status=~"5.."}[5m]))' \
  | python3 -m json.tool
# consult the service's runbook in this directory for the specific procedure
```

### Step 4: mitigate

```bash
# apply the mitigation from the service's runbook
# if no runbook covers the situation, apply the least-invasive fix and document it
# verify the mitigation with the runbook's verification steps
```

### Step 5: monitor

```bash
# confirm the service returns to normal:
kubectl get pods -n eel-system | grep <SERVICE>
# watch SLO burn and error rates return to baseline
```

### Step 6: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.incident.resolved.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"severity\":\"<SEVERITY>\",\"service\":\"<SERVICE>\",\"root_cause\":\"<CAUSE>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Step 7: post-incident review

```bash
# within 5 working days, hold a review:
# - confirm the root cause
# - identify the runbook gap or process gap
# - open follow-up tickets for corrective actions
# - update the relevant runbook if it did not cover the situation
```

## Verification steps

1. The affected service returns to normal operation.
2. SLO burn and error rates return to baseline.
3. The incident ticket records containment, diagnosis, mitigation, and root cause.
4. Transparency log contains the incident-resolved leaf.
5. A post-incident review is scheduled and follow-up tickets are open.

## Rollback procedure

Rollback is specific to the mitigation applied. If a mitigation made things worse, reverse it and escalate. For service-specific rollback, follow the service's runbook. Never leave a mitigation in place without a documented rollback path.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Service owner of the affected service
- Security Officer (contact via encrypted comms) for any security-related incident
- DR Coordinator (contact via war room bridge) if the incident affects DR posture

## Estimated duration

Varies by severity and root cause. P1 incidents should be mitigated within 30 minutes.

## Chaos-drill reference

This is the general procedure; specific chaos scenarios exercise the service-specific runbooks in this directory. For P0, see `docs/runbooks/incident-response-p0.md`.
