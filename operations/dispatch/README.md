# Personnel dispatch operations (M33)

## Overview

`field-mobile-svc` (dispatch API) manages 250,000 officer check-ins across all polling stations. This document covers operational procedures: GPS spoof handling, incident response, payroll trigger, and surge scaling.

## Normal operation

On election day, officers check in at their assigned station via the eel field app. Each check-in:

1. Submits GPS coordinates to `field-mobile-svc`.
2. Is validated against the officer's assigned station location (anti-spoofing radius check).
3. Emits a `dispatch.checkin.v1` CloudEvent to Kafka.
4. Triggers payroll accumulation in `support-admin-svc`.

## GPS spoof handling

```bash
# check the spoof detection alert queue:
kubectl logs -n eel-system deploy/field-mobile-svc --tail=50 | grep "GPS_SPOOF\|spoof_detected"

# list flagged check-ins:
curl -sf "http://field-mobile-svc.eel-system.svc.cluster.local:8080/admin/checkins?flag=spoof_suspected" \
  | python3 -m json.tool

# escalate flagged check-ins to field supervisor:
curl -sf -X POST \
  http://field-mobile-svc.eel-system.svc.cluster.local:8080/admin/checkins/<CHECKIN_ID>/escalate \
  -H "Content-Type: application/json" \
  -d '{"escalation_level":"field_supervisor","reason":"gps_spoof_suspected","ticket":"<TICKET>"}'
```

GPS spoof detection compares the submitted coordinates against the GNSS hardware attestation envelope from the device's SPIRE SVID. A deviation beyond the configured radius triggers a flag.

## Payroll trigger

Payroll is triggered automatically when an officer's check-out is confirmed by their presiding officer:

```bash
# confirm check-out (presiding officer action via ECC):
curl -sf -X POST \
  http://field-mobile-svc.eel-system.svc.cluster.local:8080/admin/checkouts/<OFFICER_ID>/confirm \
  -H "Content-Type: application/json" \
  -d '{"presiding_officer":"<PO_ID>","station_id":"<STATION_ID>"}'
# emits dispatch.checkout.confirmed.v1; payroll system picks this up automatically
```

## Incident response

For field incidents (officer not at station, kit failure, civil unrest), escalate via the comms tree in `comms-tree.md`.

```bash
# create a field incident ticket:
curl -sf -X POST \
  http://field-mobile-svc.eel-system.svc.cluster.local:8080/admin/incidents \
  -H "Content-Type: application/json" \
  -d "{\"station_id\":\"<STATION_ID>\",\"type\":\"<INCIDENT_TYPE>\",\"description\":\"<DESCRIPTION>\",\"reporter\":\"<OFFICER_ID>\"}"
```

## Files

| File | Description |
|------|-------------|
| `comms-tree.md` | Who calls whom; backup channels |
| `surge-procedure.md` | Scaling out during T-1 and T-0 |
