# Lesson P3: Data Upload and Election-Day Sync
Module: poll-day-sync | Type: Procedural | Duration: 25 min | Target: PO, DPO
Target audience: PO, DPO | Language: en | Version: 1 | Media: Text + Slides

## Learning Objectives
1. Understand the offline-first architecture and sync strategy
2. Upload queued verification events when network is restored
3. Verify upload integrity using SHA-256 proof hashes
4. Reconcile uploaded count against local count
5. Handle partial upload failures and retries

## Architecture Overview
```
field-mobile-svc (offline)
  → local SQLite DB stores all events
  → Queue-svc holds pending uploads
  → On network restore: batch upload to governance-svc
  → transparency-svc receives duplicate-free stream
  → audit_shadow records to OTEL
```

## Step-by-Step: Uploading Verification Data

### Step 1: Check Upload Queue Status
```
a) Navigate: field-mobile-svc → Sync → Upload Queue
b) Display shows: Pending count, Total events, Last sync time
c) If count = 0 → no upload needed
d) If count > 100 → schedule upload during low-traffic period
e) Warning: do NOT upload during active voting hours without approval
```

### Step 2: Initiate Upload
```
a) Ensure device has > 40% battery or is plugged in
b) Ensure stable 4G/LTE connection (minimum 2 bars)
c) Tap "Upload Queue"
d) System displays: "Uploading X events... do not power off"
e) Progress bar shows: 0% → 100%
```

### Step 3: Verify Upload Integrity
```
After upload completes, system performs:
a) Hash verification: sha256(uploaded_payload) must match local hash
b) Count reconciliation: uploaded_count == local_pending_count
c) Deduplication: governance-svc rejects duplicates by idempotency_key
d) Display: "Upload verified: X events, 0 duplicates, 0 errors"
```

### Step 4: Post-Upload Confirmation
```
a) governance-svc returns batch receipt:
   {
     "batch_id": "<UUID7>",
     "events_received": X,
     "proofHash": "sha256(batch_id + timestamp)",
     "correlation_key": "governance-upload-batch-01"
   }
b) PO must confirm receipt on screen
c) Local queue clears only after confirmation
d) Log: upload_confirmed, batch_id, timestamp
```

## Scheduled Sync Points
| Time | Action |
|------|--------|
| 05:30 (Pre-open) | Sync voter register, gallery shards |
| 12:00 (Midday) | Upload AM session data |
| 17:00 (Pre-close) | Upload PM verification events |
| 21:00 (Post-close) | Final upload + tally upload |

## Handling Upload Failures

### Network Disconnect Mid-Upload
```
1. System detects disconnect → pauses upload
2. Local events remain in queue (NOT deleted)
3. On reconnect: resume from last successful event
4. Idempotency keys ensure no duplicates
5. Log: upload_paused_reason = "network_disconnect"
```

### Partial Upload (Some Events Fail)
```
1. System identifies failed events (non-2xx response)
2. Failed events remain in queue with retry flag
3. Display: "X succeeded, Y failed. Retrying in 5 minutes..."
4. After 3 retries: flag for manual review
5. Log: upload_partial_failure, failed_event_ids
```

### Storage Full on M3 Device
```
1. System alerts when local DB > 80% capacity
2. Upload queue immediately (do not wait for scheduled time)
3. If still full: export oldest verified events to SD card
4. Call support-admin-svc for guidance
5. Log: storage_warning, events_exported_to_sd
```

## Failure Modes
| Symptom | Action |
|---------|--------|
| Upload returns 409 Conflict | Duplicate — normal, event already recorded |
| Upload returns 500 | Server error — retry after 5 minutes |
| Upload returns 401 | Auth expired — re-authenticate PO card |
| SHA hash mismatch | Data corruption — do not delete local, escalate |

## Assessment
Complete 3 upload scenarios: normal sync, network disconnect mid-upload, partial failure. Must verify hash reconciliation each time.
