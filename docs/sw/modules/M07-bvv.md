# M07 — BVV (Biometric Voter Verification)

**Runs on:** Public web + mobile portal + the Commission kiosk terminals
**Owner:** Datacraft
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Statutory 30-day public inspection of the voter register. Any citizen can confirm their own entry and file objections against errors or omissions, without exposing the register to bulk enumeration.

## 2. Personas

- Public voter (self-lookup)
- the Commission kiosk operator (in-person assistance)
- the Commission objections officer (processes filed objections)
- ODPC Officer (compliance oversight)

## 3. Functional Requirements

- **FR-M07-001** Public self-lookup by ID number + OTP to registered mobile.
- **FR-M07-002** the Commission kiosk self-service with QR-scan or manual ID entry.
- **FR-M07-003** Return record: name, DOB (masked to year), polling station name + code, ward, constituency, county, status.
- **FR-M07-004** Mask sensitive fields per privacy policy (biometrics never displayed, full DOB masked, mobile masked).
- **FR-M07-005** File objection on own record (rectification, transfer, removal request) with audit trail.
- **FR-M07-006** File objection on another's record with justification + evidence upload.
- **FR-M07-007** Objection tracking: citizen can see status of filed objection.
- **FR-M07-008** the Commission back-office: objection queue, adjudication workflow, response templates.
- **FR-M07-009** Anti-abuse controls: strict rate limiting per IP + per session; captcha; alerts on enumeration patterns.
- **FR-M07-010** Per-lookup audit entry recording actor, target voter, timestamp, IP.
- **FR-M07-011** Multilingual: English + Kiswahili.
- **FR-M07-012** WCAG 2.2 AA + screen-reader support.
- **FR-M07-013** Kenya DPA compliance: consent, purpose disclosure, retention policy shown.

## 4. Non-Functional Requirements

- **NFR-M07-001** Lookup p99 ≤ 2 s.
- **NFR-M07-002** Support 100,000 concurrent unique users during peak inspection days.
- **NFR-M07-003** No bulk export possible via UI; rate limit enforces per-session cap of 20 lookups/day.
- **NFR-M07-004** Every lookup logged permanently.

## 5. Integrations

- BVR Back-End (M02) for register reads
- SMS Gateway (I08) for OTP
- Audit / Merkle log (P05)
- the Commission objections officer back-office (Datacraft-built)

## 6. Data touched

- `Voter` (read, redacted)
- `AuditEvent` (write per-lookup)
- Objection entity (create, update)

## 7. Failure Modes

| Failure | Response |
|---|---|
| Enumeration attempt | Rate-limit + IP block + alert |
| OTP delivery failure | Retry / email fallback |
| Register-lookup slow | Cache last N lookups; degrade gracefully |

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M07-01 | 100K concurrent lookups sustained | Load test |
| A-M07-02 | Bulk-enumeration attempt blocked | Red team |
| A-M07-03 | ODPC audit passes | ODPC review |
| A-M07-04 | Objection workflow closes cases within SLA | UAT |
| A-M07-05 | Per-lookup audit entry present | Audit trace |

## 9. Cross-References

- `M02-bvr-backend.md`
- `../SECURITY.md` §3.11
