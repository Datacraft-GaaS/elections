# Lesson P1: Equipment Check-In and Setup
Module: poll-day-setup | Type: Procedural | Duration: 30 min | Target: PO, DPO
Target audience: PO, DPO | Language: en | Version: 1 | Media: Text + Checklist PDF

## Learning Objectives
By the end of this lesson you will be able to:
1. Verify all biometric hardware is present and tamper-evident seals are intact
2. Boot the M3 device into verified-election-mode (VEM)
3. Confirm network connectivity and sync time with NTP server
4. Run the self-test sequence (BVP/BVR/canary array)
5. Log the equipment check event with HSM attestation

## Required Equipment
- BVP device (biometric verification pad) — serial number on label
- BVR device (biometric verification reader) — charging bay present
- M3 rugged tablet running field-mobile-svc
- Smart card reader + 2 PO/DPO cards
- CANARY token array (17 tokens, sealed pouch)
- Power bank (10,000 mAh minimum)
- Backup battery for BVP
- GSM modem (RTS kit) with SIM

## Step-by-Step Procedure

### Step 1: Seal Verification
Before opening any box, photograph all seals. Log each seal ID in the chain-of-custody form.
```
PO Actions:
  1. Note seal numbers on dispatch manifest vs actual seals
  2. If seal is broken → call support-admin-svc immediately, do NOT proceed
  3. Record: timestamp, GPS coords, seal IDs, officer name
```

### Step 2: Device Boot Sequence
```
a) Power on BVP first, wait for green LED (biometric module ready)
b) Power on BVR dock
c) Insert PO smart card into M3 tablet
d) Launch field-mobile-svc app → select Station Mode
e) Enter station code (e.g., COMMISSION-KE-001-A)
f) System displays: "Verified Election Mode Active — Audit shadow ENABLED"
```

### Step 3: Network and Time Sync
```
a) Check signal strength — minimum 2 bars for 4G
b) Run: Settings → Network → Sync Time (NTP: time.eel.example)
c) Confirm timestamp matches official election time ±1 second
d) If NTP fails: proceed with local clock, flag for reconciliation
```

### Step 4: Biometric Self-Test
```
a) Navigate: field-mobile-verify → Diagnostics → Self-Test
b) Place finger on BVP → confirm "Bozorth3 threshold met: 82%"
c) Place iris near BVR → confirm "ISO 19794-5 compliant"
d) System logs: self_test_passed = true, device_id = M3_XXXXXX
```

### Step 5: Canary Token Verification
```
a) Open sealed canary pouch (supervisor must witness)
b) Access: field-mobile-svc → Security → Canary Status
c) Confirm all 17 tokens report "ACTIVE"
d) Any token showing "TRIGGERED" → station breach, freeze immediately
e) Log: canary_verified = true, token_hashes match dispatch manifest
```

### Step 6: HSM Attestation
```
a) Insert PO smart card → triggers HSM key ceremony
b) System displays: "Audit shadow active — OTEL endpoint: otel.eel.example"
c) Record attestation event:
   auditId: <UUID7>
   proofHash: sha256(station_code + timestamp + device_id)
   M3_device_id: <from device settings>
   correlation_key: falco-equip-check-01
```

## Audit Output
On successful completion, the system emits:
```json
{
  "event": "eel.training.equipment_check.completed.v1",
  "auditId": "018d0e8c-4f7a-6b9c-8000-000000000001",
  "proofHash": "sha256:3f4a9b2c1d0e5f8a7b3c2d4e5f6a7b8c",
  "M3_device_id": "M3-KE-001-A-001",
  "station_code": "COMMISSION-KE-001-A",
  "canary_tokens_verified": 17,
  "biometric_self_test_passed": true,
  "correlation_key": "falco-equip-check-01"
}
```

## Failure Modes
| Symptom | Cause | Action |
|---------|-------|--------|
| BVP red LED on boot | Tamper detected | Do not use device, call support |
| NTP sync fails | No network / wrong NTP | Proceed offline, flag reconciliation |
| Canary token TRIGGERED | Station compromised | Freeze mode, notify observer |
| PO card fails auth | Card damaged / not provisioned | Use backup card, log incident |

## Assessment
Passing threshold: 100% on equipment checklist. No skips permitted.
