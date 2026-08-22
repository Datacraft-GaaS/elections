# Recall (M48)

Module M48 (`recall`) owned by `governance-svc` (crate `recall`). Manages the constitutional recall process under Article 104: petition filing, signatory collection via OCR-scanned signature sheets, Article 104(2) 30%-of-registered-voters threshold verification, Article 104(3) YES-majority threshold in the subsequent recall election, and handoff to M50 for the by-election if the petition threshold is met.

## Contents

1. What M48 does (vs M50 by-election operations)
2. Data plane (schema)
3. Petition and threshold mechanics
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M48 does (vs M50 by-election operations)

- **M50** = full by-election operational lifecycle; it is triggered by M48 when the recall petition threshold is met and the recall election is won by the YES vote.
- **M48** = the preceding process: validating the petition, counting signatories, checking the Art. 104(2) 30% threshold, conducting the recall election (as a specialised sub-contest), and checking the Art. 104(3) YES-majority threshold. Only if M48 determines the recall succeeds does it emit `eel.governance.recall_threshold_met.v1` to trigger M50.

The two constitutional thresholds are distinct: Art. 104(2) governs petition validity (30% of registered voters must sign), and Art. 104(3) governs the recall election outcome (a majority of voters who vote in the recall election must vote YES).

## 2. Data plane

Four aggregates (per `services/governance-svc/crates/recall/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **RecallPetition** | Root entity: `petition_id` (UUID7), `constituency_code`, `target_mp_id_hash`, `filed_at`, `collection_deadline` (filed_at + 30 days per Art. 104(2)), `registered_voters_count` (from M35 snapshot), `threshold_required` (registered_voters_count * 0.30, ceiling), `signatories_verified`, `threshold_met` (boolean), `status` (`COLLECTION / VERIFICATION / THRESHOLD_MET / THRESHOLD_FAILED / RECALL_ELECTION / RECALL_SUCCEEDED / RECALL_FAILED`). | A |
| **SignatoryRecord** | Individual signatory: `signatory_id` (UUID7), `petition_id`, `national_id_hash`, `sheet_id`, `sheet_row`, `ocr_confidence`, `officer_verified` (boolean), `status` (`PENDING / VERIFIED / REJECTED_DUPLICATE / REJECTED_NOT_REGISTERED / REJECTED_LOW_CONFIDENCE`). | A |
| **ThresholdReport** | Threshold computation audit: `report_id` (UUID7), `petition_id`, `snapshot_root_hash` (M35 RegisterSnapshot reference), `total_signatories_submitted`, `total_verified`, `total_rejected`, `threshold_required`, `threshold_met`, `computed_at`, `hsm_signature`. | A |
| **RecallElection** | The recall ballot itself: `election_id` (UUID7), `petition_id`, `election_date`, `yes_votes`, `no_votes`, `total_valid_votes`, `yes_majority_met` (boolean), `declared_at`, `declaration_certificate_id` (FK to M51). | A |

## 3. Petition and threshold mechanics

Art. 104(2) threshold: 30% of registered voters in the constituency must sign within 30 days. The registered voter count is taken from the M35 `RegisterSnapshot` at petition filing date (immutable for this petition).

```
threshold_required = ceiling(registered_voters_count * 0.30)
```

Art. 104(3) threshold: in the recall election, the YES vote must constitute a simple majority of valid votes cast.

```
yes_majority_met = (yes_votes / total_valid_votes) > 0.50
```

Both computations are implemented in `services/governance-svc/crates/recall/src/threshold.rs` with 128-bit integer arithmetic to avoid floating-point rounding (Goldberg, "What Every Computer Scientist Should Know About Floating-Point Arithmetic", ACM Computing Surveys, 1991).

## 4. Pipeline

```
petition filed via portal or in-person (OIDC petitioner or officer)
      -> RecallPetition created (status = COLLECTION)
      -> registered_voters_count fetched from M35 RegisterSnapshot
      -> threshold_required computed + stored
      -> 30-day collection window opens
      -> petition collectors submit signature sheets (scanned PDF upload)
            -> OCR engine (services/governance-svc/py/recall-ocr/) extracts:
                  national_id_hash + signatory name hash per row
            -> SignatoryRecord created per row (status = PENDING)
            -> raw signature images deleted within 1 second of OCR extraction
              (KDPA §26 minimisation; only structured data retained)
      -> officer verification phase (on collection deadline or petition closure):
            -> officer reviews low-confidence OCR rows (ocr_confidence < 0.80)
            -> duplicates rejected (national_id_hash unique per petition)
            -> non-registered signatories rejected (checked against M35)
            -> status updated per SignatoryRecord
      -> ThresholdReport computed:
            -> total_verified counted
            -> threshold_met = (total_verified >= threshold_required)
            -> HSM-signed by Commissioner
      -> if threshold_met:
            -> petition status -> THRESHOLD_MET
            -> emit eel.governance.recall_petition_threshold_met.v1
            -> IEBC gazette notice via M52
            -> RecallElection created; date set per Elections Act
            -> recall election conducted (M48 sub-contest; M10 tally applies)
            -> on declaration: RecallElection updated with yes_votes / no_votes
            -> if yes_majority_met: status -> RECALL_SUCCEEDED
                  -> emit eel.governance.recall_threshold_met.v1
                  -> M50 by-election triggered
            -> if not: status -> RECALL_FAILED
      -> if threshold not met: status -> THRESHOLD_FAILED; petitioner notified
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/governance-recall.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/recall/petitions` | Petitioner (OIDC) | File recall petition |
| `GET /v1/recall/petitions/{petition_id}` | Petitioner (OIDC) or officer | Petition status + threshold progress |
| `POST /v1/recall/petitions/{petition_id}/sheets` | Petition officer | Upload signature sheet scans |
| `GET /v1/recall/petitions/{petition_id}/signatories` | Commission officer | List SignatoryRecords + verification status |
| `PATCH /v1/recall/signatories/{signatory_id}` | Commission officer | Override OCR verdict (officer verification) |
| `POST /v1/recall/petitions/{petition_id}/threshold` | Commissioner + HSM | Compute + sign ThresholdReport |
| `GET /v1/recall/petitions/{petition_id}/threshold` | Public (30/hr) | Published ThresholdReport (post-Commissioner sign) |
| `GET /v1/recall/elections/{election_id}` | Public | Recall election status + declared result |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Petitioner portal (Astro) | `services/governance-svc/astro/governance-public` | File petition; upload sheets; track status |
| Officer verification console (LiveView) | `services/governance-svc/ex/court_web` (shared) | Officer reviews OCR results; overrides; computes threshold |
| OCR extraction service (Python FastAPI) | `services/governance-svc/py/recall-ocr` | Machine endpoint; not a UI |
| Commissioner signing panel (Django) | `services/governance-svc/django/governance-admin` | HSM-sign ThresholdReport + gazette notice |

Languages: EN + SW on portal. Officer surfaces: EN + SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **RegisterSnapshot immutability** - the registered voter count is locked at petition filing from a specific M35 `RegisterSnapshot.root_hash`. Any subsequent register changes (deaths, corrections) do not affect this petition's threshold computation.
- **Raw image deletion within 1 second** - OCR extracts national_id_hash and name hash from the scanned sheet; the raw image is deleted before the HTTP response is returned (within 1 second). The deletion is logged in `eel_governance_audit_shadow`. Only structured data (hashes) is retained.
- **Duplicate detection** - `national_id_hash` has a unique-per-petition index. A second signature from the same national ID is rejected as `REJECTED_DUPLICATE` regardless of OCR confidence.
- **128-bit integer threshold arithmetic** - avoids floating-point precision loss (Goldberg, ACM CS 1991). The threshold is computed server-side; the client receives the result, not the raw inputs.
- **HSM-signed ThresholdReport** - the report is Commissioner-signed before it can be made public. A claim about the threshold outcome without a valid HSM signature on the ThresholdReport is not actionable under the runbook.

## 8. Kenya statute alignment

- **Constitution of Kenya 2010, Art. 104** - recall of Members of Parliament; M48 directly implements both thresholds and procedural requirements.
- **Elections Act §45A** - procedures for recall elections; M48's `RecallElection` aggregate implements the election mechanics.
- **Kenya Data Protection Act §26 (minimisation)** - raw signature images deleted within 1 second; only hashed structured data retained.
- **Kenya Data Protection Act §41** - no fully automated rejection of signatories; low-confidence OCR rows require officer review.
- **Evidence Act Cap 80** - HSM-signed ThresholdReport + P05 Merkle anchor provides admissible electronic evidence of the threshold verification outcome.

## 9. Honest caveats + external prerequisites

- OCR accuracy on handwritten signature sheets depends on scan quality. Sheets submitted as low-resolution JPEG (below 200 DPI) are rejected with a quality-check error; petitioners are advised to scan at 300 DPI or above.
- The OCR engine (`services/governance-svc/py/recall-ocr/`) uses Tesseract 5.x with a Kenyan national ID format fine-tune. Accuracy on printed sheets is higher than on handwritten sheets. The officer verification step is the quality gate for low-confidence rows.
- A constitutional recall petition is expected to be rare (one or two per election cycle at most). The system is designed for correctness over throughput; it is not optimised for bulk petition processing.
- Recall elections require voter register subsets (only the recalled MP's constituency). M50 provides the register subset infrastructure; M48 requests it via `eel.governance.recall_petition_threshold_met.v1`.

## 10. Test coverage + operational runbooks

Tests under `services/governance-svc/crates/recall/tests/`: threshold arithmetic property-based tests via `proptest` (edge cases: exactly 30.000% signatories), OCR pipeline integration with sample PDF fixtures, duplicate signatory rejection, RegisterSnapshot immutability.

Runbooks under `docs/runbooks/`:

- `governance-svc.md` - recall module deploy + rollback.
- `ocr-quality-failure.md` - response to bulk low-confidence OCR on a submitted sheet batch.
- `threshold-dispute.md` - response to a petitioner disputing the ThresholdReport outcome.

Related documents: spec at `docs/plans/modules/recall.md`; M50 at `docs/features/by-election-operations.md`; M35 at `docs/features/continuous-voter-registration.md`; capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`.
