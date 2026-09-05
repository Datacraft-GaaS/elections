# Lesson P5: Marshaller — Managing and Deploying Poll Workers
Module: marshaller-station-management | Type: Procedural | Duration: 35 min | Target: PO, DPO
Target audience: PO, DPO | Language: en | Version: 1 | Media: Text + Checklists

## Learning Objectives
1. Understand the marshaller role (station-level workforce deployment)
2. Verify worker assignments from workforce-svc roster
3. Conduct roll call, verify identification, and assign roles
4. Manage shift rotations, breaks, and replacements
5. Document all workforce events with audit chain
6. Handle absentee workers and replacements from reserve pool

## The Marshaller Role
A Marshaller is designated by the PO (with DPO support) from the station staff pool. The marshaller controls station-level workforce deployment but does NOT hold statutory authority (§34 remains with PO). The marshaller reports to workforce-svc via M3 tablet.

## Pre-Shift: Workforce Verification (30 min before open)

### Step 1: Read Roster from workforce-svc
```
a) M3 tablet → Workforce → Station Roster
b) System displays assigned staff for this station:
   - PO (assigned at national level by IEBC)
   - DPO (assigned at county level)
   - Clerks (assigned by constituency)
   - Support staff (assigned by station)
   - Observers (independent, assigned by party/observer org)
c) Download roster to M3 for offline access
```

### Step 2: Verify Identification
```
a) Each worker must present:
   - Work ID card (with photo, name, station assignment)
   - National ID (for identity verification)
   - Evidence of training certification (certificate hash or QR)
b) PO or marshaller verifies all three
c) If any missing → worker not permitted to serve
   - Log: workforce_attendance_missing_document
   - Call support-admin-svc for emergency replacement
```

### Step 3: Role Assignment
```
Based on roster, assign roles:
- PO = Station Lead (statutory authority)
- DPO = Deputy / Verification Lead
- Clerk 1 = Voter register / Biometric verification station 1
- Clerk 2 = Voter register / Biometric verification station 2
- Support 1 = Queue management / Crowd control
- Support 2 = Incident reporting / First aid
- Observer 1 = Party agent (independent observer)
- Observer 2 = Independent observer (if applicable)

Record assignments in workforce-svc:
{
  "event": "eel.workforce.assignment.v1",
  "station_code": "IEBC-KE-001-A",
  "worker_id": "<UUID7>",
  "role_assigned": "CLERK_VERIFICATION",
  "assigned_by": "PO-001",
  "timestamp": "2025-08-09T05:30:00Z"
}
```

## During Shift: Managing Workers

### Roll Call (Every 2 Hours)
```
a) Marshaller conducts roll call at 07:30, 09:30, 11:30, 13:30, 15:30
b) Each worker confirms presence with biometric check or card tap
c) System updates attendance record

d) If worker absent:
   - Check if worker sent message (workforce-svc messaging)
   - If no message within 15 min → consider absent
   - Call reserve pool (workforce-svc reserve roster)
   - Assign replacement (with verification of replacement's cert)
   - Log: absentees, replacement_id, reason
```

### Shift Rotation (Every 4 Hours)
```
a) PO and DPO rotate every 4 hours to maintain alertness (optional, per station policy)
b) Clerks rotate between verification stations to prevent fatigue errors
c) Support staff rotate between queue and incident management

d) All rotations must be documented with timestamps

e) Log: rotation_event, from_role, to_role, worker_id
```

### Break Management
```
a) Each worker entitled to 30-minute break every 4 hours (Elections Act provisions)
b) Breaks scheduled to ensure minimum 2 clerks and 1 support per verification station
c) No single verification station operates with < 2 clerks at any time

d) Log: break_start, break_end, worker_id, station_section
```

### Dealing with Reserve Pool Replacements
```
When a worker is absent or needs replacement:
1. Marshaller opens workforce-svc → Reserve Pool
2. Filter by role (e.g., CLERK_VERIFICATION), station type, language proficiency
3. Select replacement → verify replacement identification (Step 2 above)
4. Verify replacement training cert (certificate hash or QR scan)
5. Assign role → log assignment → notify PO
6. Replacement must complete 15-minute orientation before taking station
```

### Incident Reporting by Workers
```
Workers report incidents through field-mobile-svc:
- Incident type: SECURITY / MEDICAL / EQUIPMENT / CROWD / PROCEDURE
- Report includes: worker_id, station_code, timestamp, description, photo (if applicable)
- System routes to:
  - Security → falco-rules / wazuh / canary (correlation_key)
  - Medical → medical_app → dispatch (krcc/moH/ndmu)
  - Equipment → inventory-svc
  - Crowd → governance-svc / observer
  - Procedure → training-svc / PO
```

## End of Shift: Workforce Report

### Step 1: Final Roll Call
```
a) All assigned workers confirm presence
b) Any worker who left early → log departure_time, reason
c) Replacement workers confirm completion of full shift (or partial)
```

### Step 2: Submit Workforce Report
```
Marshaller submits:
{
  "event": "eel.workforce.endofshift.v1",
  "station_code": "IEBC-KE-001-A",
  "workers_present": 8,
  "workers_absent": 1,
  "workers_replaced": 1,
  "attendance_rate_pct": 88.9,
  "roll_call_count": 6,
  "auditId": "<UUID7>",
  "proofHash": "sha256(station_code + worker_count + roll_calls)",
  "correlation_key": "workforce-shift-report-01"
}
```

### Step 3: Confirm to Workforce-svc
```
a) M3 sends report to workforce-svc
b) governance-svc updates station status
c) transparency-svc records event for audit
d) workforce-svc confirms receipt → marshaller receives confirmation
```

## Assessment
Marshalling scenario: 8 assigned workers, 1 absent at 09:30, 1 equipment failure at 11:30. Must correctly identify replacement from reserve pool, document rotation, and submit end-of-shift report with full audit chain.
