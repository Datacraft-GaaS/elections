# M04 — VPRS (Voter Pre-Registration System)

**Runs on:** Public web + mobile portal fronted by CDN + WAF; back-end in primary DC
**Owner:** Datacraft — **greenfield build**
**IPR:** Source code, documentation, and IPR transferred to the Commission on completion (Section V explicit mandate).

## 1. Purpose

Let a Kenyan citizen pre-fill their voter registration form online or via a mobile app, verify their identity against IPRS, and receive a QR code that lets a KIEMS clerk complete the biometric portion in minutes rather than the current 15-minute manual process. Cuts polling-centre queuing, boosts registration coverage, and creates the demand signal for enrolment-clerk deployment.

## 2. Personas

- Registered citizen (public applicant)
- Enrolment Clerk (retrieves pre-filled form via KIEMS BVR device)
- the Commission Administrator (analytics, cancellations)
- ODPC Officer (compliance oversight)

## 3. Functional Requirements

- **FR-M04-001** Responsive web portal + progressive-web-app for citizen self-service (Android + iOS accessible).
- **FR-M04-002** Biographic capture form matching the 22 BVR fields (subset of biometric fields deferred to on-device capture).
- **FR-M04-003** IPRS identity verification: applicant enters ID number; system calls IPRS; biographic fields pre-populated for confirmation.
- **FR-M04-004** SMS OTP challenge to the mobile registered against the applicant's ID for verification.
- **FR-M04-005** Consent capture per Kenya DPA 2019: explicit consent for the Commission + IPRS processing, retention terms disclosed, right-to-withdraw explained.
- **FR-M04-006** QR code / handoff token generation once application is verified. Token expires after 30 days; extendable to 90 days on request.
- **FR-M04-007** KIEMS device API: BVR client scans the QR at the polling centre, retrieves pre-populated form via `/vprs/v1/applications/handoff/{token}`.
- **FR-M04-008** Application status portal for citizen to check progress and cancel.
- **FR-M04-009** Administrative back-office: analytics (applications by county, conversion rate), cancellations, exception queues.
- **FR-M04-010** Anti-abuse: reCAPTCHA on entry, rate limit per IP, per ID number; ODPC-safe with no register enumeration.
- **FR-M04-011** WCAG 2.2 AA accessibility on every screen. Screen-reader compatible. Keyboard-only navigable.
- **FR-M04-012** Multilingual: English + Kiswahili at launch. Extensible.
- **FR-M04-013** RBAC + full audit trail + Kenya DPA + ISO 27001:2022 compliance.
- **FR-M04-014** High availability + nationwide scalability (target: 500,000 applications in first month of a registration window).
- **FR-M04-015** Every mutation signed + audit-logged.
- **FR-M04-016** Source code + documentation transferred to the Commission on Operational Acceptance (mandatory Section V clause).

## 4. Non-Functional Requirements

- **NFR-M04-001** Page load p99 ≤ 2 s at national peak.
- **NFR-M04-002** 500,000 applications per day sustainable at peak.
- **NFR-M04-003** IPRS-lookup circuit-breaker: fail-open on read (allow application; verification retried async); fail-closed on submission (block if not verified).
- **NFR-M04-004** OTP delivery ≤ 60 s p95.
- **NFR-M04-005** WCAG 2.2 AA on every screen.
- **NFR-M04-006** Zero PII in browser storage (session-scoped only).

## 5. Integrations

- IPRS (I01)
- SMS Gateway (I08)
- BVR Device (M01) via handoff API
- BVR Back-End (M02) via conversion event
- Audit + Merkle log (P05)

## 6. Data touched

- `VPRSApplication` (create, update)
- `Voter` (link on conversion)
- `AuditEvent` (create)

## 7. UX Flow

1. Applicant visits portal on phone or PC.
2. Enters ID number → OTP to registered mobile → confirms OTP → biographic pre-populated.
3. Applicant confirms + adds missing fields (mobile, email, physical-challenge, education).
4. Applicant selects preferred polling centre from map / list.
5. Consent screen with explicit tick-box + link to full privacy notice.
6. Submits application; receives QR code + reference number.
7. At polling centre, KIEMS clerk scans QR; form auto-populated; biometric captured; enrolment completed.

## 8. Failure Modes

| Failure | Response |
|---|---|
| IPRS down | Fail-open on read; queue verification for retry; block submission until verified |
| SMS delivery fails | Offer alternate MFA (email OTP) or repeat |
| Anti-abuse trip | Rate-limit response; ODPC-audit-safe messaging |
| Bulk enumeration attempt | Block IP; alert |

## 9. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M04-01 | End-to-end apply-to-QR-to-KIEMS-conversion under normal load | UAT |
| A-M04-02 | 500K applications/day sustained | Load test |
| A-M04-03 | WCAG 2.2 AA audit pass | Independent accessibility audit |
| A-M04-04 | IPRS circuit-breaker behaves per spec | Chaos exercise |
| A-M04-05 | ODPC audit rated compliant | ODPC review |
| A-M04-06 | Source code + docs + IPR transferred to the Commission at OAT | Handover certificate |

## 10. Cross-References

- `../PRD.md` §8.1 M04
- `../DATA-MODEL.md` §1.5
- `../SECURITY.md` §3.11
