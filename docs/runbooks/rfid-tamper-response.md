# RFID Cover Tamper Response

**Service:** inventory-svc (RFID cover tag subsystem)
**Version:** 1.0
**Last reviewed:** 2026-08-22

Related surfaces:
- Rust crate: `services/inventory-svc/crates/rfid-cover-custody/`
- Django admin: `services/inventory-svc/django/inventory-admin/inventory_admin/apps/rfid_covers/`
- ECC LiveView: `services/ecc-svc/ex/ecc_live/lib/ecc_live_web/live/cover_tamper_live.ex`
- Flutter screen: `services/inventory-svc/apps/custody-mobile/lib/src/screens/rfid_scan.dart`
- Architecture diagram: `docs/features/diagrams/rfid-ballot-box-covers.svg`

Legal basis: Kenya Elections (Technology) Regulations 2017, Regulation 17 (custody chain requirement).

---

## 1. Purpose

This runbook governs the ECC response to a `cover_tamper_alert.v1` event in the eel platform.
A tamper alert fires when the RFID cover-custody crate detects one of:

- Cover-lift: antenna break or tamper-indicator bit set outside an allowed state transition.
- Cover-swap: EPC of the scanned cover does not match the EPC bound to the box body at manufacture.
- Clone attempt: HMAC-SHA256 challenge/response fails (`challenge_valid=false`).
- Anti-replay: the challenge nonce for a given tag appears twice within 5 minutes.

The goal is: confirm, quarantine, and preserve custody chain evidence within the time bounds below.

---

## 2. Prerequisites

| Prerequisite | Location |
|---|---|
| ECC analyst role in Keycloak | `auth.eel.example` realm `eel`, role `ecc_analyst` |
| Access to ECC LiveView | `https://ecc.eel.example/cover-tamper` |
| Access to Django admin | `https://inventory-admin.eel.example/rfid/tamper-triage/` |
| `kubectl` access to `eel-prod` cluster | `platform/kubeconfig/eel-prod.yaml` |
| Knowledge of `services/inventory-svc/docs/CAPABILITIES.md` | M20 + M39 + M65 |

---

## 3. Trigger conditions

Respond when the ECC LiveView `/cover-tamper` panel shows one or more unresolved alerts with:

| Severity | Target response time | ECC action |
|---|---|---|
| CRITICAL | 10 minutes | Immediate field dispatch |
| HIGH | 30 minutes | Senior analyst review + field check |
| MEDIUM | 2 hours | Analyst triage, may resolve remotely |
| LOW | 24 hours | Log review only |

A CRITICAL alert fires when `tamper_indicator=true` on a box in VOTING_ACTIVE or STATION_CLOSED state,
or when `challenge_valid=false` more than twice for the same tag within 10 minutes.

---

## 4. Step-by-step ECC dispatch procedure

### Step 1: Acknowledge the alert

Open the ECC LiveView tamper panel:

```
https://ecc.eel.example/cover-tamper
```

Find the alert card (CRITICAL alerts appear as full cards; others in the table).
Note: `alert_id`, `epc`, `paired_with_box`, `expected_state`, `observed_state`, `geo_lat/lon`, `detected_at`.

### Step 2: Pull the full chain-of-custody for the box

Use the admin box-chain view or the API:

```bash
# Django admin view
https://inventory-admin.eel.example/rfid/box-chain/?ballot_box_id=<BOX_SERIAL>

# Direct API query
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  https://inventory-svc.eel.example/v1/rfid/tags/history?paired_with_box=<BOX_SERIAL> \
  | jq '.transitions[] | {from_state, to_state, officer_id, transitioned_at, chain_valid}'
```

Verify every `chain_valid` is `true`. A `false` entry is a chain break and must be escalated to COMMISSIONER immediately (see escalation matrix, section 7).

### Step 3: Verify challenge status

```bash
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  "https://inventory-svc.eel.example/v1/rfid/tamper-alerts/<ALERT_ID>" \
  | jq '{severity, challenge_valid, tamper_indicator, expected_state, observed_state}'
```

If `challenge_valid=false`: the tag may be cloned. Treat as CRITICAL regardless of initial severity.

### Step 4: Locate the ballot box

Check the geo coordinates from the alert:

```bash
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  "https://inventory-svc.eel.example/v1/rfid/tamper-alerts/<ALERT_ID>" \
  | jq '{geo_lat, geo_lon, paired_with_box}'
```

Cross-reference with the station assignment table:

```bash
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  "https://inventory-svc.eel.example/v1/stations?box=<BOX_SERIAL>" \
  | jq '{station_code, county, constituency, polling_centre}'
```

### Step 5: Dispatch field officer

Use the ECC dispatch system (M16) to assign a field officer to the station.
The LiveView resolve button dispatches via PubSub; the analyst records the officer ID.

For CRITICAL alerts, send a paired (two-officer) team; one to secure the box, one to witness.

Log the dispatch in the resolve form:

```
Alert ID: <ALERT_ID>
Dispatched officer: <OFFICER_ID>
Resolution note: "Dispatched team [officers] to [station] at [time]. Awaiting physical inspection."
```

### Step 6: Quarantine the box (remote)

Until the physical inspection confirms the box is intact, set the cover tag to QUARANTINE:

```bash
curl -s -X POST \
  -H "Authorization: Bearer $ECC_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"officer_id": "<ECC_OFFICER_ID>", "reason": "tamper_alert_<ALERT_ID>"}' \
  https://inventory-svc.eel.example/v1/rfid/tags/<TAG_ID>/quarantine
```

### Step 7: Preserve evidence

Snapshot the Merkle inclusion proof for the tag's state-transition chain:

```bash
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  "https://inventory-svc.eel.example/v1/rfid/tags/<TAG_ID>/merkle-proof" \
  | jq '.' > /tmp/rfid-merkle-<TAG_ID>-$(date +%Y%m%d-%H%M%S).json
```

Email the proof file to `legal@eel.local` with subject: `RFID tamper evidence - <BOX_SERIAL> - <ALERT_ID>`.

### Step 8: Physical inspection by field officers

The dispatched team must:

1. Photograph the box cover seal (HARD-16 physical seal) before touching.
2. Scan the cover tag with a handheld reader and confirm EPC matches `<EPC_FROM_ALERT>`.
3. Confirm the cover-lift antenna response (intact = tag responds; lifted = no response or tamper bit).
4. Photograph the result and upload to the incident ticket.
5. If intact and no tampering found: proceed to step 9 (false alarm).
6. If tampered: secure the box, do not open, call COMMISSIONER (step 7, escalation matrix).

### Step 9: Resolve or escalate

For false alarms:

```bash
curl -s -X PATCH \
  -H "Authorization: Bearer $ECC_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"resolution_note": "Physical inspection confirmed intact. Tag antenna interference from [cause]."}' \
  https://inventory-svc.eel.example/v1/rfid/tamper-alerts/<ALERT_ID>/resolve
```

Or use the Django admin bulk resolve:
```
https://inventory-admin.eel.example/rfid/tamper-triage/
```
Select the alert, choose "Bulk resolve selected alerts", submit.

For confirmed tampering: do not resolve. Escalate per section 7.

### Step 10: Reader health check

After any tamper event, verify the reader that triggered the alert is functioning correctly:

```bash
# Django admin reader health view
https://inventory-admin.eel.example/rfid/reader-health/

# API
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  "https://inventory-svc.eel.example/v1/rfid/readers/health" \
  | jq '.readers[] | select(.reader_serial == "<READER_SERIAL>")'
```

If the reader shows `last_read` more than 30 minutes old or `tag_count=0` despite expected traffic, file a reader hardware incident.

---

## 5. Verification checklist

After resolving or escalating, confirm all items:

- [ ] Alert status in LiveView shows resolved or escalated label.
- [ ] Box chain view shows no broken `chain_valid` entries.
- [ ] Merkle proof snapshot saved and emailed to legal.
- [ ] Field officer report attached to incident ticket.
- [ ] Reader health confirmed for the triggering reader.
- [ ] COMMISSIONER notified if tamper confirmed (see escalation matrix).
- [ ] ODPP notified if criminal damage suspected.
- [ ] Incident ticket closed with resolution note and evidence links.

---

## 6. Rollback: mark false alarm and reinstate box

If a box was quarantined and inspection confirms no tampering:

```bash
# 1. Confirm the previous valid state from the chain
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  "https://inventory-svc.eel.example/v1/rfid/tags/<TAG_ID>/history" \
  | jq '.transitions[-2] | {to_state}'

# 2. Transition tag back to the last valid state
curl -s -X POST \
  -H "Authorization: Bearer $ECC_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "from_state": "QUARANTINE",
    "to_state": "<PREVIOUS_VALID_STATE>",
    "officer_id": "<RO_OFFICER_ID>",
    "reason": "false_alarm_rfid_noise"
  }' \
  https://inventory-svc.eel.example/v1/rfid/tags/<TAG_ID>/transition

# 3. Resolve the alert
curl -s -X PATCH \
  -H "Authorization: Bearer $ECC_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"resolution_note": "False alarm. Physical inspection confirmed intact. Reinstated to <PREVIOUS_VALID_STATE>."}' \
  https://inventory-svc.eel.example/v1/rfid/tamper-alerts/<ALERT_ID>/resolve
```

Verify the new transition appears in the box chain view with a valid chain hash.

---

## 7. Escalation matrix

| Condition | Escalate to | Method | Time limit |
|---|---|---|---|
| CRITICAL alert, any severity | ECC on-call officer | PagerDuty / phone | Immediate |
| Chain break (`chain_valid=false`) | COMMISSIONER | Direct phone + email | 15 minutes |
| Confirmed physical tampering | COMMISSIONER + Commission Legal | Phone + encrypted email | Immediate |
| Evidence of criminal damage or fraud | ODPP Cybercrime Unit | Written referral via COMMISSIONER | Within 1 hour |
| Reader hardware failure at >3 stations | ICT Director | Incident ticket + phone | 30 minutes |

ECC on-call contact: see `operations/contacts.yaml` (role: `ecc_on_call`).
COMMISSIONER contact: see `operations/contacts.yaml` (role: `commissioner`).
Commission Legal: see `operations/contacts.yaml` (role: `legal_officer`).
ODPP: see `operations/contacts.yaml` (role: `odpp_liaison`).

---

## 8. Estimated duration per severity

| Severity | Detection to first response | Detection to resolution |
|---|---|---|
| CRITICAL (confirmed tamper) | 10 min | 4 to 8 hours (physical + legal) |
| CRITICAL (false alarm) | 10 min | 1 to 2 hours |
| HIGH | 30 min | 2 to 4 hours |
| MEDIUM | 2 hours | 4 to 8 hours |
| LOW | 24 hours | 24 to 48 hours |

---

## 9. Chaos drill reference

This runbook is exercised in the monthly chaos drill scenario:

```
chaos/scenarios/eel-inventory-shard-loss.yaml
```

The drill injects synthetic `cover_tamper_alert.v1` events at CRITICAL severity and validates
that the ECC team completes steps 1 through 7 within the time bounds above.

To run a tabletop drill:

```bash
# Inject a synthetic tamper alert via the chaos tool
kubectl -n eel-prod exec -it deploy/chaos-agent -- \
  chaos inject rfid-tamper \
  --box BOX-SIM-001 \
  --severity critical \
  --station SIMULATED-STATION-001

# Watch the LiveView panel for the injected alert
open https://ecc.eel.example/cover-tamper
```

After the drill, review the timeline via:

```bash
curl -s -H "Authorization: Bearer $ECC_TOKEN" \
  "https://inventory-svc.eel.example/v1/rfid/tamper-alerts?tag=SIMULATED&from=<DRILL_START_ISO>" \
  | jq '.alerts[] | {alert_id, severity, detected_at, resolved_at}'
```

---

## 10. Related runbooks

- `docs/runbooks/tamper-topic-response.md` (general tamper topic response)
- `docs/runbooks/incident-response-p0.md` (P0 incident response)
- `docs/runbooks/kit-mass-revocation.md` (mass kit revocation if reader compromise suspected)
- `services/inventory-svc/docs/runbooks/ballot-discrepancy-response.md` (ballot paper discrepancy)
