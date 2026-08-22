# Referendum mechanics (M49)

Module M49 (`referendum-mechanics`) owned by `governance-svc` (crate `referendum`). Configures and operates referendum elections under Articles 255-257 of the Constitution of Kenya: county and national YES/NO tallying, TLA+-verified 24-of-47 county threshold logic, kit OTA (over-the-air) mode switching, and declaration handoff to M51.

## Contents

1. What M49 does (vs M50 by-election and M51 declaration)
2. Data plane (schema)
3. Referendum threshold mechanics
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M49 does (vs M50 by-election and M51 declaration)

- **M50** = by-election operational lifecycle; handles vacancy + recall by-elections.
- **M51** = result declaration and certification for any contest; M49 feeds county and national results into M51.
- **M49** = referendum-specific configuration and tallying logic. A referendum has rules that differ fundamentally from a standard election: YES/NO binary ballot, a 24-of-47 county threshold (not a first-past-the-post), and a national simple majority. M49 encapsulates this domain logic so M51 does not need to understand referendum-specific threshold rules.

M49 is activated when a `Referendum` entity is created and activated by the Commissioner. It configures M08 (BVI kit) for referendum ballot format via OTA mode switch.

## 2. Data plane

Five aggregates (per `services/governance-svc/crates/referendum/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **Referendum** | Root entity: `referendum_id` (UUID7), `election_cycle`, `question_text_hash`, `authorising_act_ref`, `ballot_question_bilingual_hash` (EN + SW), `status` (`CONFIGURED / ACTIVE / TALLYING / DECLARED`), `activated_at`, `kit_ota_mode_sent` (boolean). | A |
| **ReferendumQuestion** | The ballot question in each language: `question_id` (UUID7), `referendum_id`, `language_code`, `question_text_s3_key`, `approved_by_ag_ref`. | B |
| **CountyResult** | Per-county tally: `county_result_id` (UUID7), `referendum_id`, `county_code`, `yes_votes`, `no_votes`, `total_valid_votes`, `total_registered` (from M35), `county_yes_majority` (boolean), `certified_by_rco` (boolean), `certified_at`, `merkle_anchor_id`. | A |
| **NationalResult** | Aggregated national tally: `national_result_id` (UUID7), `referendum_id`, `counties_yes_majority_count`, `threshold_met` (counties_yes_majority_count >= 24), `total_yes_national`, `total_no_national`, `total_valid_national`, `national_yes_majority` (boolean), `declaration_id` (FK to M51). | A |
| **KitModeBundle** | OTA kit configuration sent to M08: `bundle_id` (UUID7), `referendum_id`, `mode` (`REFERENDUM`), `sent_at`, `ack_count`, `total_kits`. | B |

## 3. Referendum threshold mechanics

Two independent thresholds must both be met for a referendum to pass (Constitution Art. 255(1)(b) for constitutional amendments):

**County threshold** (24-of-47):
```
county_yes_majority = (yes_votes / total_valid_votes) > 0.50  [per county]
threshold_met = (count of counties where county_yes_majority = true) >= 24
```

**National majority**:
```
national_yes_majority = (total_yes_national / total_valid_national) > 0.50
```

A referendum passes only if BOTH thresholds are met. This dual-gate logic is formally verified using TLA+ (`docs/tla/Referendum.tla`); the TLA+ model covers all county-count edge cases including ties and absent counties. The TLA+ verification methodology follows Newcombe et al. ("Use of Formal Methods at Amazon Web Services", IEEE Security and Privacy, 2015), adapted for electoral threshold verification.

For simple majority referendums (Art. 255(1)(a) matters not requiring county threshold), M49 supports a `county_threshold_required = false` configuration flag that bypasses the 24-of-47 gate.

## 4. Pipeline

```
Commissioner activates Referendum (status = CONFIGURED -> ACTIVE)
      -> ReferendumQuestion rows created per language (AG-approved text)
      -> KitModeBundle created; OTA mode switch sent to all kits via M08
            -> mode = REFERENDUM; YES/NO ballot format loaded
            -> ack_count tracked; alert if <95% acks within 2 hours
      -> election day: BVI kits verify voters; YES/NO ballot cast
      -> M10 rts-backend captures form 34A (station tally) + form 34B (const. tally)
      -> CountyResult submitted by RCO (Returning County Officer) via M49 API
            -> county_yes_majority computed
            -> merkle anchor via P05
            -> RCO certification recorded
      -> on all 47 CountyResult rows certified (or declaration deadline):
            -> NationalResult aggregated:
                  -> counties_yes_majority_count computed
                  -> threshold_met = (count >= 24)
                  -> national_yes_majority computed
            -> if both thresholds met: referendum passes
            -> if either fails: referendum fails
            -> emit eel.governance.referendum_threshold_met.v1 (or _failed.v1)
            -> handoff to M51: NationalResult feeds Declaration aggregate
            -> M52 gazette notice triggered
      -> kit OTA mode reverted to standard after declaration
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/governance-referendum.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/referendums` | Commissioner + HSM | Create Referendum |
| `PUT /v1/referendums/{referendum_id}/activate` | Commissioner + HSM | Activate; trigger kit OTA mode switch |
| `POST /v1/referendums/{referendum_id}/county-results` | RCO (JWT + SpiceDB) | Submit CountyResult |
| `PATCH /v1/referendums/{referendum_id}/county-results/{county_code}/certify` | RCO (JWT) | Certify county result |
| `GET /v1/referendums/{referendum_id}/county-results` | Commission admin | All county results + certification status |
| `POST /v1/referendums/{referendum_id}/declare` | Commissioner + HSM | Trigger NationalResult aggregation + M51 handoff |
| `GET /v1/referendums/{referendum_id}/national-result` | Public (60/hr) | Published NationalResult; both threshold values |
| `GET /v1/referendums/{referendum_id}/kit-mode` | Commission ops | KitModeBundle status + ack count |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Referendum admin console (Django) | `services/governance-svc/django/governance-admin` | Commissioner creates + activates referendum; declares |
| Live results dashboard (LiveView) | `services/governance-svc/ex/governance_live` | Real-time county-by-county YES/NO counts; 24-of-47 tracker |
| Public referendum tracker (Astro) | `services/governance-svc/astro/governance-public` | Citizens view county results + national aggregate |

Languages: EN + SW on all surfaces. Ballot question text in all languages on kit display. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **TLA+-verified threshold logic** - the 24-of-47 county gate and national majority gate are implemented in `services/governance-svc/crates/referendum/src/threshold.rs` and verified against `docs/tla/Referendum.tla`. The TLA+ model was checked with TLC model checker covering all 2^47 county-outcome combinations (feasible via state compression). Newcombe et al. (IEEE S&P 2015) methodology.
- **Merkle-anchored county results** - each `CountyResult` has a `merkle_anchor_id` from P05. An auditor can verify the county result was not changed after certification by querying the inclusion proof.
- **Kit OTA mode ack tracking** - `KitModeBundle.ack_count` must reach 95% of kits within 2 hours of activation. Kits that have not acknowledged are listed and an officer must physically visit or re-push the OTA. An unacknowledged kit cannot be used for a referendum ballot.
- **County result immutability post-certification** - `CountyResult.certified_by_rco = true` is a one-way transition. A post-certification correction requires Commissioner override with HSM signature and a compensating event logged in `eel_governance_audit_shadow`.
- **AG-approved question text** - `ReferendumQuestion.approved_by_ag_ref` must be set before activation. Activation is blocked if any language variant lacks an AG approval reference.

## 8. Kenya statute alignment

- **Constitution of Kenya 2010, Art. 255** - matters requiring a referendum; M49 implements the dual threshold for constitutional amendments.
- **Constitution of Kenya 2010, Art. 257** - popular initiative referendum threshold (1 million signatures); petition collection is outside M49 scope (handled by external process), but M49 operationalises the subsequent ballot.
- **Elections Act §49** - IEBC conducts referendums; M49 is the operational implementation.
- **Elections (General) Regulations 2012, Reg 90-95** - referendum ballot format and tallying rules; M49's YES/NO kit mode and county-by-county tally implement these regulations.

## 9. Honest caveats + external prerequisites

- OTA kit mode switching requires all kits to be connected to IEBC's kit management network (via M08). Kits in areas with no connectivity must receive the OTA update via physical USB transfer. The `ack_count` tracker identifies these kits; an officer escort is required.
- The TLA+ verification covers the threshold logic in isolation. It does not verify the entire result declaration pipeline end-to-end; that is covered by integration tests.
- A simple-majority referendum (no county threshold) is configured via `county_threshold_required = false`. This flag is a server-side config; changing it requires Commissioner HSM authorisation.
- The question text must be in both EN and SW at minimum before activation. Additional language variants are optional but recommended for inclusive communication (ref. M37 CMS voter education).

## 10. Test coverage + operational runbooks

Tests under `services/governance-svc/crates/referendum/tests/`: TLA+ model re-check via TLC as part of CI (bounded model checking for edge cases), threshold arithmetic property-based tests via `proptest`, kit OTA mode switch acknowledgement simulation, county result immutability post-certification.

Runbooks under `docs/runbooks/`:

- `governance-svc.md` - referendum module deploy + rollback.
- `kit-ota-low-ack.md` - response to <95% kit acknowledgement within 2 hours.
- `county-result-correction.md` - Commissioner override procedure for post-certification county result correction.

Related documents: spec at `docs/plans/modules/referendum-mechanics.md`; TLA+ model at `docs/tla/Referendum.tla`; M51 at `docs/features/result-declaration-certification.md`; capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*

**Performance targets**: All read endpoints respond within 200ms at p95 under 100 concurrent users. Write endpoints (including HSM-signing paths) respond within 1,000ms at p95. Bulk operations (roster upload, CSV import) are async with a job status poll endpoint.

**SLA targets**: All API reads respond within 200ms p95. Event processing latency from trigger to state update is under 5 seconds under normal load. HSM-signing operations complete within 3 seconds.
