# Lesson P2: Voter Verification Using Biometrics
Module: voter-verification | Type: Procedural | Duration: 45 min | Target: PO, DPO, CLERK
Target audience: PO, DPO, CLERK | Language: en | Version: 1 | Media: Text + Video (embedded)

## Learning Objectives
1. Enroll a voter using ID card scan + biometric capture
2. Execute NBIS v5.0.0 Bozorth3 gallery match with correct threshold
3. Handle pass/fail/adjudication scenarios correctly
4. Record the verification event with full M5 audit chain
5. Manage queue flow to avoid bottlenecks

## Prerequisites
- Equipment check (P1) must be completed and logged
- M3 device must show "Audit shadow: ACTIVE"
- Voter register loaded and synced from governance-svc

## Step-by-Step: Standard Verification Flow

### Step 1: Voter Arrival and ID Check
```
1. Voter presents national ID card (or temporary certificate)
2. Clerk scans barcode or manually enters ID number
3. System queries governance-svc for voter record
4. Display: voter name, photo (if available), registration station
5. PO/DPO verifies photo match physically
6. If voter not on register → follow NOT-FOUND procedure (see Failure Modes)
```

### Step 2: Biometric Capture
```
a) Ask voter to place finger on BVP
b) BVP captures fingerprint → sends to M3 tablet
c) M3 runs NBIS extraction (mindtct → bozorth):
   - mindtct: generates/minutiae from fingerprint image
   - bozorth: compares minutiae against gallery shard
d) Display: "Matching against gallery shard <shard_id>..."
e) System returns: match_score (0-100%), match_decision
```

### Step 3: Decision Logic
```
match_score >= 82% (ADR-0006 threshold):
  → Auto-approve. Display: "VERIFIED ✓"
  → Clerk presses PROCEED. System logs verification event.
  → Voter proceeds to ballot.

82% > match_score >= 60%:
  → PO/adjudication required.
  → DPO takes second fingerprint.
  → If second attempt >= 82% → approve.
  → If second attempt < 82% → manual review.

match_score < 60%:
  → Automatic fail. Do not admit.
  → Voter may file dispute at support-admin-svc kiosk.
  → Log: biometric_fail, score=<value>, reason
```

### Step 4: Recording the Verification Event
```
Every successful verification emits:
{
  "event": "eel.voter.verification.completed.v1",
  "auditId": "<UUID7>",
  "voter_id_hash": "sha256(<national_id>)",
  "station_code": "IEBC-KE-001-A",
  "match_score": 87,
  "match_decision": "AUTO_APPROVED",
  "proofHash": "sha256(voter_id_hash + station_code + timestamp)",
  "gallery_shard_id": "shard-07",
  "M3_device_id": "M3-KE-001-A-001",
  "canary_token_used": "token-03",
  "correlation_key": "falco-biom-verification-01"
}
```

### Step 5: Queue Management
```
a) Maximum 3 voters in verification area at once
b) If queue exceeds 5 → DPO assists with ID checks
c) If queue exceeds 10 → call additional clerk from workforce-svc roster
d) Track queue length in queue-svc dashboard
e) Target: < 3 minutes per voter verification
```

## Managing Difficult Situations

### Voter Refuses Biometric
```
1. Explain process calmly. Reference Elections Act §34.
2. If still refused: offer manual verification with PO + observer witness
3. PO signs manual override form (incident record in field-mobile-svc)
4. Log: override_reason = "voter_refused_biometric"
```

### Voter Has Injury/Damage to Finger
```
1. Attempt alternate finger (thumb, other hand)
2. If multiple attempts fail: use iris verification via BVR
3. If iris also fails: PO/adjudication with witness
4. Log all failed attempts with scores
```

### Voter Has New Biometric (enrollment not in gallery)
```
1. This voter was not previously enrolled in ABIS
2. Check if eligible for on-the-spot enrollment (governance-svc flag)
3. If eligible: new enrollment via field-mobile-verify enrollment flow
4. If not eligible: refer to nearest registration centre
5. Log: enrollment_not_found, voter_referred
```

## Failure Modes
| Symptom | Action |
|---------|--------|
| Voter not on register | Incident screen → unsupported voter → observer signs |
| Gallery shard sync failed | Proceed with cached data, flag for reconciliation |
| Match score 0% | Check fingerprint quality, try alternate finger |
| Device offline | Offline mode with queue-svc local queue |
| PO card removed mid-verify | System pauses, re-authenticate to continue |

## Assessment
Scenario-based test: 5 voters (3 auto-pass, 1 adjudication, 1 fail). Must correctly classify all 5 and complete audit log entries.
