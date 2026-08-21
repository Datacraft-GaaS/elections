# RTS tally amendment: compensating event and re-declaration

## Purpose

Issue a compensating CloudEvent and trigger re-declaration in `rts-svc` when a tally error is discovered post-certification. Every amendment must be dual-officer signed and recorded in the transparency log. This procedure is auditable and reversible at the event-sourcing level.

## Prerequisites

- The erroneous tally event has been identified: event ID, affected station or ward, and the nature of the error are documented in the incident ticket (`<TICKET>`).
- Two authorised officers present for dual sign-off.
- Access to `rts-svc` admin API and `governance-svc` admin API.
- Access to `grpcurl` and the transparency-svc endpoint.
- Legal and Compliance notified and on record in the ticket.

## Commands

### Step 1: identify the erroneous tally event

```bash
# query rts-svc for the event:
curl -sf "http://rts-svc.eel-system.svc.cluster.local:8080/admin/tally-events?station=<STATION_ID>&date=<DATE>" \
  | python3 -m json.tool
# note the event_id, sequence_number, and certified_at fields
```

### Step 2: verify the error

```bash
# compare against the paper register or original count:
curl -sf "http://rts-svc.eel-system.svc.cluster.local:8080/admin/tally/<EVENT_ID>" \
  | python3 -c "
import sys, json
e = json.load(sys.stdin)
print('Certified totals:', e['totals'])
print('Source:', e['source'], 'Certified at:', e['certified_at'])
"
# document the discrepancy in the ticket before proceeding
```

### Step 3: issue compensating CloudEvent

```bash
# the compensating event negates the original:
curl -sf -X POST \
  http://rts-svc.eel-system.svc.cluster.local:8080/admin/compensate \
  -H "Content-Type: application/json" \
  -d "{
    \"compensates_event_id\": \"<EVENT_ID>\",
    \"reason\": \"<PLAIN_TEXT_REASON>\",
    \"ticket\": \"<TICKET>\",
    \"officer_a\": \"<ID_A>\",
    \"officer_b\": \"<ID_B>\",
    \"corrected_totals\": {<CORRECTED_TOTALS_JSON>}
  }"
# expected response: {"compensating_event_id":"<COMP_EVENT_ID>","status":"accepted"}
```

### Step 4: trigger re-declaration in rts-svc

```bash
curl -sf -X POST \
  http://rts-svc.eel-system.svc.cluster.local:8080/admin/redeclare \
  -H "Content-Type: application/json" \
  -d "{
    \"station_id\": \"<STATION_ID>\",
    \"compensating_event_id\": \"<COMP_EVENT_ID>\",
    \"ticket\": \"<TICKET>\"
  }"
# expected: {"redeclaration_id":"<REDECL_ID>","status":"pending_governance_approval"}
```

### Step 5: notify governance-svc and get dual-officer approval

```bash
curl -sf -X POST \
  http://governance-svc.eel-system.svc.cluster.local:8080/admin/approve-amendment \
  -H "Content-Type: application/json" \
  -d "{
    \"redeclaration_id\": \"<REDECL_ID>\",
    \"officer_a\": \"<ID_A>\",
    \"officer_b\": \"<ID_B>\",
    \"ticket\": \"<TICKET>\"
  }"
# expected: {"status":"approved","effective_at":"<ISO8601>"}
```

### Step 6: emit amendment to transparency log

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"rts.tally.amendment.v1\",\"payload\":{\"compensating_event_id\":\"<COMP_EVENT_ID>\",\"redeclaration_id\":\"<REDECL_ID>\",\"station_id\":\"<STATION_ID>\",\"incident_ticket\":\"<TICKET>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

### Step 7: verify corrected totals are reflected

```bash
curl -sf "http://rts-svc.eel-system.svc.cluster.local:8080/admin/tally/<STATION_ID>/current" \
  | python3 -m json.tool
# totals must match the corrected values from step 3
```

## Verification steps

1. `rts-svc` returns corrected totals for the affected station.
2. Transparency log contains the `rts.tally.amendment.v1` leaf.
3. Governance-svc shows the amendment status as `approved`.
4. Original erroneous event is still present in the log (compensating, not deleted).
5. Audit trail in the ticket includes both officer IDs, timestamps, and tlog leaf hashes.

## Rollback procedure

A compensating event cannot be deleted; the audit trail is permanent. If the compensating event itself contains an error, issue a second compensating event against it. Each iteration requires dual-officer sign-off and a fresh tlog leaf. Document each iteration in the incident ticket.

## Escalation contacts

- Site Reliability Lead (on-call via PagerDuty rotation)
- Legal and Compliance (contact details in incident ticket template)
- DR Coordinator (contact via war room bridge) if multiple stations are affected

## Estimated duration

20 to 40 minutes including dual-officer coordination.

## Chaos-drill reference

This runbook is exercised in tabletop drills only. The closest chaos scenario for rts-svc resilience is: `chaos/scenarios/eel-pod-kill-rts-svc.yaml`
