# e-Learning operations (M38 LMS)

## Overview

The eel LMS (`training-svc`) delivers mandatory poll-worker training. Completion is a gate to M33 dispatch: a worker without a valid completion certificate cannot be dispatched to a station. This document covers content freeze windows, T-30 mandatory completion enforcement, offline sync for remote counties, certificate issuance monitoring, and the gate to dispatch.

## Key dates and milestones

| Milestone | Action |
|-----------|--------|
| T-60 days | Content freeze; all training modules locked for production release |
| T-45 days | Offline sync packages generated and distributed to county servers |
| T-30 days | Mandatory completion deadline; workers without valid cert are blocked from dispatch |
| T-14 days | Final certification report submitted to election commission |
| T-0 (election day) | LMS read-only; no new enrolments or assessments |
| T+7 days | Post-election LMS audit; cert records archived |

## Content freeze window

During the content freeze (T-60 to T-0):

- No new module versions may be published without Security Officer approval.
- All module releases must be HSM-signed (see `content-approval-workflow.md`).
- Emergency content corrections require a dual-officer override and a new signed release bundle.

## T-30 mandatory completion enforcement

```bash
# generate completion report:
curl -sf http://training-svc.eel-system.svc.cluster.local:8080/admin/completion-report \
  -H "Content-Type: application/json" \
  -d '{"as_of":"<ISO8601_DATE>"}' \
  | python3 -m json.tool

# identify workers without valid cert:
curl -sf "http://training-svc.eel-system.svc.cluster.local:8080/admin/incomplete?as_of=<ISO8601_DATE>" \
  | python3 -m json.tool

# block uncertified workers from dispatch (dispatch-gate checks this automatically):
# the gate is enforced in field-mobile-svc/dispatch; no manual step required
# verify gate is active:
curl -sf http://field-mobile-svc.eel-system.svc.cluster.local:8080/admin/dispatch-gate \
  | grep '"training_cert_required"'
# must return "training_cert_required":true
```

## Offline sync for remote counties

Counties with unreliable internet access receive a pre-packaged offline sync bundle:

```bash
# generate offline bundle for a county:
curl -sf -X POST \
  http://training-svc.eel-system.svc.cluster.local:8080/admin/offline-bundle \
  -H "Content-Type: application/json" \
  -d '{"county_id":"<COUNTY_ID>","valid_until":"<ISO8601_DATE>"}' \
  -o /tmp/offline-bundle-<COUNTY_ID>.tar.gpg
# bundle is GPG-encrypted with the county server's public key
# deliver via secure courier or encrypted transfer

# verify bundle integrity on the county server:
gpg --verify /tmp/offline-bundle-<COUNTY_ID>.tar.gpg.sig \
  /tmp/offline-bundle-<COUNTY_ID>.tar.gpg
```

Offline completions are synced back to `training-svc` when connectivity is restored. The sync is idempotent; duplicate submissions are deduplicated by the LMS.

## Certificate issuance monitoring

```bash
# monitor cert issuance rate:
curl -sf http://training-svc.eel-system.svc.cluster.local:9090/metrics \
  | grep 'eel_training_certs_issued_total'

# check for cert issuance failures:
kubectl logs -n eel-system deploy/training-svc --tail=50 | grep "cert_error\|issuance_failed"

# list workers whose certs expire before election day:
curl -sf "http://training-svc.eel-system.svc.cluster.local:8080/admin/expiring-certs?before=<ELECTION_DAY_ISO8601>" \
  | python3 -m json.tool
```

## Gate to M33 dispatch

The dispatch gate in `field-mobile-svc` checks `training-svc` before confirming any dispatch assignment:

1. Worker's certificate is valid and not expired.
2. Certificate covers the role being dispatched (BVR officer, BVI officer, RTS officer, presiding officer).
3. Completion date is within the allowed window (not older than 12 months).

If any check fails, the dispatch assignment is rejected with a `TRAINING_CERT_INVALID` error and the worker is flagged in `support-admin-svc` for follow-up.

## Files

| File | Description |
|------|-------------|
| `content-approval-workflow.md` | Reviewer roles, fact-check gate, HSM-signed release bundles |
