# M09 — RTS Returning Officer / Constituency Tally Module

**Runs on:** Tallying tablet (constituency + county workstations)
**Owner:** Datacraft
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

Convert scanned Forms 34A into signed constituency-level Form 34B and county / national Form 34C. Provide the human-in-the-loop review of ICR extractions from the RTS back-end. Enable party agent + auditor countersignature workflow. Publish forms to the Public Results Portal.

## 2. Personas

- Returning Officer (RO) — signs Form 34B per constituency
- County Returning Officer (CRO) — signs Form 34C per county
- IEBC national tally officer — Form 34C national
- Party Agent — witnesses + countersigns
- Independent Auditor — witnesses + audits

## 3. Functional Requirements

- **FR-M09-001** RO / CRO login with biometric + MFA on assigned workstation or tallying tablet.
- **FR-M09-002** Dashboard: expected stations vs reported stations, per elective position, live.
- **FR-M09-003** Form 34A review queue: ICR extraction result vs signed image side-by-side; RO confirms or edits with reason.
- **FR-M09-004** Human review mandatory for low-confidence ICR (below configured threshold) and randomly sampled high-confidence forms.
- **FR-M09-005** Aggregation into Form 34B: constituency-level totals per candidate per position, derivation trail traced to contributing Form 34A IDs.
- **FR-M09-006** Aggregation into Form 34C: county / national totals with cross-check against 34B contributions.
- **FR-M09-007** Validation rules enforced: votes cast ≤ registered voters; rejected + valid = total votes cast; non-negative counts.
- **FR-M09-008** RO signs Form 34B with biometric + HSM signature.
- **FR-M09-009** Party agents countersign with biometric or issued PIN.
- **FR-M09-010** Independent auditor countersignature for public transparency.
- **FR-M09-011** Publish signed Form 34B / 34C to RTS back-end (M10) → Public Portal (M11) with Merkle-inclusion proof.
- **FR-M09-012** Export in PDF + CSV per gazette template.
- **FR-M09-013** Dispute-handling workflow: an agent can flag a Form 34A / 34B for dispute; workflow escalates to national tally.
- **FR-M09-014** Session recording: RO screens recorded during the count for post-election forensic review.
- **FR-M09-015** Emergency close of constituency count with justification + dual-control.

## 4. Non-Functional Requirements

- **NFR-M09-001** Form 34B generation ≤ 60 s per constituency after last station reports.
- **NFR-M09-002** UI response ≤ 500 ms.
- **NFR-M09-003** Zero silent auto-tally; every constituency total has RO signature or explicit skip with justification.
- **NFR-M09-004** Session recording encrypted with dual custody.

## 5. Integrations

- RTS Back-End (M10)
- Public Portal (M11)
- HSM via IEBC signing service (P06)
- Audit + Merkle log (P05)

## 6. Failure Modes

| Failure | Response |
|---|---|
| ICR misread | Human review; original image authoritative |
| Missing station | Wait; escalate after configured wait time; RO decides count-with-partial or await |
| Party agent absence | Continue with RO signature; agent may countersign later within window |
| Signature refusal by agent | Log refusal + reason; dispute channel opened |
| Aggregation arithmetic anomaly | Refuse to sign; escalate to national |
| Dispute flag | Route to national dispute board; no unilateral publication |

## 7. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M09-01 | Constituency Form 34B produced within 60 s after last station | UAT |
| A-M09-02 | ICR low-confidence forms routed to human review | UAT |
| A-M09-03 | Party agent countersignature flow works | UAT |
| A-M09-04 | Validation rules enforced (non-negative, sum-consistency) | Bench |
| A-M09-05 | Session recording encrypted with dual custody | Bench |
| A-M09-06 | Merkle-inclusion proof retrievable per published form | Public verification |

## 8. Cross-References

- `M08-rts-device.md`
- `M10-rts-backend.md`
- `M11-public-portal.md`
