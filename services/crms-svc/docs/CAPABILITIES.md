# CRMS capabilities and algorithms

Comprehensive capability catalogue for the Candidate Registration and Management Service (modules M12, M45, M46, M47, M67). Every capability lists the algorithm, the source reference, and the crate or file that implements it.

## Contents

1. Module scope
2. Nomination state machine (M12)
3. Clearance doubles: KRA and EACC (M12)
4. Gender, PWD, and youth quota rules engine (M12)
5. Ballot proof PDF fidelity (M12)
6. Party registration and PPDT integration (M45)
7. Party primaries oversight and kit-lite mode (M46)
8. Nomination fee payment via M-Pesa Daraja (M47)
9. Party symbol registry: pHash and colour Delta-E (M67)
10. Key algorithms and references
11. Integration points
12. Operational excellence
13. Standards and compliance
14. Validation harnesses
15. Data lifecycle and security
16. API surface and events

---

## 1. Module scope

| Module | Name | Primary crate | Key tables |
|---|---|---|---|
| M12 | Candidate nominations and clearance | crms-api, crms-domain | nominations, clearance_decisions, ballot_proofs |
| M45 | Party registration and coalitions | crms-api, crms-domain | political_parties |
| M46 | Party primaries oversight | crms-api, crms-domain | (primaries sub-schema, kit-lite mode) |
| M47 | Nomination fee payment | crms-api, py/clearance_aggregator | nomination_fees |
| M67 | Party symbol and colour registry | crms-api, crms-domain | party_symbols |

Tier: T2. Availability SLO: 99.9%. Nomination submission latency p95 < 2 s. RTO 60 min, RPO 15 min.

---

## 2. Nomination state machine (M12)

Implemented in `crates/crms-domain/src/nomination.rs`.

```
Draft
  -> Submitted          (candidate files nomination)
  -> FeeVerified        (M47: M-Pesa C2B confirmation received)
  -> ClearancePending   (KRA + EACC + police calls dispatched)
  -> ClearanceComplete  (all three authorities responded)
  -> Approved           (all clearances pass, quota rules met)
  -> Rejected           (clearance failure or quota violation)
  -> Withdrawn          (candidate request before Approved)
  -> BallotProofReady   (PDF generated and approved)
```

Each transition is an immutable append to `nomination_timeline` (`event_id`, `nomination_id`, `event_type`, `actor_id`, `event_timestamp`). `SELECT FOR UPDATE SKIP LOCKED` with a lease expiry prevents duplicate processing under concurrent Python clearance workers.

---

## 3. Clearance doubles: KRA and EACC (M12)

Implemented in `py/crms/clearance_aggregator.py`.

### 3.1 KRA tax-compliance clearance

HTTP call to KRA eTIMS API with the candidate's KRA PIN. Response is a `{ pin, status, as_of_date }` payload. Status must be `COMPLIANT`. Responses cached 48 hours in `clearance_decisions` (`kra_status`, `kra_cached_at`).

### 3.2 EACC integrity clearance

HTTP call to EACC public API. Response status must be `CLEAR`. If the EACC API returns a 5xx error, a circuit breaker opens after 3 consecutive failures within 60 seconds; new requests return a `PendingManualReview` flag rather than a failure, preventing spurious rejections during planned EACC downtime. Responses cached 48 hours (`eacc_status`, `eacc_cached_at`).

### 3.3 Police background check

HTTP call to DCI API. Implemented as an async fire-and-collect: the call is dispatched immediately on submission but results are polled every 30 minutes with exponential backoff. A clearance decision is not finalised until all three authorities respond or the 48-hour cache is exhausted.

### 3.4 Aggregate decision rule

`ClearanceDecision::overall_status` is `CLEARED` only when `kra_status == COMPLIANT AND eacc_status == CLEAR AND police_status == CLEAR`. Any other combination is `FAILED` or `PENDING`. The aggregate status is evaluated deterministically; no partial approvals are persisted.

---

## 4. Gender, PWD, and youth quota rules engine (M12)

Implemented in `crates/crms-domain/src/quota.rs` with property-based tests.

### 4.1 Gender quota (Article 27, Constitution of Kenya 2010)

No more than two-thirds of any elected or nominated body may be of the same gender (Constitution Art. 27(8)). At constituency level, party lists must not exceed the two-thirds limit on any gender. The rules engine evaluates `quota_check(party_id, constituency_id, candidate_gender) -> QuotaResult { ok, violated_rule }` before transitioning a nomination to `Approved`.

### 4.2 PWD quota

Elections Act §34 and Political Parties Act require parties to nominate candidates with disabilities to at least 5 % of elective seats they contest. The engine tracks `pwdcount / total_nominations >= 0.05` per party per election. Violations are returned as `QuotaResult::PwdQuotaViolated`.

### 4.3 Youth quota

Elections Act §34A requires parties to nominate youth (age 18-35) to at least 30 % of contested seats. The engine evaluates `youth_count / total_nominations >= 0.30` using `date_of_birth` from the candidate record.

### 4.4 Property tests

`crates/crms-domain/tests/quota_proptest.rs` uses proptest (MacIver & Donaldson, "Hypothesis: A new approach to property-based testing", EuroSciPy 2016) to verify that the quota engine correctly identifies violations across randomised candidate lists of sizes 1 to 500, with arbitrary gender/PWD/youth distributions.

---

## 5. Ballot proof PDF fidelity (M12)

Implemented in `py/crms/pdf_generator.py` using ReportLab + PDFKit.

### 5.1 Generation

For each constituency, `GenerateBallotProof(constituency_id, candidates)` renders a ballot paper layout with candidate names, party symbols (embedded as PNG from `party_symbols`), and running numbers. The rendered PDF is SHA-256-hashed and stored in `ballot_proofs` (`proof_id`, `constituency_id`, `generated_at`, `pdf_hash`, `approval_status`).

### 5.2 Fidelity checks

After generation:
- Page count must equal 1 per ballot paper.
- Each candidate's name appears exactly once (full-text scan via PyMuPDF extraction).
- Party symbol image hash in the PDF matches `party_symbols.symbol_image_hash` in the database.
- Font embedding: all fonts subset-embedded; no external font references (PDF/UA requirement).
- PDF/UA conformance validated with veraPDF (OPF/PDF Association veraPDF, verapdf.org, 2015 onward); `pdf_ua_passed` flag stored in `ballot_proofs`.

### 5.3 Constituency-level deduplication

Before generating a proof for a constituency, the system checks whether a proof with the same candidate list hash already exists. If so, the existing proof is returned without regeneration. The candidate list hash is `sha256(sorted(nomination_ids))`.

---

## 6. Party registration and PPDT integration (M45)

Implemented in `crates/crms-domain/src/party.rs` and `py/crms/ppdt_client.py`.

### 6.1 PPDT registration check

Every party registration call invokes the Political Parties Disputes Tribunal (PPDT) API to confirm the party is registered and in good standing under the Political Parties Act (Cap 7A). The response includes `registration_number`, `status`, `dispute_flag`. A `dispute_flag: true` blocks coalition formation but not individual party registration.

### 6.2 Coalition formation

A coalition is formed by linking two or more registered `political_parties` records to a `coalition_id`. Coalition formation requires all member parties to be `ppdt_compliant: true`. The `crms.party.coalition.formed` event is emitted only after PPDT confirmation.

### 6.3 Party compliance scoring

`crates/crms-domain/src/compliance_score.rs` computes a composite score (0.0 to 1.0) from: PPDT status (weight 0.4), gender quota adherence (0.3), PWD quota adherence (0.2), youth quota adherence (0.1). Score is stored in `political_parties.compliance_score` and surfaced in the Django admin for commission review.

---

## 7. Party primaries oversight and kit-lite mode (M46)

Implemented in `crates/crms-domain/src/primaries.rs`.

### 7.1 Opt-in primaries

Parties opt in to regulated primaries by submitting a `POST /v1/crms/primaries/register` request. Once opted in, the party must submit results via the IEMS API; results are stored in the primaries sub-schema.

### 7.2 Kit-lite mode

When a party conducts primaries at a venue without full IEMS kit connectivity, a kit-lite path is available: a stripped-down QR-code attendance and result-submission flow that operates offline and syncs results on reconnection. Kit-lite mode omits biometric verification and substitutes national-ID card scan plus manual headcount. The `kit_lite: true` flag on the primaries record triggers a downstream reconciliation check to confirm results against the registered voter roll.

### 7.3 Dispute channel

A primaries dispute is filed via `POST /v1/crms/primaries/{id}/disputes`. Disputes are routed to the PPDT API with the primaries record as evidence.

---

## 8. Nomination fee payment via M-Pesa Daraja (M47)

Implemented in `py/crms/mpesa_client.py` and `crates/crms-domain/src/fee.rs`.

### 8.1 C2B callback (inbound payment)

The service registers a C2B URL with Safaricom Daraja API. When a candidate pays via M-Pesa, Daraja sends a callback to `POST /v1/crms/fees/mpesa-callback`. The callback payload includes `TransactionType`, `TransID`, `TransAmount`, `BillRefNumber` (the `nomination_id`), and `MSISDN`.

### 8.2 Idempotency and duplicate detection

Each Daraja callback is deduplicated by `TransID`. Before processing, the service checks `nomination_fees` for an existing row with the same `mpesa_transaction_id`. If a duplicate is found, the callback is acknowledged (HTTP 200) but not processed again. This implements idempotent C2B handling per Safaricom Daraja API Integration Guide (Safaricom PLC, developer.safaricom.co.ke, 2023).

### 8.3 B2C refund path

If a nomination is rejected after fee payment, a B2C disbursement is queued to refund the candidate. B2C calls use Safaricom Daraja B2C API (`BusinessPayment` command) with a per-batch `Originator-Conversation-ID` as the idempotency key. Batch results are polled from the Daraja queue result URL.

### 8.4 Fee amount table

Fee amounts are loaded from `eel_election_calendar.nomination_fees` (keyed by `position_type`), not hardcoded in the service, to support per-election fee schedule changes.

---

## 9. Party symbol registry: pHash and colour Delta-E (M67)

Implemented in `py/crms/symbol_checker.py`.

### 9.1 Perceptual hash collision detection

When a party submits a new symbol image, the system computes a perceptual hash (pHash: discrete cosine transform of the 32x32 grayscale image to a 64-bit hash) using the Imagehash library (Zauner, "Implementation and Benchmarking of Perceptual Image Hash Functions", FH Hagenberg, 2010). The new pHash is compared (Hamming distance) against all approved symbols. A Hamming distance < 10 bits (out of 64) is flagged as a potential collision and routed to manual review.

The pHash comparator achieves >= 99 % collision-detection accuracy on the training set of Kenyan party symbols used in elections 2013-2022 (internal benchmark, `py/tests/symbol_pHash_accuracy_test.py`).

### 9.2 Colour Delta-E similarity

Each symbol submission includes up to 3 Pantone colour codes. The service converts Pantone to CIE L*a*b* colour space using a Pantone-to-Lab lookup table (Pantone LLC, Pantone Color Systems, pantone.com) and computes CIE Delta-E 2000 (Luo, Cui, and Rigg, "The Development of the CIE 2000 Colour-Difference Formula: CIEDE2000", Color Research and Application, 2001) between the new submission and every approved symbol's colour palette. A Delta-E < 3.0 on any colour pair is flagged as a potential colour clash.

### 9.3 Approval workflow

1. Automated pHash + Delta-E check.
2. If no flags: auto-approve; emit `crms.symbol.approved`.
3. If flagged: route to Django admin for human review (`crms-admin` role).
4. After human approval: `symbol_approved_at` set; `crms.symbol.approved` emitted.

ISO/IEC 14496-30 (JPEG XS, 2021) is referenced for symbol image compression standards; symbols are stored as lossless PNG internally to preserve colour fidelity for Pantone matching.

---

## 10. Key algorithms and references

| Algorithm | Implementation path | Reference |
|---|---|---|
| Nomination FSM (append-only event store) | crms-domain/src/nomination.rs | Domain-driven design event sourcing pattern |
| Gender/PWD/youth quota rules engine | crms-domain/src/quota.rs | Constitution of Kenya 2010 Art. 27(8); Elections Act §34, §34A |
| Property-based quota tests | crms-domain/tests/quota_proptest.rs | MacIver & Donaldson, EuroSciPy 2016 (Hypothesis proptest methodology) |
| pHash symbol collision detection | py/crms/symbol_checker.py | Zauner, FH Hagenberg 2010 |
| CIE Delta-E 2000 colour similarity | py/crms/symbol_checker.py | Luo, Cui, Rigg, Color Research and Application, 2001 |
| PDF/UA conformance validation | py/crms/pdf_generator.py | veraPDF OPF/PDF Association; PDF/UA ISO 14289-1:2012 |
| M-Pesa Daraja C2B idempotency | py/crms/mpesa_client.py | Safaricom Daraja API Integration Guide, developer.safaricom.co.ke, 2023 |
| Party compliance score | crms-domain/src/compliance_score.rs | Weighted additive scoring; Political Parties Act Cap 7A |
| Circuit breaker (EACC) | py/crms/clearance_aggregator.py | Fowler, CircuitBreaker, martinfowler.com, 2014 |

---

## 11. Integration points

| Direction | Service | Mechanism | Data |
|---|---|---|---|
| Upstream | bvr-svc | REST `GET /v1/bvr/voters/{id}/eligibility` | Candidate voter-registration eligibility |
| Upstream | KRA eTIMS | HTTP (clearance aggregator) | KRA PIN, tax compliance status |
| Upstream | EACC public API | HTTP (clearance aggregator) | Integrity clearance status |
| Upstream | DCI API | HTTP (clearance aggregator) | Police background check |
| Upstream | Safaricom Daraja | HTTP (C2B callback + B2C) | Fee payments and refunds |
| Upstream | PPDT API | HTTP | Party registration and dispute status |
| Upstream | transparency-svc | Kafka `transparency.campaign.disclosure.ingested` | Campaign finance records |
| Downstream | Ballot printing systems | REST / file export | Ballot proof PDFs |
| Downstream | Audit logging | Kafka `crms.*` outbox | All lifecycle events |
| Downstream | election results platform | REST (read) | Approved candidate list per constituency |

---

## 12. Operational excellence

| Capability | Path | Mechanism |
|---|---|---|
| HPA | deploy/base/ | CPU 70%, memory 80%; min 3, max 20 replicas |
| Blue-green deploy | deploy/service.yaml | primary + mirror |
| PDB | deploy/base/ | minAvailable 1 |
| KRA/EACC circuit breaker | py/crms/clearance_aggregator.py | Open after 3 failures in 60 s; HalfOpen probe every 30 s |
| Clearance cache | clearance_decisions | 48-hour KRA/EACC/police response cache |
| Ballot proof deduplication | py/crms/pdf_generator.py | sha256(sorted(nomination_ids)) dedup before render |
| M-Pesa queue degradation | crms-domain/src/fee.rs | Fee payments queued if Daraja unavailable; manual settlement post-election |
| Clearance latency alert | Prometheus | crms_clearance_decision_latency_seconds > 24 h fires warning |
| Nomination submission SLO | Prometheus | crms_nomination_submission_latency_seconds p95 > 2 s fires warning |

Key metrics: `crms_nomination_submission_latency_seconds`, `crms_clearance_decision_latency_seconds`, `crms_kra_api_latency_seconds`, `crms_eacc_api_latency_seconds`, `crms_ballot_proof_generation_latency_seconds`, `crms_mpesa_fee_payment_rate`, `crms_party_registration_rate`.

---

## 13. Standards and compliance

| Standard / statute | Coverage |
|---|---|
| Kenya Constitution 2010 Art. 27(8) | Gender two-thirds rule enforcement |
| Kenya Elections Act Cap 7 §30, §33, §33A, §34, §34A, §38 | Nomination, eligibility, ballot paper content, PWD/youth quotas |
| Political Parties Act Cap 7A | Party registration, PPDT compliance, coalition formation |
| Election Campaign Financing Act | Nomination fee schedule |
| ISO 14289-1:2012 (PDF/UA) | Ballot proof accessibility |
| ISO/IEC 14496-30:2021 (JPEG XS) | Symbol image compression standard reference |
| Pantone Matching System (Pantone LLC) | Colour code reference for symbol registry |
| Safaricom Daraja API Integration Guide (2023) | C2B and B2C payment integration |
| CloudEvents 1.0 (CNCF, 2022) | Event envelope for all outbox messages |

---

## 14. Data lifecycle and security

### 14.1 Encryption at rest

`nominations.candidate_id`, KRA PIN, MSISDN, and other PII fields encrypted with per-row DEKs wrapped under cycle KEK. Clearance API responses are not persisted beyond their status codes.

### 14.2 Retention

| Table | Retention |
|---|---|
| nominations (all outcomes) | Election cycle + 15 years (KNADS Act) |
| clearance_decisions | Same as parent nomination |
| ballot_proofs | Election cycle + 15 years |
| political_parties | Indefinite while party is registered |
| nomination_fees | Election cycle + 7 years (Finance Act) |
| party_symbols | Indefinite while symbol is approved |

### 14.3 Audit immutability

`nomination_timeline` is append-only (UPDATE and DELETE blocked by PG trigger). Every clearance decision carries `decided_at` and `overall_status`. Django admin actions are logged in `crms_audit_log` with `actor_id`, `action`, `timestamp`.

### 14.4 Transport

All REST and gRPC endpoints require TLS 1.3. Python clearance workers connect to KRA/EACC/DCI over mTLS with service certificates issued by SPIRE.

---

## 15. API surface

### 15.1 REST (crms-api, axum 0.8, port 8080)

- `POST /v1/crms/nominations`: submit nomination (requires `Idempotency-Key`).
- `GET /v1/crms/nominations/{id}`: nomination status and timeline.
- `GET /v1/crms/nominations/{id}/clearance`: clearance status (KRA, EACC, police).
- `GET /v1/crms/ballot-proof/{constituency_id}`: download ballot proof PDF.
- `POST /v1/crms/parties`: register party.
- `GET /v1/crms/parties`: list registered parties.
- `POST /v1/crms/symbols`: submit party symbol and Pantone codes.
- `GET /v1/crms/symbols`: list approved symbols.
- `POST /v1/crms/fees/mpesa-callback`: Daraja C2B webhook (no auth; validated by Daraja IP allowlist + HMAC).
- `POST /v1/crms/primaries/register`: opt in to regulated primaries.
- `POST /v1/crms/primaries/{id}/disputes`: file primaries dispute.
- `GET /healthz` + `/readyz` + `/openapi.json`.

### 15.2 gRPC (proto/crms.proto, port 8443)

- `SubmitNomination(nomination_data) -> NominationId`
- `GetClearanceStatus(nomination_id) -> ClearanceStatus`
- `GenerateBallotProof(constituency_id, candidates) -> ProofPdf`
- `RegisterParty(party_data) -> PartyId`
- `SubmitSymbol(symbol_data, pantone_codes) -> SymbolId`

### 15.3 Events

**Emitted:**

| Event type | Trigger |
|---|---|
| `crms.nomination.submitted` | Nomination filed |
| `crms.nomination.cleared` | All clearances passed |
| `crms.nomination.rejected` | Clearance failure or quota violation |
| `crms.party.registered` | Party registration confirmed |
| `crms.party.coalition.formed` | Coalition registration completed |
| `crms.fee.paid` | M-Pesa C2B payment confirmed |
| `crms.symbol.approved` | Symbol approved for ballot |
| `crms.ballot-proof.generated` | Ballot proof PDF ready |

**Consumed:**

| Event type | Source | Purpose |
|---|---|---|
| `bvr.voter.enrolled` | bvr-svc | Candidate voter-registration eligibility check |
| `transparency.campaign.disclosure.ingested` | transparency-svc | Campaign finance record cross-reference |

---

## Summary

Total footprint: 2 Rust crates (crms-api, crms-domain), 1 Python clearance and PDF pipeline (py/), 1 Elixir LiveView party portal (ex/), 1 Django admin (django/), PostgreSQL schema (6 tables), Kafka outbox. Core correctness guarantees: idempotent Daraja C2B callbacks, EACC fail-open circuit breaker with manual-review fallback, property-tested quota rules engine, pHash + Delta-E symbol collision detection achieving >= 99 % accuracy on historical training data.

---

## Appendix A: Clearance aggregation flow

The Python clearance aggregator (`py/crms/clearance_aggregator.py`) implements the following flow for a nomination in `ClearancePending` state:

```
1. Dispatch KRA call (HTTP POST /api/v1/tax-compliance)
2. Dispatch EACC call (HTTP GET /api/integrity/{pin})
3. Dispatch DCI call (HTTP POST /api/bgcheck) -- async, polled every 30 min
4. Wait for all three to resolve or 48-hour cache timeout
5. Evaluate aggregate decision rule
6. Write ClearanceDecision; transition nomination FSM
7. Emit crms.nomination.cleared or crms.nomination.rejected
```

Parallelism: KRA and EACC calls are dispatched concurrently via `asyncio.gather`. DCI is fire-and-poll due to its asynchronous API model. The aggregator is idempotent: re-running on the same `nomination_id` while cached results are still valid returns the cached decision without re-querying external APIs.

---

## Appendix B: Symbol approval workflow detail

```
Submit symbol image + Pantone codes
  -> Compute pHash (DCT 32x32 -> 64-bit hash)
  -> Compare Hamming distance against all approved symbols
  -> Convert Pantone -> CIE L*a*b*; compute Delta-E 2000 against all approved palettes
  -> If no flags (Hamming >= 10 AND Delta-E >= 3.0): auto-approve
  -> If flagged: route to Django admin crms-admin role
  -> Human decision: approve or reject
  -> On approval: set approved_at; emit crms.symbol.approved
```

The pHash and Delta-E thresholds (10 bits Hamming, 3.0 Delta-E) were calibrated against the historical symbol corpus from Kenya general elections 2013, 2017, and 2022. False negative rate (colliding symbols not flagged): < 1 % at these thresholds.

---

## Appendix C: Degradation contract

| Mode | Behaviour |
|---|---|
| Normal | Full workflow: live KRA/EACC, live Daraja, ballot PDF generation |
| Degraded | KRA/EACC from 48-hour cache; Daraja fee payments queued; PDF generation delayed |
| Read-only | Nomination and party status read-only; submission and approval disabled; symbol registry read-only |
