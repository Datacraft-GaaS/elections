# CRMS (M12)

Module M12 (`crms`) owned by `crms-svc`. Candidate and Nomination Registration Management System: candidate + party nomination FSM, KRA + EACC + DCI clearance doubles, gender-quota enforcement, gazette publishing, and ballot-proof generation.

## Contents

1. What M12 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M12 does

M12 manages the lifecycle of candidates standing for election: party nomination, Independent Electoral and Boundaries Commission (IEBC) acceptance, statutory clearance from Kenya Revenue Authority (KRA), Ethics and Anti-Corruption Commission (EACC), and Directorate of Criminal Investigations (DCI), gender-quota verification, gazette publication of candidate lists, and generation of signed ballot proofs used by the ballot printing chain.

## 2. Data plane

Crates: `services/crms-svc/crates/crms-domain/src/` (candidate.rs, nomination.rs, clearance.rs, coalition.rs, gazette.rs, ballot.rs, party.rs, quota.rs, primaries.rs). Django back-office: `services/crms-svc/django/`.

| Aggregate | Purpose | Data class |
|---|---|---|
| **Candidate** | candidate_id (UUID7), national_id_hash, party_id, constituency_code, position_type, status (NOMINATED/CLEARED/GAZETTED/REJECTED/WITHDRAWN) | A |
| **NominationRecord** | nomination_id, candidate_id, form_submitted_at, party_endorsement_sig, officer_verified_at | A |
| **ClearanceBundle** | bundle_id, candidate_id, kra_clearance (PASS/FAIL/PENDING), eacc_clearance, dci_clearance, verified_at | A |
| **QuotaCheck** | check_id, election_scope, gender_quota_met (bool), male_count, female_count, analysis_ts | B |
| **GazetteNotice** | notice_id, candidate_ids[], gazette_ref, published_at, gov_printer_receipt_id | B |
| **BallotProof** | proof_id, constituency_code, candidate_list_hash, Ed25519_sig, published_at | A |
| **Coalition** | coalition_id, party_ids[], agreement_hash, effective_from | B |

## 3. Ingest sources

| Source | Auth | Notes |
|---|---|---|
| Party nomination officer | JWT + SpiceDB `PARTY_OFFICER` | Uploads nomination form + party endorsement signature |
| Commission vetting officer | JWT + SpiceDB `COMMISSION_OFFICER` | Verifies nomination, triggers clearance requests |
| KRA (tax compliance certificate) | SVID mTLS + KRA secret | Clearance double in `integrations/kra-itax/` |
| EACC (integrity clearance) | SVID mTLS + EACC secret | Clearance double in `integrations/eacc/` |
| DCI (criminal record check) | SVID mTLS + DCI secret | Clearance double in `integrations/dci/` |
| Gov Printer (gazette) | `integrations/gov-printer/` | XML envelope schema; receipt ID stored |

## 4. Pipeline

Crates: `services/crms-svc/crates/crms-domain/src/nomination.rs`, `clearance.rs`, `quota.rs`, `gazette.rs`, `ballot.rs`.

```
party POST nomination form (JWT, party endorsement sig)
      -> validate NominationRecord: form completeness, party membership, position eligibility
      -> Commission officer verifies nomination (biometric-gated)
      -> trigger parallel clearance requests:
             KRA: tax compliance certificate (SVID mTLS, circuit breaker)
             EACC: integrity clearance (SVID mTLS, circuit breaker)
             DCI: criminal record check (SVID mTLS, circuit breaker)
      -> ClearanceBundle assembled on all three responses:
             all PASS -> Candidate.status = CLEARED
             any FAIL -> Candidate.status = REJECTED; reason code stored
             any PENDING -> timer alert if > SLA (48 h configurable)
      -> QuotaCheck: count CLEARED candidates per position per party
             gender_quota_met? (Elections Act s.27 one-third rule)
             not met -> alert party officer; block gazette for party
      -> gazette compilation:
             all parties quota-met -> GazetteNotice compiled
             Gov Printer XML envelope submitted via integrations/gov-printer/
             receipt_id stored; GazetteNotice.published_at set
      -> BallotProof generation:
             candidate_list_hash = SHA-256(sorted GAZETTED candidates)
             Ed25519 sig by Commission HSM key
             BallotProof published to transparency log (P05)
             consumed by ballot printing chain
      -> emit eel.crms.candidate_gazetted.v1
             eel.crms.ballot_proof_published.v1
```

## 5. API surface

Contract: `packages/openapi-contracts/crms/nomination.yaml`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /crms/v1/nominations` | Party officer (JWT + SpiceDB) | Submit candidate nomination |
| `GET /crms/v1/nominations/{id}` | Commission officer (JWT + SpiceDB) | Review nomination detail |
| `PATCH /crms/v1/nominations/{id}/verify` | Commission officer (JWT, biometric-gated) | Accept or reject after clearance |
| `GET /crms/v1/clearances/{candidate_id}` | Commission officer (JWT + SpiceDB) | Clearance bundle status |
| `GET /crms/v1/quota/{scope}` | Commission officer (JWT + SpiceDB) | Gender quota compliance check |
| `POST /crms/v1/gazette` | Commission admin (JWT + SpiceDB, HSM-gated) | Compile and submit gazette notice |
| `GET /crms/v1/ballots/{constituency_code}` | Ballot printer (SVID mTLS) | Retrieve signed BallotProof |
| `GET /crms/v1/candidates` | Public (rate-limited) | Published gazetted candidate list |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Commission back-office (Django) | `services/crms-svc/django/` | Nomination review, clearance tracking, quota dashboard, gazette compilation |
| Party officer portal | Django (party-facing role) | Nomination submission, status tracking |
| Public candidate list | REST `GET /crms/v1/candidates` + portal-svc | Published gazetted candidates |

Languages: English + Kiswahili. WCAG 2.1 AA. Quota dashboard shows gender distribution per party per position in real time.

## 7. Integrity + anti-tamper

- **Party endorsement signature** on every nomination; Commission officer biometric-confirmation required to accept.
- **BallotProof HSM-signed** -- `candidate_list_hash` signed by Commission HSM key before submission to ballot printing chain; any candidate list modification invalidates the proof.
- **Transparency log anchoring** -- BallotProof published to P05 tlog; inclusion proof retrievable for court evidence.
- **Clearance circuit breakers** (`packages/eel-resilience-rs`) -- KRA/EACC/DCI outages do not corrupt the ClearanceBundle; PENDING state held until response received.
- **Gazette receipt** -- Gov Printer receipt_id stored in GazetteNotice; gazette cannot be declared published without receipt.
- **Immutable gazette** -- once GazetteNotice.published_at is set, no candidate can be added or removed without a superseding gazette notice (new GazetteNotice with amendment flag).
- **Quota block** -- gender quota check enforced at gazette compilation time; no gazette possible while any party's quota is unmet; not bypassable by API.
- **Audit shadow** -- every nomination state change logged with actor, ts_hlc, and reason code.

## 8. Kenya statute alignment

- **Elections Act s.22 (nomination)** -- nomination FSM enforces submission deadlines and documentation requirements.
- **Elections Act s.27 (one-third gender rule)** -- QuotaCheck enforces the constitutional gender quota; gazette blocked if unmet.
- **Political Parties Act s.25 (party primaries)** -- party endorsement signature on nomination records that primaries result is attached.
- **Elections (General) Regulations Reg. 47 (clearance)** -- KRA + EACC + DCI clearances required; ClearanceBundle tracks all three.
- **Kenya Gazette Act** -- GazetteNotice submitted via Gov Printer integration; receipt ID is the legal publication confirmation.

## 9. Honest caveats + external prerequisites

- **KRA, EACC, DCI API MoUs** are external prerequisites; adapter doubles available under `integrations/{kra-itax,eacc,dci}/`; live endpoints require IEBC MoU.
- **Gov Printer XML schema** is documented in `integrations/gov-printer/`; exact schema locked at first live gazette submission.
- **Clearance SLA** (48 h default) depends on the external agencies responding within that window; circuit-breaker logic handles delays but cannot force agency response.
- **Primaries result attachment** -- CRMS stores party primaries result hash but does not run or verify primaries; that is the party's responsibility. CRMS only records the endorsed candidate.
- `services/crms-svc` has no CAPABILITIES.md; capability detail is embedded in the domain crates.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (nomination FSM, quota calculator, clearance aggregator, gazette validator) | nextest | M12-UNIT- |
| Property (quota invariant: gazette never published while quota unmet) | proptest | M12-PROP- |
| Contract (KRA/EACC/DCI doubles; ballot printer BallotProof consumer) | Pact | M12-PACT- |
| Integration (clearance doubles; HSM SoftHSM; Gov Printer double; tlog) | nextest + testcontainers | M12-INT- |
| E2E (full nomination-to-gazette flow; quota block; ballot proof) | Django test + nextest | M12-E2E- |
| Security (clearance double-submission bypass; quota override; gazette injection) | ZAP + manual | M12-SEC- |

Runbook (to create): `docs/runbooks/crms-svc-degradation.md` (clearance agency down; gazette submission failure; quota block escalation).

Full spec: `docs/plans/modules/crms.md`. Domain crates: `services/crms-svc/crates/crms-domain/src/`. Django back-office: `services/crms-svc/django/`.
