# Party registration management (M45)

Module M45 (`party-registration-management`) owned by `crms-svc` (crate `crms-api`, domain models in `crms-domain`). Automates the full lifecycle of political party registration and renewal under the Political Parties Act (PPA) 2011 and the Registrar of Political Parties (RPP) mandate: new registration, annual returns, coalition formation, symbol/colour metadata sync, governance disclosure, faction dispute intake routed to the Political Parties Disputes Tribunal (PPDT), and compliance scoring.

## Contents

1. What M45 does (vs M12 candidate management and M46 primaries)
2. Data plane (schema)
3. Registration and renewal lifecycle
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M45 does (vs M12 candidate management and M46 primaries)

- **M12** = candidate registration; a candidate's party affiliation is a FK to M45 `PoliticalParty.party_id`.
- **M46** = party primaries oversight; requires M45 `PoliticalParty.status = ACTIVE` as a prerequisite gate before a party can opt into the primaries module.
- **M45** = the upstream truth about party legal existence, governance structure, membership, symbol/colour, and compliance score. It does not manage candidates or primaries; it is the identity and status source for both.

## 2. Data plane

Four aggregates (per `services/crms-svc/crates/crms-domain/src/models/party.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **PoliticalParty** | Root entity: `party_id` (UUID7), `name`, `abbreviation`, `rpp_registration_number`, `founding_date`, `national_headquarters_county`, `status` (`PENDING / ACTIVE / SUSPENDED / DEREGISTERED`), `membership_count_verified`, `compliance_score` (0-100), `election_cycle_eligibility[]`, `symbol_id` (FK to M67 symbol registry). | A |
| **CoalitionRegistration** | Pre-election coalition: `coalition_id` (UUID7), `coalition_name`, `member_party_ids[]`, `presidential_candidate_party_id`, `registered_at`, `election_cycle`, `status` (`ACTIVE / DISSOLVED`), `dissolution_at`. | A |
| **FactionDispute** | Internal party dispute routed to PPDT: `dispute_id` (UUID7), `party_id`, `complainant_id_hash`, `dispute_category` (`LEADERSHIP / MEMBERSHIP / FINANCE / NOMINATION`), `description_hash`, `filed_at`, `ppdt_ref`, `status` (`FILED / PPDT_ACKNOWLEDGED / DETERMINATION_ISSUED / APPEALED / CLOSED`), `determination_summary_hash`. | B |
| **AnnualReturn** | Annual statutory filing: `return_id` (UUID7), `party_id`, `year`, `membership_count`, `audited_accounts_s3_key`, `governance_report_s3_key`, `filed_at`, `accepted_at`, `compliance_delta` (score change from this return). | B |

## 3. Registration and renewal lifecycle

| Stage | Actor | Gate |
|---|---|---|
| Application | Party founder / secretary | Name uniqueness check; symbol + colour not already taken (M67) |
| RPP review | RPP officer | Physical verification of membership (10,000 members across >24 counties per PPA §9) |
| Registration | RPP (Commissioner) | HSM-signed certificate; `status -> ACTIVE` |
| Annual return | Party secretary (December deadline) | Audited accounts + governance report required |
| Compliance scoring | System | Score computed from: returns filed on time (40%), membership (30%), governance (20%), disputes resolved (10%) |
| Suspension | RPP officer | `compliance_score < 40` for 2 consecutive years; or court order |
| Deregistration | RPP (Commissioner) | PPA §34; HSM-signed gazette notice |

Compliance scoring is a weighted rule engine (not ML); weights are configurable in the `governance-config` table without a code change.

## 4. Pipeline

```
party files application via portal
      -> PoliticalParty created (status = PENDING)
      -> name + symbol uniqueness checked via M67 API
      -> RPP officer reviews membership evidence + county spread
      -> Commissioner HSM-signs registration -> status -> ACTIVE
      -> emit eel.crms.party_registered.v1
      -> annual return cycle (December):
            -> AnnualReturn submitted by party secretary
            -> accounts + governance report uploaded to S3 (WORM)
            -> RPP officer accepts/rejects
            -> compliance_score recomputed (weighted rule engine)
            -> if compliance_score < 40: compliance_warning event
            -> if 2nd consecutive year < 40: suspension trigger -> officer review
      -> coalition registration (pre-election window):
            -> CoalitionRegistration submitted; all member parties must be ACTIVE
            -> RPP officer registers; emit eel.crms.coalition_registered.v1
            -> on dissolution: emit eel.crms.coalition_dissolved.v1
      -> faction dispute:
            -> FactionDispute filed via portal (OIDC-authenticated complainant)
            -> PPDT integration: dispute submitted via integrations/ppdt-api/
            -> ppdt_ref stored; status -> PPDT_ACKNOWLEDGED
            -> on determination: determination_summary_hash stored; status -> DETERMINATION_ISSUED
      -> deregistration: Commissioner HSM-sign; emit eel.crms.party_deregistered.v1
         -> M12 candidates with this party_id flagged PARTY_DEREGISTERED
         -> M46 opt-ins for this party revoked
```

## 5. API surface

Contract: `packages/openapi-contracts/crms/party-registration.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/parties` | Party founder (OIDC) | Register new party application |
| `GET /v1/parties` | Public (60/hr) | List ACTIVE parties (name, abbreviation, status only) |
| `GET /v1/parties/{party_id}` | Auth user | Party detail + compliance score |
| `POST /v1/parties/{party_id}/annual-return` | Party secretary (OIDC) | File AnnualReturn |
| `PATCH /v1/parties/{party_id}/annual-return/{return_id}` | RPP officer | Accept / reject return |
| `POST /v1/coalitions` | Party representative (OIDC) | Register coalition |
| `DELETE /v1/coalitions/{coalition_id}` | Party representative or RPP officer | Dissolve coalition |
| `POST /v1/disputes` | Party member (OIDC) | File FactionDispute |
| `PATCH /v1/disputes/{dispute_id}/determination` | RPP officer | Record PPDT determination |
| `POST /v1/parties/{party_id}/suspend` | RPP officer + Commissioner | Suspend party (HSM required) |
| `POST /v1/parties/{party_id}/deregister` | Commissioner + HSM | Deregister party |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Party registration portal (Astro) | `services/crms-svc/astro/party-registration` | Party secretary submits application, annual returns, disputes |
| RPP back-office console (LiveView) | `services/crms-svc/ex/rpp_web` | RPP officers manage party lifecycle, compliance scoring |
| Commissioner panel (Django) | `services/crms-svc/django/governance-admin` (shared) | HSM-sign registration + deregistration gazette notices |
| Public party register (Astro) | `services/crms-svc/astro/party-registration` | Public views registered parties + compliance scores |

Languages: EN + SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Merkle-anchored status changes** - every `PoliticalParty.status` change emits a signed event anchored in P05 via the outbox. A court can request the inclusion proof for a specific status-change event.
- **HSM-signed gazette notices** - registration and deregistration certificates carry Commissioner HSM signatures. The certificate PDF is stored in S3 WORM.
- **Compliance score immutability** - `compliance_score` is recomputed and stored as a point-in-time value per `AnnualReturn`; the historical scores are not overwritten. This provides an auditable compliance trajectory.
- **PPDT integration idempotency** - dispute submissions to PPDT (`integrations/ppdt-api/`) use the `dispute_id` as the idempotency key. Duplicate submissions are rejected by the adapter.
- **Coalition member ACTIVE gate** - server-side: coalition formation requires all member party IDs to resolve to `status = ACTIVE` at the time of registration. A party that becomes SUSPENDED after coalition formation does not automatically dissolve the coalition; the RPP officer is notified.

## 8. Kenya statute alignment

- **Political Parties Act 2011 §9** - minimum 10,000 registered members in >24 counties; M45 records and stores the membership evidence.
- **Political Parties Act 2011 §34** - grounds and procedure for deregistration; M45 workflow operationalises this with Commissioner HSM-signed gazette notice.
- **Political Parties Act 2011 §40** - annual return requirement; M45 `AnnualReturn` aggregate implements this.
- **Elections Act §22** - only registered parties may field candidates; M12 enforces this via FK to M45 `PoliticalParty.party_id` with `status = ACTIVE` gate.
- **Kenya Data Protection Act §26** - membership lists are T2 data; only member counts are exposed via the public API.

## 9. Honest caveats + external prerequisites

- PPDT API integration (`integrations/ppdt-api/`) requires the PPDT to expose a machine-readable API. As of the spec date, the PPDT uses a manual submission process. The adapter is written to a documented schema; if the PPDT does not provision an API, the adapter falls back to generating a structured email submission.
- Physical membership verification (24-county spread) is a manual RPP officer judgment. The system records the evidence documents but cannot independently verify the physical membership rolls.
- Symbol + colour uniqueness is enforced via the M67 symbol registry. M67 must be available for party registration to proceed. If M67 is unavailable, the registration step is blocked (hard dependency).
- Compliance scoring weights are configurable but changes must be gazette-noticed to ensure transparency. The config table has an audit log but no automatic gazette submission.

## 10. Test coverage + operational runbooks

Tests under `services/crms-svc/tests/`: registration lifecycle integration tests via `testcontainers`, compliance score arithmetic property-based tests via `proptest`, PPDT adapter mock via `pytest-httpserver`.

Runbooks under `docs/runbooks/`:

- `crms-svc.md` - deploy, rollback, compliance score recompute.
- `party-suspension-protocol.md` - two-consecutive-year suspension procedure + party notification.
- `coalition-dissolution.md` - coalition dissolution + M12 candidate status update.

Related documents: spec at `docs/plans/modules/party-registration-management.md`; M46 at `docs/features/party-primaries-oversight.md`; capability catalogue at `services/crms-svc/docs/CAPABILITIES.md`.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*

**Performance targets**: All read endpoints respond within 200ms at p95 under 100 concurrent users. Write endpoints (including HSM-signing paths) respond within 1,000ms at p95. Bulk operations (roster upload, CSV import) are async with a job status poll endpoint.

**SLA targets**: All API reads respond within 200ms p95. Event processing latency from trigger to state update is under 5 seconds under normal load. HSM-signing operations complete within 3 seconds.
