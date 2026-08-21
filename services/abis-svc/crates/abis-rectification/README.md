# eel-abis-rectification

Subject-driven biometric correction workflow for the eel ABIS with 30-day SLA.

## Workflow States

```
Submitted -> Triage -> EvidencePending -> Approved/Rejected -> Completed
```

## Request Types and Policy

| Type | Officer Required | Evidence |
|------|-----------------|----------|
| biometric_recapture | Yes | None |
| demographic_update | No | national_id |
| template_removal | Yes | odpc_reference |
| duplicate_merge | No | None |

## SLA

30-day SLA from submission. SLA sweep runs every 15 minutes, logging
warnings at NearBreach (3 days remaining) and Breached (deadline passed).

## Integration Points

- `eel_rectification_requests` table: request lifecycle.
- `template_removal`: writes a tlog entry for audit trail.
- SLA sweep: cron job calling `sweep_sla` every 15 minutes.
- Consumer: operations dashboard, subject portal.

## Usage

```rust
use eel_abis_rectification::{workflow::*, domain::{RectificationRequest, RectificationRequestType}};

let mut req = RectificationRequest::new(subject_id, RectificationRequestType::BiometricRecapture);
to_triage(&mut req)?;
assign_officer(&mut req, officer_id)?;
decide(&mut req, true, "approved", officer_id)?;
complete(&mut req)?;
```
