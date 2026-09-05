# Lesson P4: Inventory Tracking and Asset Management
Module: inventory-tracking | Type: Procedural | Duration: 20 min | Target: PO, DPO, Support
Target audience: PO, DPO, Support | Language: en | Version: 1 | Media: Text

## Learning Objectives
1. Use inventory-svc to log all station assets at opening
2. Report asset damage or loss during election day
3. Track consumables (ballot papers, ink, receipt rolls)
4. Complete end-of-day inventory reconciliation
5. Submit inventory report to governance-svc

## Assets Tracked by inventory-svc
| Asset | Asset ID Format | Serial Field |
|-------|----------------|--------------|
| BVP device | INV-BVP-001-A | BVP-KE-001 |
| BVR device | INV-BVR-001-A | BVR-KE-001 |
| M3 tablet | INV-M3-001-A | M3-KE-001-A-001 |
| GSM modem | INV-MODEM-001 | GSM-KE-001 |
| Smart cards (PO) | INV-CARD-PO | CARD-PO-001 |
| Smart cards (DPO) | INV-CARD-DPO | CARD-DPO-001 |
| Ballot boxes | INV-BOX-001-N | BOX-KE-001-N |
| Ballot papers | INV-BALLOT-N | (count, not serial) |
| Biometric receipt rolls | INV-RECEIPT-N | (count, not serial) |

## Step-by-Step: Opening Inventory Check

### Step 1: Scan All Assets
```
a) Launch field-mobile-svc → Inventory → Opening Check
b) Scan each asset barcode using device camera
c) System matches barcode to manifest from dispatch list
d) Display: Asset INV-BVP-001-A: FOUND ✓
e) If asset not on manifest → add manually, flag for review
f) If expected asset missing → log as MISSING, notify support-admin-svc
```

### Step 2: Condition Assessment
```
For each asset, record condition (1-5 scale):
  1 = New/Perfect, 2 = Good, 3 = Minor wear, 4 = Damaged, 5 = Unusable
a) BVP: check screen, fingerprint sensor, card slot
b) BVR: check iris camera, display, battery
c) M3: check screen, stylus, ports, SIM slot
d) GSM modem: check antenna, SIM, indicator lights
e) Any asset rated 4 or 5 → photograph + incident report
```

### Step 3: Count Consumables
```
a) Ballot papers: count in bundles of 100
b) Ink bottles: count sealed bottles
c) Receipt rolls: count spare rolls
d) Enter counts in inventory-svc form
e) System calculates: total_ballots = bundles × 100
f) Compare against dispatch manifest
g) Any discrepancy → log and photograph
```

### Step 4: Submit Opening Inventory
```
a) Review all entries
b) Tap "Submit Opening Inventory"
c) System generates:
   {
     "event": "eel.inventory.opening_check.v1",
     "auditId": "<UUID7>",
     "station_code": "IEBC-KE-001-A",
     "assets_scanned": 12,
     "assets_missing": 0,
     "assets_damaged": 0,
     "ballot_count": 500,
     "proofHash": "sha256(station_code + asset_count + timestamp)"
   }
d) PO signs on screen (PIN or biometric)
e) Observer counter-signs
```

## During Election Day: Asset Events

### Reporting Damage
```
a) Navigate: field-mobile-svc → Inventory → Report Event
b) Select asset from list
c) Select event type: DAMAGED / LOST / MALFUNCTION
d) Photograph asset (required)
e) Enter description (max 500 chars)
f) Submit → event sent to inventory-svc + governance-svc
g) Log: asset_event_id, timestamp, description
```

### Replacing a Malfunctioning Device
```
1. Call support-admin-svc → request replacement device
2. Support dispatches replacement from nearby station pool
3. PO receives replacement device with sealed tamper-evident bag
4. PO verifies new device serial against dispatch note
5. PO logs swap in inventory-svc:
   {
     "event": "eel.inventory.device_swap.v1",
     "old_device_id": "M3-KE-001-A-001",
     "new_device_id": "M3-KE-001-A-002",
     "reason": "screen_crack",
     "auditId": "<UUID7>"
   }
6. All queued events transferred to new device
7. Old device placed in sealed bag, tagged for return
```

## End-of-Day Reconciliation

### Closing Inventory Check
```
a) Navigate: field-mobile-svc → Inventory → Closing Check
b) Scan all assets again (same as opening)
c) Count remaining ballot papers (spoiled + unused)
d) Enter final counts for all consumables
e) System calculates:
   - Ballots used = Opening count - Closing count
   - Spoiled = Ballots in spoilage box
   - Valid votes = Ballots in ballot box
   - Reconciliation: used + spoiled + unused = opening count ✓
f) Submit closing inventory → triggers governance-svc reconciliation
```

### Asset Return
```
a) All assets placed back in dispatch boxes
b) Tamper-evident seals applied (new seals)
c) Seal numbers recorded in inventory-svc
d) Dispatch vehicle driver signs manifest
e) PO retains copy of sealed manifest
f) System logs: assets_returned, seal_count, driver_id
```

## Assessment
Complete opening inventory for a station with 8 assets, 1 damaged item, and 1 missing item. Must generate correct audit event and submit to governance-svc.
