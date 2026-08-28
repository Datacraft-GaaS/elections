# M08 — RTS Device Client (Results Transmission on the KIEMS Kit)

**Runs on:** KIEMS kit + legacy Android with feature-matrix
**Owner:** Datacraft
**IPR:** Custom Software vests in IEBC per GCC 15.4

## 1. Purpose

At close of poll, the Presiding Officer scans the completed Form 34A (and equivalents for other elective positions), the kit binds the scan to the station + election + kit + signer, transmits it over telco VPN with signed retry semantics, and provides an SMS/USSD fallback so results reach the tally centre even when data connectivity fails.

## 2. Personas

- Presiding Officer (PO) — sole authorised signer
- Party Agent — witnesses scan + countersigns
- Deputy PO — assists under PO authority

## 3. Functional Requirements

- **FR-M08-001** Scan Form 34A on the KIEMS device scanner with auto-alignment, double-feed detection, encrypted PDF output.
- **FR-M08-002** Support all elective-position forms: 34A (President), 35A (Governor), 36A (Senator), 37A (Woman Rep), 38A (MP), plus MCA + referenda equivalents. Automatic form naming by polling-station code + election + position.
- **FR-M08-003** Image quality validation: reject and prompt re-scan if OCR-hostile (skew, blur, missing corners, resolution below spec).
- **FR-M08-004** Embed per-station QR code + digital signature at scan time. QR binds station + kit + form + election event.
- **FR-M08-005** PO biometric signature required to authorise transmission. Party agent(s) may add countersignature via biometric or issued PIN.
- **FR-M08-006** Transmit to RTS back-end (M10) over telco VPN with mTLS.
- **FR-M08-007** Retry on network loss with exponential backoff + jitter; local signed queue survives kit reboot.
- **FR-M08-008** SMS / USSD signed fallback: on data-unreachable, transmit compressed signed Form 34A summary via SMS short-code; back-end reconstructs.
- **FR-M08-009** Regular event reports during and after transmission: opening, closing, periodic turnout, incident, transmission attempt, transmission success/failure.
- **FR-M08-010** Local audit log of every submission attempt with signature + timestamp + GPS fix.
- **FR-M08-011** Post-transmission local storage of the signed form image; retained until end-of-election-cycle.
- **FR-M08-012** Confirmation slip printed (or displayed) showing form ID, hash, transmission status, transparency-log index.
- **FR-M08-013** Party agent view: agent may verify that the form transmitted matches the physical form via QR scan on their own phone.
- **FR-M08-014** LoRa mesh option: if enabled for high-risk stations, transmit via constituency-level uplink node as third-tier fallback.

## 4. Non-Functional Requirements

- **NFR-M08-001** Scan + sign + transmit cycle ≤ 90 s per form under normal conditions.
- **NFR-M08-002** SMS fallback delivers a compressed signed submission ≤ 2 minutes end-to-end.
- **NFR-M08-003** Zero data loss on power failure.
- **NFR-M08-004** Bandwidth budget: forms transmitted at ≤ 200 KB compressed on average.

## 5. Integrations

- RTS Back-End (M10)
- Telco VPN (I03)
- SMS/USSD gateway (I08)
- HSM via kit certificate (P04, P06)
- Local audit + Merkle log (P05)

## 6. Failure Modes

| Failure | Response |
|---|---|
| Scanner jam | UI prompts; scan retry |
| Poor image quality | Reject; re-scan |
| Network unreachable | Queue; retry; escalate to SMS/USSD fallback after N tries |
| Party agent absent | Continue with PO signature; agent countersignature added later |
| Kit revoked mid-transmission | Halt; PO uses paper backup |
| Signature mismatch on submission | Back-end rejects; kit re-signs and retries |

## 7. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M08-01 | End-to-end scan+sign+transmit ≤ 90 s | Bench |
| A-M08-02 | SMS fallback delivers signed submission | Field trial |
| A-M08-03 | Party agent QR verification confirms match | UAT |
| A-M08-04 | Local queue survives kit reboot | Chaos exercise |
| A-M08-05 | Multi-carrier failover in ≤ 30 s | Live carrier flap |
| A-M08-06 | Confirmation slip includes transparency-log index | Manual test |

## 8. Cross-References

- `M10-rts-backend.md`
- `../SECURITY.md` §3.5, §3.9
- `../THREAT_MODEL.md` §12–13
