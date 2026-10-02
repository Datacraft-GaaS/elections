# M05 — BVI Device Client (Polling-Day Voter Identification)

**Runs on:** KIEMS kit + legacy Android with feature-matrix
**Owner:** Datacraft
**IPR:** Custom Software vests in the Commission per GCC 15.4

## 1. Purpose

Authenticate every voter presenting at a polling station on election day. Confirm they are on the register for that specific station, prevent double-voting, redirect misdirected voters, transmit polling-day events to the back-end, and never let software failure block the vote (paper backup always exists).

## 2. Personas

- Presiding Officer (PO) — primary user, holds the kit
- Deputy PO / Clerk — assists under PO authority
- Voter — presents ID, is identified

## 3. Functional Requirements

- **FR-M05-001** PO opens polling session with biometric authentication + timestamp + HSM signature; kit locks to the assigned polling station.
- **FR-M05-002** Configurable poll-open / poll-close window per Elections Act; kit enforces window.
- **FR-M05-003** Voter authentication primary: fingerprint (any of 10). Fallback: iris → facial → MRZ card scan → name+ID lookup with justification.
- **FR-M05-004** On successful identification, kit displays voter's registered polling station. If wrong station → REDIRECT screen with correct station name + GPS map link.
- **FR-M05-005** Prevent double-voting: kit marks voter as AUTHENTICATED and refuses re-authentication in same session.
- **FR-M05-006** Geo-fence enforcement: kit refuses BVI operation if GNSS fix falls outside station polygon (multi-constellation + IMU cross-check).
- **FR-M05-007** Time-fence enforcement: kit refuses BVI operation outside gazetted polling window.
- **FR-M05-008** Live on-device dashboard: election name, current time vs open/close, turnout %, voters authenticated, kit serial, battery, connectivity, GPS fix.
- **FR-M05-009** Emergency close per Elections Act §55 with justification + PO biometric + supervisor biometric (dual-control).
- **FR-M05-010** Log every identification event (voter_id, modality, outcome, timestamp, GPS fix, kit signature) to local audit + transmit to BVI back-end.
- **FR-M05-011** No biometric templates are re-transmitted per event; only voter_id + outcome.
- **FR-M05-012** Distress finger triggers covert alert to war-room while displaying normal auth.
- **FR-M05-013** MRZ fallback: if voter's biometrics fail after N attempts, PO can scan ID card MRZ + record justification.
- **FR-M05-014** Register pre-load: at kit-prep time, station-specific register extract is loaded onto kit; refreshed if register updates before poll.
- **FR-M05-015** Kit operates fully offline; queues transmitted events with retry.
- **FR-M05-016** Rejected voter (not on register / wrong station / already voted) receives printed receipt with reason code and referral.
- **FR-M05-017** Accessibility: audio-guided identification, tactile overlay, high-contrast mode, assistive Braille card lookup.
- **FR-M05-018** Party agent read-only view: agents can view turnout counters (no PII, no biometric).
- **FR-M05-019** Kit reset / recovery requires dual-control (PO + Deputy PO biometric).
- **FR-M05-020** Silent transmission of kit health telemetry (battery, network, temperature) at least every 5 min.

## 4. Non-Functional Requirements

- **NFR-M05-001** Voter authentication ≤ 3 s fingerprint; ≤ 5 s iris; ≤ 5 s facial.
- **NFR-M05-002** Kit sustains 700 authentications in ≤ 10 hours polling day on primary battery + attached power bank.
- **NFR-M05-003** Zero data loss on power failure or crash.
- **NFR-M05-004** Kit operable in 0–45 °C, humidity 15–95%, direct sunlight (screen readability).
- **NFR-M05-005** No blocking of voter throughput on software failure — paper backup always available.

## 5. Integrations

- BVI Back-End (M06) for event transmission
- BVR register extract (M02)
- Geo-fence (P01)
- Kit certificate (P04)
- Distress channel (SMS/USSD)
- MDM (O01)

## 6. Failure Modes

| Failure | Response |
|---|---|
| Fingerprint sensor fail | Fallback to iris → facial → MRZ; PO logs |
| Network loss | Queue events; UI shows OFFLINE; continue authentication |
| Kit unresponsive | Reboot via hardware key; log; restore session |
| Voter not found | Fail-fast; PO checks register; possibly redirect |
| Duplicate identification | Refuse; alert; log |
| Off-fence detection | Refuse operation; alert war-room |
| Time-fence violation | Refuse operation; alert war-room |
| Distress detected | Silent alert; kit continues normally to voter |
| Kit revoked centrally | Kit halts; PO uses backup kit + paper |
| GNSS jamming | IMU-based dead-reckoning; if IMU disagrees with GNSS "off-fence", stay operational and alert |

## 7. Standards

Same as M01 plus GNSS multi-constellation.

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M05-01 | End-to-end voter identification ≤ 3 s fingerprint | Bench |
| A-M05-02 | 700 authentications on battery + power bank | Field trial |
| A-M05-03 | Wrong-station voter correctly redirected | UAT |
| A-M05-04 | Emergency close requires dual control | Policy test |
| A-M05-05 | Distress finger triggers silent alert | Red team |
| A-M05-06 | Kit off-fence blocks operation; back-end alert | Field trial |
| A-M05-07 | Anti-jam mode maintains fence integrity | Signal test |
| A-M05-08 | Legacy Android compatibility with feature degradation | Bench |
| A-M05-09 | Kit revocation ≤ 60s from central | Chaos exercise |
| A-M05-10 | Party agent read-only view has no PII exposure | Manual test |

## 9. Cross-References

- `M06-bvi-backend.md`
- `../SECURITY.md` §3.12
- `../THREAT_MODEL.md` §4, §5
