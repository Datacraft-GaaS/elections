# Intent preservation (M27)

Module M27 (`intent-preservation`) owned by `support-admin-svc`. Guards voter intent during ballot scanning by validating pre-mark annotations before image deletion, enforcing a 1-second maximum delete window, classifying ambiguous marks through a human-in-the-loop AI pipeline using the A1-A8 / V1-V3 reason-code catalogue, and flagging z-score outlier stations where rejection rates deviate more than 3.0 sigma from the constituency mean.

## Contents

1. What intent preservation does (vs related modules)
2. Data plane (schema)
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What intent preservation does (vs related modules)

- **M09 Results tabulation** = aggregates validated vote counts; M27 validates the mark classification before M09 receives the tally.
- **M27** = post-scan, pre-tally intent gate: validates that the pre-mark annotation process completed before the ballot image was deleted, classifies ambiguous marks against the statutory reason-code catalogue, and flags stations with statistically anomalous rejection rates.
- **M26 Anomaly detection** = broad telemetry and result distribution analysis; M27 feeds its per-station rejection rate to M26 Layer 2 as a station anomaly signal.

M27 emits `PreMarkValidationEvent` for every ballot. No ballot image is deleted before M27 emits a validation event.

## 2. Data plane

Five aggregates under `services/support-admin-svc/crates/intent-api/src/models.rs`:

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **PreMarkValidationEvent** | Per-ballot validation: ballot_id, station_id, image_hash, annotation_complete (bool), image_delete_elapsed_ms, validation_status (VALID/AMBIGUOUS/REJECTED), ts_hlc. | A |
| **AmbiguousMark** | Ambiguous mark requiring human review: ballot_id, image_hash, candidate_marks (JSON), reason_code (A1-A8 or V1-V3), reviewer_id, decision, sig. | B |
| **ReasonCode** | Statutory reason-code definition: code, category (ACCEPT=A / VOID=V), description, statutory_ref. | D |
| **RejectionRateAnomaly** | Z-score outlier: station_id, constituency_id, rejection_rate, z_score, sigma_threshold (3.0), ts_hlc. | B |
| **AuditEvent** | Every officer action on ambiguous marks: actor, action, ballot_id, ts_hlc, sig (append-only). | A |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| Ballot scanner firmware event | Per ballot | Kafka | Device cert (scanner SVID) | Validation event required before image delete; scanner holds if event not received within 1 s |
| Pre-mark annotation (officer) | Per ballot pre-scan | Kafka | Device cert | Missing annotation = validation_status AMBIGUOUS; image delete blocked |
| M09 tally ingestion | Per station close | Kafka | SVID mTLS | M09 waits for PreMarkValidationEvent for every ballot_id before accepting tally |
| M26 anomaly feed | Hourly | Kafka | Internal | M27 publishes rejection rate; M26 consumes as Layer 2 signal |

## 4. Pipeline

```
ballot scanned at station (scanner firmware)
  -> PreMarkValidationEvent created with image_hash
  -> annotation_complete checked:
       false: image_delete blocked; officer prompted to annotate
       true: image_delete_elapsed_ms timer starts
  -> image_delete_elapsed_ms checked:
       > 1000 ms: PreMarkValidationEvent(status=REJECTED, reason=ELAPSED)
                  ballot flagged; presiding officer notified; M26 L2 signal
       <= 1000 ms: proceed
  -> mark classifier (M28 A4 vision model, confidence threshold 0.90):
       confidence >= 0.90: reason_code assigned (A1-A8 / V1-V3)
       confidence < 0.90: AmbiguousMark created; human reviewer queue

A1-A8 accept codes:
  A1 Clear mark in candidate box
  A2 Mark partially outside box but candidate intent clear
  A3 Two marks, one clearly cancelled
  A4 Stamp rather than pen mark
  A5 Mark in party column, candidate unambiguous
  A6 Tick, cross, or X in candidate box
  A7 Initials only but candidate unambiguous
  A8 Mark smeared but candidate unambiguous

V1-V3 void codes:
  V1 Two or more candidates marked, no cancellation
  V2 No mark in any candidate area
  V3 Candidate identification impossible

  -> human reviewer makes decision on AmbiguousMark; sig captured
  -> final reason_code stored; PreMarkValidationEvent updated

every 30 min per station:
  rejection_rate = rejected / total scanned
  constituency_mean and std computed from peer stations
  z_score = (rejection_rate - mean) / std
  if |z_score| > 3.0:
    RejectionRateAnomaly created
    eel.intent.rejection_outlier.v1 emitted
    M26 L2 signal created
```

## 5. API surface

Contract: `packages/openapi-contracts/support-admin/intent.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /intent/v1/validations` | Scanner device (cert) | Submit PreMarkValidationEvent |
| `GET /intent/v1/validations/{ballot_id}` | M09 / IAB | Validation result for a ballot |
| `GET /intent/v1/ambiguous` | Officer (JWT + SpiceDB INTENT_REVIEWER) | Ambiguous mark review queue |
| `PATCH /intent/v1/ambiguous/{id}` | Officer | Submit human decision on ambiguous mark |
| `GET /intent/v1/anomalies` | M26 / IAB (JWT) | Rejection rate anomaly feed |
| `GET /intent/v1/reason-codes` | Public (CDN-cached) | Statutory reason-code catalogue |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Ambiguous mark review (Phoenix LiveView) | `services/support-admin-svc/ex/intent_web/` | Officer sees ballot image, pre-mark annotation, and AI classification; confirms or overrides reason code |
| Station anomaly dashboard (Grafana embed) | `platform/observability/support-admin-svc/` | Z-score per station; outliers highlighted; feeds M26 console |
| Pre-mark annotation prompt (M31 companion) | `services/field-mobile-svc/flutter/poll-clerk/` | Presiding officer completes annotation before scan |
| Public reason-code reference (Astro) | `services/support-admin-svc/astro/intent-public/` | Published statutory reason-code catalogue |

WCAG 2.2 AA on all web surfaces. Languages: EN + SW.

## 7. Integrity + anti-tamper

- **1-second delete window** -- the scanner firmware holds the image delete signal until the `PreMarkValidationEvent` is received via a local Kafka consumer with a 1 000 ms timeout; if the event is not received within 1 s, the firmware cancels the delete and logs a `DELETE_HOLD` event; the image is retained for manual review.
- **Image hash in validation event** -- the `image_hash` (SHA-256) in the `PreMarkValidationEvent` is compared against the hash provided by M09's scanner-hash feed; any mismatch is a P0 alert.
- **Human-in-the-loop for ambiguous marks** -- the A4 vision model never makes a final decision; it populates a suggested reason_code; a human officer signature is always required before the reason_code is finalised.
- **Append-only audit events** -- no officer can delete or modify an `AuditEvent`; every decision is permanently linked to the officer's JWT identity and device cert.
- **Z-score outlier P05 anchor** -- every `RejectionRateAnomaly` is anchored in the P05 transparency log within 60 s; the anomaly cannot be suppressed post-hoc.

## 8. Kenya statute alignment

- **Elections Act s.72 (validity of votes)** -- the A1-A8 / V1-V3 reason-code catalogue implements the statutory ballot validity criteria; every void decision must cite a V1-V3 code with a statutory_ref.
- **Elections (General) Regulations 2012 r.63** -- officer must annotate ambiguous marks before ballot is tallied; the annotation_complete flag enforces this.
- **DPA 2019 s.26 (minimisation)** -- ballot images are processed for intent validation and then deleted; no long-term storage of individual ballot images; image_hash retained for audit.
- **Elections (Technology) Regulations 2017** -- scanner firmware event log and PreMarkValidationEvent feed retained for 7 years as part of the election audit trail.

## 9. Honest caveats + external prerequisites

- The A4 vision model confidence threshold (0.90) is a design parameter; calibration requires a representative training set of Kenyan ballot mark types, which must be assembled from historical election samples.
- The 1-second delete window assumes sub-second network latency between the scanner and the Kafka broker; in high-latency or offline scenarios the firmware DELETE_HOLD event fires and the image is retained for manual review; this increases the manual review queue size.
- Z-score outlier detection assumes at least 10 peer stations in the same constituency are operational and reporting; constituencies with fewer active stations produce unreliable z-scores and the anomaly flag is suppressed.
- The human reviewer queue can become a bottleneck during high-volume polling periods; the SLA for ambiguous mark review is 30 minutes from queue entry; breach triggers a presiding officer escalation.

## 10. Test coverage + operational runbooks

Coverage under `services/support-admin-svc/crates/intent-api/tests/`: unit tests for delete-window enforcer, reason-code classifier, z-score outlier detector; property-based tests asserting validation_status VALID only when annotation_complete is true and elapsed_ms <= 1000; contract tests (Pact) for scanner firmware event, M09 ballot hash feed, M26 anomaly signal; integration tests with scanner stub, SoftHSM reviewer signing, P05 tlog stub; e2e scenario covering scan, annotation, 1-second window enforcement, ambiguous mark review, and rejection-rate anomaly generation; load tests (k6) for 10 000 ballots per hour per station.

Runbooks under `docs/runbooks/`:

- `intent-preservation-degradation.md` (create): Kafka consumer lag; scanner DELETE_HOLD spike; ambiguous mark queue backlog; z-score suppression.

Dashboards: `platform/observability/support-admin-svc/grafana-intent.json` (validation rate, ambiguous queue depth, rejection rate per station, z-score outlier count). SLO: 99.9% validation event before delete; ambiguous review SLA breach rate <1%; z-score outlier anchor lag <90 s.

Related: `docs/plans/modules/intent-preservation.md`; `services/support-admin-svc/docs/CAPABILITIES.md`; edge-case matrix EC-INT-* in `docs/plans/08-edge-case-matrix.md`.
