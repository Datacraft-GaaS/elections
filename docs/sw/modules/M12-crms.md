# M12 — CRMS (Candidate Registration & Management System)

**Runs on:** Party portal + IEBC RO back-office, primary + mirror DC
**Owner:** Datacraft
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

End-to-end candidate nomination for every elective position (President, Governor, Senator, MP, CWR, MCA). Party officials submit nominations online with biographic + portrait + documents. IEBC Returning Officers clear them against statutory criteria. Cleared nominations feed ballot proof generation for the printer.

## 2. Personas

- Party Official (party HQ user)
- Independent Candidate (self-nomination)
- Returning Officer (RO) clearance role
- IEBC Administrator (gazette, oversight)
- Auditor

## 3. Functional Requirements

- **FR-M12-001** Party portal registration + login with MFA per party official.
- **FR-M12-002** Nomination form per position: biographic, portrait (ICAO-compliant), party affiliation, running-mate (for President/Governor tickets), area, symbol.
- **FR-M12-003** Independent-candidate registration with additional signature-collection evidence upload.
- **FR-M12-004** Integration with voter register (M02): candidate must be a registered voter; system verifies automatically.
- **FR-M12-005** Integration with KRA (via adapter) for tax-compliance check on candidate (advisory).
- **FR-M12-006** Document upload: educational certificates, integrity clearance (EACC), ethics affidavits, party clearance.
- **FR-M12-007** Party quota / gender rule enforcement per Elections Act (e.g., two-thirds gender rule where applicable).
- **FR-M12-008** RO clearance workflow: RO reviews submission, requests additional documents, approves or rejects with reason.
- **FR-M12-009** Clearance status lifecycle: SUBMITTED → DOCS_UPLOADED → RO_REVIEWING → CLEARED / REJECTED → GAZETTED.
- **FR-M12-010** Public gazette publication of cleared candidates with all statutory details.
- **FR-M12-011** Ballot proof generation: PDF layout per position matching printer specifications; symbol + portrait + name + party colour; ballot order per lottery or registration-time policy.
- **FR-M12-012** Ballot proof preview + party sign-off before printer handoff.
- **FR-M12-013** Nominations window enforcement: closes at gazetted deadline; no further submissions accepted.
- **FR-M12-014** Every mutation signed + audit-logged.

## 4. Non-Functional Requirements

- **NFR-M12-001** Party portal p99 ≤ 1 s.
- **NFR-M12-002** Support 50 parties × 500 candidates per general election concurrently during nomination window.
- **NFR-M12-003** WCAG 2.2 AA on public gazette + party portal.
- **NFR-M12-004** RBAC per party + per RO scope.

## 5. Integrations

- BVR Back-End (M02) for candidate voter-status verification
- KRA (optional) for tax check
- EACC (optional) for integrity check
- Ballot printer via signed PDF export
- Audit + Merkle log (P05)
- HSM (P06)

## 6. Failure Modes

| Failure | Response |
|---|---|
| KRA/EACC unreachable | Advisory only; RO can override with reason logged |
| Candidate not on register | Reject with clear message + link to BVV |
| Deadline missed | Refuse submission |
| Party quota violation | Warn; RO may seek waiver per Elections Act |

## 7. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M12-01 | End-to-end nomination + clearance + gazette flow | UAT |
| A-M12-02 | Register cross-check prevents non-voter candidates | Bench |
| A-M12-03 | Party quota rule enforced | Bench |
| A-M12-04 | Ballot proof matches printer spec | Printer sign-off |
| A-M12-05 | Nominations window enforced | Time-based test |
| A-M12-06 | Public gazette accessible with WCAG audit pass | Independent audit |

## 8. Cross-References

- `M02-bvr-backend.md`
- `M11-public-portal.md`
- `../DATA-MODEL.md` §3.3
