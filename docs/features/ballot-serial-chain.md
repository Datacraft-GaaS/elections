# Ballot serial chain (M63)

Module M63 (`ballot-serial-chain`) owned by `rts-svc`. Event-sourced chain of custody for ballot paper serial numbers: assignment from the printer to constituencies, issuance to polling stations, spoilt-ballot recording, and post-poll reconciliation. Result-hold flags block RTS result acceptance when reconciliation is incomplete.

## Contents

1. What Ballot Serial Chain does (vs M39 Ballot Printing Chain)
2. Data plane (schema)
3. Event sourcing model
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Ballot Serial Chain does (vs M39 Ballot Printing Chain)

- **M39** (Ballot Printing Chain) = manages the print-run authorisation, serial range gazettal, and physical dispatch from the printer to constituency stores. M39 owns the print record.
- **M63** = manages what happens after the ballots arrive at constituency stores: serial assignment to polling stations, issuance to presiding officers, spoilt-ballot tracking during voting, and reconciliation at close of poll.

M63 is event-sourced (ADR-0013); every custody change is an append-only event. Result holds (`result_hold` flag on the `SerialAssignment` aggregate) prevent M10 from accepting results for a station with unreconciled ballots.

## 2. Data plane

Four aggregates (per `services/rts-svc/crates/ballot-serial/src/models.rs` and the M63 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **SerialAssignment** | Assignment of a serial range to a polling station: `assignment_id`, `constituency_code`, `station_code`, `serial_range_start`, `serial_range_end`, `assigned_at`, `assigned_by_officer`, `result_hold` (bool, true when reconciliation is incomplete). | A |
| **IssuanceReceipt** | Presiding officer acknowledges receipt of ballots: `receipt_id`, `assignment_id`, `station_code`, `issued_count`, `issued_at`, `presiding_officer_id`, `biometric_verified` (bool). | A |
| **SpoiltBallotEvent** | Spoilt ballot recorded during voting: `spoilt_id`, `assignment_id`, `station_code`, `serial_number`, `reason` (TORN / MARKED / PRINTER_ERROR / VOTER_SPOILT), `spoilt_at`, `recorded_by_officer`. | A |
| **Reconciliation** | Post-poll balance: `rec_id`, `assignment_id`, `issued_count`, `votes_cast`, `spoilt_count`, `unused_returned_count`, `balance_check` (`issued == votes_cast + spoilt + unused`), `reconciled_at`, `presiding_officer_id`, `result_hold_cleared_at`. | A |

## 3. Event sourcing model

M63 is one of the event-sourced modules defined in ADR-0013. The event stream for each `SerialAssignment` is:

```
eel.rts.serial_assigned.v1
eel.rts.issuance_receipt_recorded.v1
eel.rts.spoilt_ballot_recorded.v1  (0..N)
eel.rts.reconciliation_submitted.v1
eel.rts.result_hold_cleared.v1     (emitted only when balance_check passes)
```

The current state of a `SerialAssignment` (including `result_hold`) is derived by replaying its event stream. The current-state projection is materialised in a Postgres read model for query performance.

## 4. Pipeline

Files on disk under `services/rts-svc/crates/ballot-serial/src`: `lib.rs`, `models.rs`, `assignment.rs`, `issuance.rs`, `spoilt.rs`, `reconciliation.rs`, `result_hold.rs`, `routes/`.

```
M39 emits eel.inventory.serial_range_dispatched.v1 to constituency
      -> M63 creates SerialAssignment with result_hold = true
      -> emit eel.rts.serial_assigned.v1
      -> presiding officer (M08 BVI kit) acknowledges receipt
            -> create IssuanceReceipt (biometric_verified gate)
            -> emit eel.rts.issuance_receipt_recorded.v1
      -> during voting: spoilt ballots recorded
            -> create SpoiltBallotEvent per serial number
            -> emit eel.rts.spoilt_ballot_recorded.v1
      -> at close of poll: presiding officer submits Reconciliation
            -> balance_check: issued == votes_cast + spoilt + unused
            -> if balance_check passes:
                  -> result_hold = false
                  -> emit eel.rts.result_hold_cleared.v1
                  -> M10 RTS backend receives cleared event; unlocks result for station
            -> if balance_check fails:
                  -> result_hold remains true
                  -> emit eel.rts.reconciliation_failed.v1
                  -> Returning Officer notified; manual adjudication
      -> M10 will not accept Form 34A for a station with result_hold = true
```

## 5. API surface

Contract: `packages/openapi-contracts/rts/ballot-serial.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/ballot-serial/assignments` | Constituency coordinator (JWT + SpiceDB `CONSTITUENCY_COORDINATOR`) | Create serial assignment for station |
| `POST /v1/ballot-serial/assignments/{id}/issuance` | Presiding officer (BVI kit JWT) | Record issuance receipt |
| `POST /v1/ballot-serial/assignments/{id}/spoilt` | Presiding officer | Record spoilt ballot |
| `POST /v1/ballot-serial/assignments/{id}/reconciliation` | Presiding officer | Submit post-poll reconciliation |
| `GET /v1/ballot-serial/assignments/{id}` | RO, RTS system | Current assignment state + result_hold |
| `GET /v1/ballot-serial/assignments` | RO, constituency coordinator | List assignments by constituency + result_hold status |
| `GET /v1/ballot-serial/assignments/{id}/events` | Legal officer, OAG auditor | Full event stream for one assignment |
| `PATCH /v1/ballot-serial/assignments/{id}/override-hold` | Returning Officer + Commissioner countersign | Manual result hold override (exceptional, logged) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Constituency coordinator console (Django) | `services/rts-svc/django/ballot_serial_admin` | Create assignments, monitor result holds |
| Presiding officer app (BVI kit) | `services/biometric-edge-svc/` | Acknowledge receipt, record spoilt ballots, submit reconciliation |
| Returning Officer dashboard (LiveView) | `services/rts-svc/ex/ballot_serial_web` | Real-time reconciliation status; result hold overview |

Languages: EN + SW (presiding officer app). WCAG 2.2 AA (web surfaces).

## 7. Integrity + anti-tamper

- **Event append-only**: the `ballot_serial_events` table has no application `UPDATE` or `DELETE` grants; all state changes are new event inserts. The materialised read model is rebuilt from events.
- **Result hold as a hard gate**: M10's Form 34A ingestion endpoint (`POST /v1/rts/batches/form34a`) calls M63 gRPC `CheckResultHold(station_code)` before accepting results. A `result_hold = true` response causes M10 to reject with HTTP 422 and a `RESULT_HOLD_ACTIVE` error code.
- **Biometric gate at issuance**: `IssuanceReceipt.biometric_verified` must be true; biometric verification is performed by the BVI kit (M05) against ABIS (M03). A receipt with `biometric_verified = false` is rejected by the API.
- **Balance check arithmetic**: `Reconciliation.balance_check` is computed server-side; the presiding officer submits the four counts, and the server performs the addition. Client-submitted balance flags are ignored.
- **Override audit**: the `PATCH /override-hold` endpoint requires both Returning Officer JWT and Commissioner HSM countersignature; the override event is written to the event stream and flagged in `eel_rts_audit_shadow`.

## 8. Kenya statute alignment

- **Elections Act §39 (ballot paper security)**: serial number chain custody from printer to station to reconciliation is the paper audit trail required by §39.
- **Elections Act §74 (result transmission)**: the result-hold gate enforces that no results are transmitted for a station with unreconciled ballots, satisfying the statutory requirement for a complete count before declaration.
- **Elections (Technology) Regulations 2017 Regulation 7 (audit trail)**: the event stream per `SerialAssignment` constitutes the statutory audit trail for ballot paper movement.
- **Evidence Act Cap 80 §78 (electronic records)**: the event stream with timestamps and officer IDs is admissible as evidence of custody at each stage.

## 9. Honest caveats + external prerequisites

- M63 relies on M39 (Ballot Printing Chain) emitting `eel.inventory.serial_range_dispatched.v1` with accurate serial ranges. If M39 emits an incorrect range, M63 will create a `SerialAssignment` with wrong bounds; this would be detected at reconciliation but not before.
- The biometric issuance gate requires ABIS (M03) availability at the BVI kit. If ABIS is unavailable (network outage), the kit falls back to PIN-based verification; the `IssuanceReceipt` records `biometric_verified = false` and is flagged for manual review by the Returning Officer.
- Manual override of a result hold (`PATCH /override-hold`) is an exceptional path designed for extreme situations (e.g., catastrophic spoilt-ballot event at a station). Overrides are individually logged and reported to the IAB.
- Spoilt ballot recording is per-serial-number; if a presiding officer records a spoilt ballot against an incorrect serial number, the reconciliation balance check may still pass (if the count is correct), creating a silent error in the per-serial audit trail.

## 10. Test coverage + operational runbooks

Coverage under `services/rts-svc/crates/ballot-serial/tests/`: event stream integrity (property-based, event replay must produce same state), result hold gate (integration: M10 Form 34A rejection with active hold), balance check arithmetic (property-based, fuzz four counts), biometric gate (integration, BVI kit mock), override audit log (integration, override event must appear in audit shadow).

Runbooks under `docs/runbooks/`:

- `rts-svc.md` (section: ballot-serial-chain) -- deploy, rollback, event stream repair.
- `result-hold-manual-override.md` -- procedure for authorised override of a stuck result hold.
- `reconciliation-failure-adjudication.md` -- Returning Officer steps when balance check fails.

Related documents: full spec at `docs/plans/modules/ballot-serial-chain.md`; service capability catalogue at `services/rts-svc/docs/CAPABILITIES.md`; M39 at `docs/features/ballot-printing-chain.md`; M10 at `docs/features/rts-backend.md`.
