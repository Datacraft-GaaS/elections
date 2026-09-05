# Lesson P6: Station Management — Closing and Reconciliation
Module: station-management | Type: Procedural | Duration: 30 min | Target: PO, DPO, Support
Target audience: PO, DPO | Language: en | Version: 1 | Media: Text

## Learning Objectives
1. Execute closing procedures in correct sequence
2. Secure all materials, devices, and evidence
3. Complete reconciliation (ballots, votes, inventory, workforce)
4. Submit all reports to governance-svc with audit chain
5. Confirm station security before departure

## Closing Sequence (Must Be Completed in Order — Do Not Skip)

### Phase 1: Verify No Voters Remain (15 min after election close)
```
a) Announce closing via station PA / loudspeaker
b) Lock station door (if permitted by law — confirm with observer)
c) Allow voters already in line to complete voting (up to 30 min)
d) After 30 min: close doors, begin closing procedure
e) PO confirms with DPO: "No voters remain inside station"
```

### Phase 2: Tally Verification (30 min)
```
a) PO and DPO review tally sheets
b) Count ballot papers from ballot box — verify count matches tally records
c) Verify: total ballots counted == valid votes + spoiled + rejected + invalid
   - Valid = voter made correct choice
   - Spoiled = voter accidentally invalidated
   - Rejected = voter made invalid choice (e.g., extra mark)
   - Invalid = ballot not properly marked
d) Confirm tally count against M3 device electronic tally (if available)
e) Both PO and DPO sign tally sheet
f) Photograph tally sheet with station code visible
g) Log: tally_verification_complete
```

### Phase 3: Evidence Security
```
a) Seal ballot box with tamper-evident seals (new seals — record numbers)
b) Place tally sheets in sealed envelope (envelope sealed with station seal)
c) Place biometric logs (USB export from M3) in sealed evidence bag
d) Place canary token array (18th token — the master verification token) in sealed pouch
e) All sealed items placed in secure lockbox (dual-key: PO + observer)
f) Log: evidence_sealed, seal_numbers, evidence_items_count
```

### Phase 4: Inventory Return (30 min)
```
a) Complete closing inventory (P4 — Step 4)
b) Load all devices into dispatch boxes
e) Apply tamper-evident seals to boxes (new seals, different from opening)
d) Record all seal numbers
e) Photograph sealed boxes with station code visible
f) Log: equipment_returned, box_seal_numbers
```

### Phase 5: Workforce End Report (10 min)
```
a) Marshaller completes P5 closing procedures
b) All workers confirm departure with biometric/card tap
c) Submit workforce report to workforce-svc
d) Confirm receipt to marshaller
```

### Phase 6: Final Audit Logging (5 min)
```
a) M3 device performs final audit export
b) System emits: "eel.station.closing.v1" event with full audit chain
c) PO verifies event details on screen (auditId, proofHash, M3_device_id, station_code)
d) Copy audit record to SD card (backup)
e) Log closing time, PO signature, observer signature, station status = SECURED
```

## Security Before Departure
```
Before leaving station:
1. All doors locked
2. All windows closed / locked
3. All evidence in lockbox / sealed
4. All devices in sealed boxes
5. Station lights off (unless required for security)
6. Station camera (if installed) confirmed active
7. GPS tracker (in M3) confirmed active
8. Call governance-svc → confirm station status = CLOSED_SECURED
9. Call security (if required) → confirm no suspicious activity
10. PO and DPO depart together (or with security escort if needed)
11. Send confirmation to governance-svc: station closed, evidence secured, workers departed
```

## Assessment
Complete closing sequence for simulated station (tally + inventory + evidence + workforce + audit). Must handle 1 error scenario (seal broken, tally discrepancy, missing device). Log all events correctly.
