# M27 — Intent Preservation Module

**Module ID:** M27
**Name:** Voter Intent Preservation (Pre-Mark Validation, Adjudication, Reject-Rate Minimisation)
**Runs on:** Kit-side (on-device inference) + Central adjudication console
**Owner:** Datacraft
**Status:** Draft v2
**IPR:** Custom Software vests in IEBC per GCC 15.4; on-device vision model OSS base (Apache 2.0)

## 1. Purpose

Minimise rejected and spoilt ballots through seven complementary elements: optimised ballot design, opt-in kit-side pre-mark visual validation (on-device only, no image retained), spoilt-ballot replacement per Elections Act s.54, human adjudication of ambiguous ballots with a standardised reason-code catalogue, multi-channel voter education, presiding officer training, and ex-post per-station rejection-rate reconciliation with statistical outlier flagging.

Target: rejected ballot rate < 1.0% per contest (down from ~2.5% in 2017).

Strategy reference: §XVII.

## 2. User Personas

- **Voter** — primary beneficiary; receives clearer ballot, optional pre-mark check, education
- **Presiding Officer** — adjudicates ambiguous ballots; administers spoilt-ballot replacement; trained on reason-code catalogue
- **Party Agent** — witnesses ambiguous-ballot adjudication; countersigns reason codes
- **IEBC Returning Officer** — reviews per-station rejection rates; flags outlier stations
- **IAB Adjudication Auditor** — reviews a sample of adjudicated-ballot images post-election
- **IEBC Ballot Design Team** — receives ballot-design test results and iterated specs
- **Voter Education Coordinator** — uses portal tutorials and IVR scripts generated from this module

## 3. Functional Requirements

### Element 1 — Ballot Design

- **FR-M27-001** Produce a scientifically-tested ballot design for each of the six simultaneous contests (presidential, gubernatorial, senate, MP, women-rep, MCA). Each design tested with >= 200 real Kenyan voters across literacy strata and regions before finalisation.
- **FR-M27-002** Design requirements: candidate photograph + party symbol; clear marking box with generous whitespace (>= 15mm per side); simplified instructions in contest-specific language + pictogram guidance; colour-differentiation per contest (presidential yellow, gubernatorial blue, senate green, MP red, women-rep purple, MCA orange).
- **FR-M27-003** Per-ballot instructions at the top of each ballot: language at Kenyan Standard Grade 5 reading level (EN + SW); pictogram "make one mark inside the box". Large-print version available at station on request.
- **FR-M27-004** Ballot design published at T-30 for party-agent and public review before printing. Any design change after T-30 requires IEBC Chair approval + IAB notification.
- **FR-M27-005** Ballot design artefacts (PDF + test-results summary) included in the T-14 evidentiary bundle.

### Element 2 — Kit-Side Pre-Mark Validation (Opt-In, On-Device Only)

- **FR-M27-006** After the voter marks their ballot but before depositing it, the kit displays an opt-in prompt: "Would you like to check your ballot?" (EN/SW/local language). The voter can decline with one tap; no coercion or default to participation.
- **FR-M27-007** If the voter opts in, they hold the ballot face-down to the kit camera. The kit's on-device vision model (INT8 quantised, <= 10 MB) analyses the ballot image in under 3 seconds and returns one of: VALID (mark detected in exactly one box), UNMARKED (no mark detected), MULTIPLE_MARKS (marks in more than one box), AMBIGUOUS_MARK (mark partially inside / partially outside box), WRONG_BALLOT (wrong-colour ballot for current contest).
- **FR-M27-008** The kit displays the result in plain language: "Your ballot looks correctly marked" or "We detected [issue]. Would you like to replace your ballot?" Replacement offered per Elections Act s.54.
- **FR-M27-009** MANDATORY: the ballot image is processed entirely on-device and DELETED immediately after inference. No image, no feature vector, no inference artefact is transmitted or stored. DPA 2019 s.26 lawful basis: voter consent for the check; data minimisation principle satisfied by on-device deletion.
- **FR-M27-010** Kit logs: (a) whether the voter opted in (boolean), (b) result code (VALID / UNMARKED / MULTIPLE_MARKS / AMBIGUOUS_MARK / WRONG_BALLOT), (c) whether the voter proceeded to replacement. No voter identity is logged alongside the pre-mark check result; the event is logged as a station aggregate.
- **FR-M27-011** Opt-in rate published as an anonymised station-level aggregate in the post-election report.
- **FR-M27-012** Presiding Officer can disable the pre-mark validation feature for a specific kit session via a configuration toggle with a reason code and party-agent witness; disabling is logged and included in the M26 Layer 1 telemetry.

### Element 3 — Spoilt-Ballot Replacement

- **FR-M27-013** Elections Act s.54 allows a voter to spoil a ballot and receive a replacement, up to 3 times per contest.
- **FR-M27-014** Kit tracks spoilt-ballot events per voter per contest (session-only; cleared on session close). On the third spoil, kit locks further replacement for that voter per contest and logs the event.
- **FR-M27-015** Spoilt ballot is retained by the Presiding Officer in a sealed spoilt-ballot envelope; envelope sealed with RFID-tagged seal; count of spoilt ballots recorded on Form 34A.
- **FR-M27-016** Post-election: spoilt ballots retained for the 30-day petition window; available for IAB sample audit.

### Element 4 — Ambiguous-Ballot Adjudication

- **FR-M27-017** Any ballot that the Presiding Officer cannot clearly accept or reject on first inspection enters the adjudication flow.
- **FR-M27-018** Adjudication procedure: Presiding Officer + minimum two party agents view the ballot together; each party agent states their position (VALID / REJECT); Presiding Officer makes the determination with a reason code from the standardised catalogue (see below).
- **FR-M27-019** Reason-code catalogue (exhaustive; any new category requires IEBC Chair sign-off):
  - A1 UNMARKED_BALLOT — no mark visible
  - A2 OVER_MARK — marks in more than one box in the same contest
  - A3 AMBIGUOUS_MARK — mark partially outside the designated box
  - A4 TORN_DEFACED — physical damage preventing intent determination
  - A5 VOTER_IDENTIFIABLE — voter's name, ID, or other identifying detail on ballot
  - A6 NON_STANDARD_MARK — mark type not covered by any accepted category
  - A7 WRONG_CONTEST_BALLOT — ballot issued for the wrong contest deposited in box
  - A8 UNAUTHENTICATED_BALLOT — no official stamp or barcode
  - V1 VALID_STANDARD_MARK — mark clearly in the designated box
  - V2 VALID_CROSS_ACCEPTED — cross-mark within box accepted per Elections Act adjudication standards
  - V3 VALID_AMBIGUOUS_ACCEPTED — ambiguous mark accepted per intent-of-voter principle
- **FR-M27-020** Kit records: adjudication event (reason code, Presiding Officer decision, party-agent signatures, timestamp). Adjudicated ballot photographed with kit camera; photo stored on-device (AES-256) and synced centrally; photo included in Form 34A attachments.
- **FR-M27-021** Party-agent dissent: if any party agent dissents from the Presiding Officer's determination, dissent is recorded with agent's name and reason; dissent log included in Form 34A.

### Element 5 — Voter Education

- **FR-M27-022** Multi-channel pre-election campaign: interactive "how to mark your ballot" tutorial in Public Verifier app (Learn tab); IVR audio guide in EN + SW + 5 additional Kenyan languages; sample ballot displayed at every polling station entrance; radio drama series (15-episode).
- **FR-M27-023** Practice ballot: voter can complete a practice marking exercise in the Public Verifier app; practice is anonymous and results are discarded; shows real ballot design.
- **FR-M27-024** Barrier-specific messaging: targeted messages for elderly voters (simplified audio), first-time voters (step-by-step illustrated), North Eastern and Coast regions (historically higher rejection rates — language-specific content).

### Element 6 — Presiding Officer Training

- **FR-M27-025** Standardised adjudication training module: 100+ example ballots with official adjudication reasoning; interactive quiz format; 30-day pre-election online refresher; in-person training with printed reference card.
- **FR-M27-026** Training completion tracked in CRMS (M12); no Presiding Officer may be deployed to a station without completing the online module and an in-person session.

### Element 7 — Ex-Post Reconciliation and Statistical Outlier Flagging

- **FR-M27-027** Per-station reconciliation after close: rejected + spoilt + valid + assisted = ballots issued. Any arithmetic mismatch triggers a Form 34A hold pending Presiding Officer explanation.
- **FR-M27-028** Statistical outlier flagging: any station with rejection rate > 3 standard deviations above the constituency mean is auto-flagged for IEBC Returning Officer review. Flag published in the post-election report.
- **FR-M27-029** IAB adjudication-accuracy audit: IAB samples >= 5% of adjudicated-ballot photographs post-election; compares IAB auditor's determination against Presiding Officer's; disagreement rate published. Target: > 99% agreement.
- **FR-M27-030** Post-election rejection analysis report: per-contest, per-county, per-reason-code breakdown; comparison to 2013/2017/2022 baselines; published within T+30.

## 4. Non-Functional Requirements

- **NFR-M27-001** Pre-mark validation inference time: <= 3 seconds on reference KIEMS kit hardware.
- **NFR-M27-002** Ballot image deleted from device memory within 1 second of inference completion; deletion verified by OS secure-erase call.
- **NFR-M27-003** On-device vision model: INT8 quantised, <= 10 MB, no internet call during inference.
- **NFR-M27-004** WCAG 2.2 AA: all voter-facing screens (opt-in prompt, result display); audio guidance in all supported languages.
- **NFR-M27-005** Languages: EN + SW baseline; local-language support for opt-in prompt and result display in 12 Kenyan languages.
- **NFR-M27-006** DPA 2019 lawful basis for pre-mark check: explicit voter consent (s.30(1)(a)); data minimisation (s.26 — image not stored); purpose limitation (s.26 — check only, not registration or identification).
- **NFR-M27-007** Adjudicated-ballot photographs: encrypted at rest (AES-256); accessible only to IEBC Returning Officer + IAB auditor under logged access events; deleted 7 years post-election.
- **NFR-M27-008** Spoilt-ballot count: reconciled against Form 34A within 60 minutes of station close.
- **NFR-M27-009** Rejection-rate outlier report: available to IEBC Returning Officers within 4 hours of Form 34C publication.

## 5. User Stories

1. As a first-time voter, I opt into the pre-mark check and the kit tells me I accidentally marked two boxes; I spoil my ballot and get a replacement without embarrassment.
2. As an elderly voter in Turkana, I hear the pre-mark check prompt in Turkana language via the kit's audio guide and choose to use it.
3. As a party agent, I witness the adjudication of an ambiguous ballot, state my position, and my dissent is recorded when I disagree with the Presiding Officer's determination.
4. As a Presiding Officer, I refer to my laminated reason-code card and correctly adjudicate a torn ballot as A4 TORN_DEFACED; the kit records my decision with my biometric sign-off.
5. As an IEBC Returning Officer, I see that Station 17 in my constituency has a 6.8% rejection rate vs a 0.9% constituency mean; I request the adjudicated-ballot images for review.
6. As an IAB Adjudication Auditor, I pull the 5% sample of adjudicated-ballot photographs and find 99.2% agreement with the Presiding Officer's reason codes.
7. As a voter, I complete the practice ballot in the Public Verifier app three days before the election; I know exactly what to do on election day.
8. As a data-protection officer, I verify that no ballot image or facial feature vector ever leaves the kit; the deletion log confirms secure erase within 1 second of inference.
9. As a voter who marked my ballot incorrectly for the third time, I am told I have used all three replacements and receive guidance from the Presiding Officer.
10. As a member of the public, I read the post-election rejection analysis and see that the 2027 rejection rate was 0.7%, down from 2.5% in 2017.

## 6. Data Model Additions

```python
class PreMarkValidationEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    station_code: str
    contest_id: str
    opted_in: bool
    result_code: Literal[
        "VALID","UNMARKED","MULTIPLE_MARKS","AMBIGUOUS_MARK","WRONG_BALLOT"
    ] | None        # null if opted_in = false
    voter_proceeded: bool | None    # null if opted_in = false
    event_at: datetime
    # NOTE: no voter_id field — event is station-aggregate only

class SpoiltBallotEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    station_code: str
    contest_id: str
    spoilt_number: int              # 1, 2, or 3 per voter per contest
    replacement_issued: bool
    presiding_officer_id: str
    event_at: datetime
    # NOTE: voter_id encrypted and session-only; cleared on session close

class AdjudicationEvent(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    station_code: str
    contest_id: str
    reason_code: str                # from standardised catalogue above
    presiding_officer_decision: Literal["VALID","REJECT"]
    presiding_officer_id: str
    party_agent_positions: dict[str, Literal["VALID","REJECT"]]  # agent_id -> position
    dissent_recorded: bool
    dissent_agent_id: str | None
    dissent_reason: str | None
    ballot_photo_blob_id: str       # encrypted; access-logged
    event_at: datetime
    merkle_node_id: str | None

class RejectionRateAlert(BaseModel):
    model_config = ConfigDict(extra='forbid', validate_by_name=True)
    id: str
    station_code: str
    contest_id: str
    station_rejection_rate: float
    constituency_mean_rejection_rate: float
    z_score: float
    alert_generated_at: datetime
    reviewed_by_officer_id: str | None
    review_outcome: str | None
    resolved_at: datetime | None
```

## 7. API Surface

| Method | Path | Auth | Rate limit | Description |
|--------|------|------|------------|-------------|
| POST | /v1/intent/pre-mark-events | KIT_CERT mTLS | 500/min | Submit pre-mark validation aggregate |
| POST | /v1/intent/spoilt-events | KIT_CERT mTLS | 200/min | Submit spoilt ballot event |
| POST | /v1/intent/adjudication | KIT_CERT mTLS | 100/min | Submit adjudication event |
| GET | /v1/intent/adjudication/{id} | IEBC_OFFICER JWT | 100/hr | Fetch adjudication detail |
| GET | /v1/intent/adjudication/{id}/photo | IAB_AUDITOR JWT | 50/hr | Access adjudicated-ballot photo (logged) |
| GET | /v1/intent/rejection-rates | IEBC_OFFICER JWT | 100/hr | Rejection rates per station/constituency |
| GET | /v1/intent/rejection-alerts | IEBC_RO JWT | 50/hr | Statistical outlier alerts |
| PATCH | /v1/intent/rejection-alerts/{id} | IEBC_RO JWT | 50/hr | Record review outcome |
| GET | /v1/intent/report | Public | 500/min | Post-election rejection analysis report |
| GET | /v1/intent/model-card | Public | 200/min | Pre-mark vision model card |

## 8. Integration Points

- **M08 RTS Device** — pre-mark validation, spoilt-ballot, and adjudication events originate on kit
- **M10 RTS Back-End** — events ingested and stored; rejection-rate computation
- **M26 Anomaly Detection** — rejection-rate outliers fed to Layer 2 peer-comparison; pre-mark validation disable events to Layer 1 telemetry
- **M11 Public Portal** — public rejection analysis report; Learn tab voter education
- **M28 AI Monitoring Stack** — on-device vision model card registered in model registry; eval harness
- **P05 Merkle Chain** — adjudication events anchored
- **IEBC Ballot Design Team** (internal) — ballot design test results feed M27 Element 1

## 9. UI/UX Key Screens

1. **Opt-In Prompt (Voter-Facing, on Kit)** — full-screen prompt with two large buttons: "Check my ballot" (green) and "Skip" (grey); audio plays the prompt automatically; 10-second auto-dismiss to Skip if no input.
2. **Pre-Mark Result Display (Voter-Facing, on Kit)** — large icon + plain-language text for each result code; if warning, "Get a replacement ballot?" button; if VALID, green tick + "Please deposit your ballot."
3. **Adjudication Workflow (Presiding Officer)** — shows ballot photo captured by kit camera; reason-code selector (tiles with code and description); party-agent biometric sign-off buttons (sequential); dissent checkbox per agent; final decision button with HSM sign.
4. **Spoilt-Ballot Counter (Presiding Officer)** — small persistent widget showing current voter's spoilt count (1/3, 2/3, 3/3); replacement issue button; seal-and-retain confirmation.
5. **Rejection Rate Dashboard (IEBC Returning Officer)** — table of stations in constituency sorted by rejection rate; z-score column; outlier badge; click-through to adjudication event list for that station.
6. **Public Rejection Analysis Report** — tabbed by contest; bar chart: reason-code distribution; choropleth: rejection rate by county; trend line: 2013-2017-2022-2027.

## 10. Compliance Mapping

| Requirement | Source |
|-------------|--------|
| Spoilt ballot replacement | Elections Act 2011 s.54 (up to 3 replacements per contest) |
| Voter assistance | Elections Act 2011 s.72 |
| Ballot secrecy during pre-mark check | Constitution Art. 38(3); DPA 2019 s.26 — no image stored |
| Intent-of-voter adjudication principle | Elections Act 2011 s.62; common law ballot-interpretation standards |
| Adjudication record | Elections Act s.39; Election Technology Regulations 2017 |
| Accessibility | Constitution Art. 54 (persons with disabilities); WCAG 2.2 AA |
| Data protection for ballot photos | DPA 2019 s.26 — data minimisation; s.30(1)(b) — legal obligation |
| Multi-language | Constitution Art. 7(3); National Cohesion and Integration Act |

## 11. Success Metrics + SLA

| Metric | Target | Measurement | Sign-off |
|--------|--------|-------------|----------|
| Rejected ballot rate per contest | < 1.0% nationally | Form 34A aggregate | IEBC Chair |
| Pre-mark validation inference time | <= 3 seconds | Kit performance test | Mechrevo QA |
| Ballot image deletion within 1 second of inference | 100% | OS erase log | Datacraft Security |
| Adjudication reason-code coverage | 100% of adjudicated ballots have a code | Form 34A audit | IAB |
| IAB adjudication-accuracy audit agreement rate | > 99% | IAB audit report | IAB |
| Rejection-rate outlier alert availability | <= 4 hours post Form 34C | Report timestamp | Datacraft Technical |
| Post-election rejection report published | <= T+30 | Publication timestamp | IEBC Chair |
| Public Learn tab visits pre-election | > 2M | Portal analytics | IEBC Comms |

## 12. Test Strategy

- **Unit:** reason-code catalogue validator; pre-mark inference pipeline (mock model); spoilt-ballot counter; rejection-rate z-score computation.
- **Integration:** full pre-mark flow (opt-in, inference, result display, replacement offer); adjudication event recording; Form 34A reconciliation with spoilt and rejected counts.
- **E2E:** simulated polling station with 200 synthetic voters including opted-in pre-mark checks, 5 spoilts, 3 adjudications.
- **Privacy:** confirm ballot image is deleted within 1 second (hook into OS secure-erase API); confirm no image or feature vector in any log or network packet.
- **Accessibility:** audio-guided opt-in flow for visually impaired voters; large-text mode for low-vision voters; WCAG 2.2 AA.
- **Model validation:** accuracy of on-device vision model on a held-out test set of 1,000 synthetic ballot images annotated by IEBC ballot-design team; target >= 97% recall on VALID class.
- **Load:** 55,393 stations submitting rejection-rate data simultaneously; outlier computation within 4 hours.
- **Offline:** 8-hour offline session; verify all adjudication events sync on reconnect with integrity preserved.

## 13. Rollout

| Milestone | Date | Description |
|-----------|------|-------------|
| Ballot design study complete | 2026-10-31 | 200-voter test across literacy strata |
| Vision model trained and validated | 2026-12-01 | On synthetic + annotated ballot images |
| Voter education content produced | 2027-03-01 | All channels and languages |
| Presiding officer training module live | 2027-04-01 | Online + in-person |
| Ballot design published (T-30) | 2027-07-11 | For party-agent review |
| Kit firmware with pre-mark validation | 2027-06-01 | Integrated in standard firmware |
| Dry-run on by-election | 2027-02-28 | Real voters; pre-mark opt-in tested |
| Post-election rejection report | 2027-09-10 | T+31 |

## 14. Dependencies

- M08 RTS Device — on-kit vision model deployment
- M10 RTS Back-End — event ingestion and rejection-rate analytics
- M26 Anomaly Detection — rejection-rate outlier integration (Layer 2 and Layer 7)
- M28 AI Monitoring Stack — pre-mark vision model registration, model card, eval harness
- M11 Public Portal — Learn tab voter education; public rejection analysis report
- P05 Merkle Chain — adjudication event anchoring
- IEBC Ballot Design Team — ballot design sign-off (external to Datacraft; IEBC-owned process)
- National Languages Board / translators — 12-language pre-mark prompt translation

## 15. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Vision model misclassifies a valid ballot as invalid, causing voter to unnecessarily spoil | Medium | Medium | Voter retains the right to override the model's result and deposit as-is; model result is advisory, not authoritative; high recall on VALID class targeted (>= 97%) to minimise false invalids |
| Voters feel coerced into opting in to the pre-mark check | Medium | High | Opt-in is explicit with a prominent Skip button; 10-second auto-dismiss defaults to Skip; audio prompt emphasises "this is optional"; presiding officer training includes anti-coercion instruction |
| Adjudicated-ballot photographs accessed without authorisation | Low | High | Access logged with officer identity and reason; IAB auditor and Returning Officer are the only roles with access; access outside those roles triggers a DPA 2019 breach notification |
| Pre-mark validation disable button used to suppress check for all voters at a suspect station | Medium | Medium | Disable requires party-agent witness + reason code; disable event feeds M26 Layer 1 telemetry alert; ECC Analyst reviews any station with > 5% disable rate |
| Rejection rate remains above 1.0% despite all interventions | Low | Medium | Public report published with reason-code breakdown and geographic analysis; failure to hit target is not a system failure but an input to the next-cycle improvement programme |
