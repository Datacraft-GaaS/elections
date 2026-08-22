# Result declaration and certification (M51)

Module M51 (`result-declaration-certification`) owned by `rts-svc` (crate `declaration`). Event-sourced declaration lifecycle producing HSM-countersigned `CertificateOfElection` per contest. Handles run-off triggers (presidential 50%+1 rule), tie state (`TIE_PENDING`), corrections via compensating events, petition window tracking with judiciary acknowledgement hooks, and gazette handover to M52.

## Contents

1. What M51 does (vs M10 rts-backend and M52 gazette)
2. Data plane (schema)
3. Declaration state machine
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M51 does (vs M10 rts-backend and M52 gazette)

- **M10** = rts-backend; captures form 34A (station) and form 34B (constituency) tallies in real time. It publishes `eel.rts.tally_published.v1` which unblocks M51.
- **M51** = takes the published tally and executes the formal declaration: determines the winner (or triggers run-off/tie), issues the HSM-countersigned certificate, tracks the petition window, and hands the gazette notice obligation to M52.
- **M52** = election calendar and gazette; consumes `eel.declaration.result_declared.v1` to publish the gazette notice of the election result.

M51 is the only event-sourced module among M36-M52. Every state change is a committed event; corrections are compensating events, not overwritten state. This is the strongest integrity guarantee in the electoral system.

## 2. Data plane

Four aggregates (per `services/rts-svc/crates/declaration/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **Declaration** | Event-sourced root: `declaration_id` (UUID7), `contest_id`, `election_cycle`, `status` (`TALLYING / DECLARED / AMENDED / RUNOFF_PENDING / TIE_PENDING / CERTIFIED`), `declared_at`, `winner_candidate_id_hash` (null if TIE_PENDING or RUNOFF_PENDING), `vote_total`, `winning_margin`, `petition_window_expires_at`. | A |
| **CertificateOfElection** | Issued after no petition or petition dismissed: `certificate_id` (UUID7), `declaration_id`, `candidate_id_hash`, `contest_type`, `constituency_code`, `issued_at`, `hsm_signature`, `gazette_notice_id` (FK to M52). | A |
| **PetitionWindowTracker** | Tracks the 28-day petition window: `tracker_id` (UUID7), `declaration_id`, `window_opens_at`, `window_expires_at`, `petition_filed` (boolean), `petition_ref`, `judiciary_acknowledged_at`, `status` (`OPEN / PETITION_FILED / DISMISSED / UPHELD / EXPIRED`). | A |
| **DeclarationAmendment** | Compensating event for post-declaration corrections: `amendment_id` (UUID7), `declaration_id`, `amended_by` (Commissioner), `reason_hash`, `before_state_sha256`, `after_state_sha256`, `amendment_at`, `hsm_signature`. | A |

## 3. Declaration state machine

```
TALLYING -> DECLARED (tally published, winner clear)
         -> RUNOFF_PENDING (presidential: winner < 50%+1 of valid votes)
         -> TIE_PENDING (exact tie; Commissioner intervention required)

DECLARED -> CERTIFIED (petition window expires with no petition)
          -> AMENDED (compensating event applied; re-declares)

RUNOFF_PENDING -> DECLARED (run-off result received)

TIE_PENDING -> (Commissioner resolves; implementation-defined; typically lot or court order)
```

Presidential run-off condition (Art. 138(4)):
```
runoff_required = (winner_votes / total_valid_votes) <= 0.50
                  OR (winner_votes - runner_up_votes) < (total_valid_votes * 0.25)
```

Tie condition:
```
tie = (top_candidate_votes == second_candidate_votes)
```

Both conditions are implemented with 128-bit integer arithmetic (Goldberg, ACM Computing Surveys, 1991) to avoid floating-point rounding.

## 4. Pipeline

```
eel.rts.tally_published.v1 consumed (hard prerequisite; declaration blocked until received)
      -> Declaration created (status = TALLYING)
      -> winner determination logic:
            -> if presidential: apply 50%+1 + 25% margin check
            -> if tie: status -> TIE_PENDING; Commissioner alerted
            -> if runoff needed: status -> RUNOFF_PENDING; run-off contest created
            -> else: status -> DECLARED; winner_candidate_id_hash set
      -> PetitionWindowTracker created:
            -> window_opens_at = declared_at
            -> window_expires_at = declared_at + 28 days
            -> eel.declaration.petition_window_opened.v1 emitted
      -> emit eel.declaration.result_declared.v1
      -> M52 consumes event: gazette notice created
      -> M40 court bundle assembly triggered
      -> on petition_filed = true (court webhook or Commission legal input):
            -> PetitionWindowTracker.petition_filed = true
            -> PetitionWindowTracker.petition_ref set
            -> eel.declaration.petition_filed.v1 emitted
            -> CertificateOfElection issuance HELD
      -> on petition window expiry with no petition:
            -> PetitionWindowTracker.status -> EXPIRED
            -> CertificateOfElection issued:
                  -> Commissioner + HSM countersignature
                  -> gazette_notice_id FK to M52 gazette
                  -> eel.declaration.certificate_issued.v1
      -> on amendment (post-declaration correction):
            -> DeclarationAmendment created (compensating event)
            -> before_state_sha256 + after_state_sha256 computed
            -> Commissioner HSM-signs amendment
            -> Declaration re-emits eel.declaration.result_declared.v1 (amended=true)
```

## 5. API surface

Contract: `packages/openapi-contracts/rts/result-declaration-certification.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/declarations` | System (on tally published event) | Create Declaration record |
| `PUT /v1/declarations/{declaration_id}/declare` | Returning Officer + HSM | Formal declaration with winner |
| `PUT /v1/declarations/{declaration_id}/runoff` | Commissioner + HSM | Trigger run-off contest |
| `POST /v1/declarations/{declaration_id}/amend` | Commissioner + HSM | Apply DeclarationAmendment (compensating event) |
| `GET /v1/declarations/{declaration_id}` | Auth user | Declaration status + petition window |
| `GET /v1/declarations/{declaration_id}/certificate` | Returning Officer (JWT) or court (SVID) | Retrieve CertificateOfElection |
| `PUT /v1/declarations/{declaration_id}/judiciary-ack` | Commission legal | Record judiciary acknowledgement of petition |
| `GET /v1/declarations/public/{contest_id}` | Public (60/hr) | Declared result + certificate status; no candidate PII |
| `GET /v1/declarations/{declaration_id}/amendment-log` | Court registrar | Full DeclarationAmendment history |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Declaration admin console (Django) | `services/rts-svc/django/declaration-admin` | Returning Officers declare; Commissioner signs; amendments |
| Commissioner certificate panel (Django) | `services/rts-svc/django/declaration-admin` | HSM-sign certificates; gazette handover |
| Public results portal (Astro, via M11) | `services/portal-svc/astro/gazette` | Public views declared results + certificate status |

Languages: EN + SW on all surfaces. CertificateOfElection document is EN + SW bilingual. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Event sourcing** - M51 is the only fully event-sourced module among M36-M52, following ADR-0013. Every state change is an immutable event; the current state is a projection of all events. A corrupted state can be rebuilt from the event log.
- **HSM countersignature on certificates** - `CertificateOfElection.hsm_signature` is produced by the Commissioner HSM using the per-cycle KEK from `eel_kek_generations`. The certificate is verifiable independently using the public key in the P05 transparency log.
- **Compensating events for amendments** - post-declaration corrections use compensating events (not overwrites). The `DeclarationAmendment` records before and after state hashes. Both the original declaration and the amendment are P05-anchored; there is no way to silently replace the original.
- **128-bit integer arithmetic for thresholds** - run-off and tie conditions use 128-bit integer arithmetic to avoid floating-point edge cases (Goldberg, ACM CS 1991).
- **Petition window immutability** - `PetitionWindowTracker.window_expires_at` is computed at declaration time and cannot be extended without a Commissioner HSM-signed amendment. Any extension is a public event in the audit shadow.
- **Gazette handover confirmation** - M52 must acknowledge `eel.declaration.result_declared.v1` within 30 minutes. If no acknowledgement is received, an alert fires and M52 is manually triggered by ops.

## 8. Kenya statute alignment

- **Constitution of Kenya 2010, Art. 138** - presidential election; 50%+1 + 25% margin rule for run-off. M51's run-off condition directly implements Art. 138(4).
- **Elections Act §86** - High Court petition window; 28 days from declaration date. M51's `PetitionWindowTracker.window_expires_at = declared_at + 28 days`.
- **Elections Act §83** - validity of elections; M51's CertificateOfElection is the statutory instrument.
- **Elections (Technology) Regulations 2017** - electronic declaration records must be signed and auditable; M51's event-sourced + HSM-signed architecture satisfies this.
- **Evidence Act Cap 80** - the event log + HSM signatures + P05 Merkle anchors provide admissible electronic evidence for court proceedings.

## 9. Honest caveats + external prerequisites

- `TIE_PENDING` is a valid terminal state in the state machine, but the resolution procedure (lot-drawing or court order per Elections Act) is not automated. The Commissioner must manually resolve a tie via the amendment path. The system parks in `TIE_PENDING` until the Commissioner acts.
- Petition window tracking relies on judiciary API hooks (eCourt) for petition filing notification. If eCourt is unavailable, Commission legal staff must manually record `petition_filed = true` via the console. The system does not auto-detect court filings.
- Run-off scheduling is not automated; M51 creates `RUNOFF_PENDING` and alerts the Commission, but the run-off election date and logistics are set manually and fed back via a new `Declaration` for the run-off contest.
- Event store growth: for a general election with ~1,700 contests, the declaration event log will have 10,000-50,000 events. This is well within PostgreSQL's capacity; no event archival is needed for the first 3 election cycles.

## 10. Test coverage + operational runbooks

Tests under `services/rts-svc/crates/declaration/tests/`: event sourcing round-trip (write events, rebuild projection, verify state), run-off and tie arithmetic via `proptest`, compensating event application, HSM signature mock verification, petition window expiry simulation.

Runbooks under `docs/runbooks/`:

- `rts-svc.md` - declaration module deploy + rollback.
- `tie-resolution.md` - Commissioner procedure for resolving TIE_PENDING state.
- `declaration-amendment.md` - post-declaration correction procedure (compensating event).
- `petition-window-extension.md` - (exceptional) Commissioner-authorised window extension.

Related documents: spec at `docs/plans/modules/result-declaration-certification.md`; M10 at `docs/features/`; M52 at `docs/features/election-calendar-gazette.md`; capability catalogue at `services/rts-svc/docs/CAPABILITIES.md`.
