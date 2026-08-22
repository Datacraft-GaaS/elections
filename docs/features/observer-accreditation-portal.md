# Observer accreditation portal (M43)

Module M43 (`observer-accreditation-portal`) owned by `training-svc`. Accredits domestic and international observer organisations and their individual observers (KEOG, ELOG, EU EOM, AU, Commonwealth, Carter Center, and others); handles late-roster submissions; shares CRL infrastructure and badge-scanner Flutter app with M42 (media accreditation); and feeds M34 (mobile observer app) with observer identity tokens.

## Contents

1. What M43 does (vs M42 media accreditation and M34 observer app)
2. Data plane (schema)
3. Accreditation lifecycle
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M43 does (vs M42 media accreditation and M34 observer app)

- **M42** = media-specific accreditation with MCK integration; journalists only.
- **M43** = observer-specific accreditation for civil society organisations, domestic observer groups, and international election observation missions (IEOMs). Uses the same badge-scanner app and CRL infrastructure but separate organisation-level registration and different identity verification gates (OIDC + letter of authorisation, not MCK).
- **M34** = mobile observer app that observers use during election day to log observations. M34 consumes M43 `observer_token` (derived from `IndividualObserver.badge_qr_token`) to establish observer identity inside the app.

A late-roster scenario (observer mission adds members after the roster deadline) is handled by an escalated review flow that requires Commission Director sign-off rather than accreditation officer approval.

## 2. Data plane

Three aggregates (per `services/training-svc/crates/training-api/src/models/observer_accreditation.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **ObserverOrganisation** | Registered observer body: `org_id` (UUID7), `name`, `org_type` (`DOMESTIC_NGO / DOMESTIC_POLITICAL / INTERNATIONAL_MISSION / ACADEMIC`), `authorising_body` (for IEOMs: sending country or multilateral body), `max_roster_size`, `status` (`PENDING / VERIFIED / SUSPENDED / DEREGISTERED`), `verified_at`, `election_cycle`. | B |
| **IndividualObserver** | Individual observer credential: `observer_id` (UUID7), `org_id`, `national_id_hash` (or passport_hash for internationals), `name_hash`, `badge_qr_token` (UUID7, HMAC-SHA256 signed), `access_zones[]`, `is_late_roster` (boolean), `issued_at`, `valid_until`, `status` (`ACTIVE / SUSPENDED / REVOKED`). | A |
| **LateRosterRequest** | Late roster escalation: `request_id` (UUID7), `org_id`, `observer_ids[]` (requested additions), `justification_hash`, `requested_at`, `director_decision` (`APPROVED / REJECTED`), `director_id`, `decided_at`. | B |

CRL infrastructure and `AccreditationAuditEntry` (section 7 below) are shared with M42; they live in the same `training-svc` schema partition and are served by the same CRL endpoint with a `type` query parameter distinguishing `media` from `observer` entries.

## 3. Accreditation lifecycle

| Stage | Actor | Gate |
|---|---|---|
| Organisation registration | Observer org submits application + authorisation letter | Letter upload required for IEOMs |
| Organisation verification | Accreditation officer | Manual review; for IEOMs, Foreign Affairs ministry notification |
| Roster submission | Org desk (bulk CSV, up to 5,000 rows) | Org must be `VERIFIED`; roster deadline must not have passed |
| Individual review | System (auto-approve if org verified + quota not exceeded) | Quota = `max_roster_size` |
| Badge issuance | System | `IndividualObserver.status = ACTIVE`; badge_qr_token generated |
| Late roster escalation | Org desk | Roster deadline passed; Director sign-off required |
| Badge scan at station | Scanner app (shared with M42) | CRL + zone + expiry + signature |
| Revocation | Accreditation officer or system (org suspension) | Immediate CRL update; M34 token invalidated |

## 4. Pipeline

```
observer org registers via portal
      -> ObserverOrganisation created (status = PENDING)
      -> officer reviews; for IEOMs: foreign affairs notification step (manual)
      -> status -> VERIFIED
      -> org submits roster (bulk CSV up to 5,000 rows)
            -> each row: IndividualObserver created
            -> auto-approve if org VERIFIED + quota not exceeded
            -> badge_qr_token generated per observer (HMAC-SHA256 signed)
            -> badge PDF generated + emailed to org desk
            -> observer_token derived from badge_qr_token -> pushed to M34 identity store
      -> late roster scenario:
            -> LateRosterRequest submitted
            -> Commission Director reviews in governance-admin panel
            -> if APPROVED: observers added with is_late_roster = true
            -> if REJECTED: org notified with reason
      -> election day: observer scans badge at station
            -> same scanner app as M42; CRL shared endpoint
            -> AccreditationAuditEntry recorded (shared table)
      -> on org suspension:
            -> all org's IndividualObserver records set REVOKED
            -> bulk CRL entries created
            -> M34 tokens invalidated via eel.training.observer_revoked.v1
      -> post-election: all accreditations expire at valid_until (election day + 7 days)
```

## 5. API surface

Contract: `packages/openapi-contracts/training/training-accreditation.yaml` (shared contract; observer paths under `/v1/observers/`). All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/observers/organisations` | Org desk | Register ObserverOrganisation |
| `GET /v1/observers/organisations/{org_id}` | Org desk or officer | Org detail + verification status |
| `POST /v1/observers/organisations/{org_id}/verify` | Accreditation officer | Verify organisation |
| `POST /v1/observers/organisations/{org_id}/roster` | Org desk | Bulk CSV roster upload (up to 5,000 rows) |
| `GET /v1/observers/accreditations` | Officer | List IndividualObserver records; filter by org / status |
| `POST /v1/observers/accreditations/{observer_id}/revoke` | Accreditation officer | Revoke individual badge |
| `POST /v1/observers/late-roster` | Org desk | Submit LateRosterRequest |
| `PATCH /v1/observers/late-roster/{request_id}` | Commission Director | Approve or reject late roster |
| `GET /v1/observers/crl` | Scanner app (device SVID) | Fetch observer CRL; ETag caching; `type=observer` param |
| `GET /v1/observers/verify/{badge_qr_token}` | Scanner app or M34 (SVID) | Online badge verify (300ms SLA) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Observer accreditation portal (Astro) | `services/training-svc/astro/observer-accreditation` | Org registration + roster upload + late roster request |
| Accreditation officer console (LiveView, shared with M42) | `services/training-svc/ex/accreditation_web` | Review org applications; manage late rosters |
| Director review panel (Django) | `services/training-svc/django/accreditation_admin` | Director approves or rejects late roster requests |
| Badge scanner app (Flutter, shared with M42) | `services/training-svc/apps/badge-scanner` | Station badge verification; offline CRL cache |

Languages: EN + SW + French + Arabic display on the portal (for international missions). Badge PDF is EN/SW bilingual. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Shared HMAC key + CRL with M42** - the HMAC signing key and CRL endpoint are shared with M42, but the QR payload includes a `type` field (`OBSERVER` vs `MEDIA`). A scanner configured for observer verification rejects a media badge with a zone-mismatch error even if the signature is valid.
- **Quota enforcement** - `max_roster_size` is enforced server-side at roster upload time. A CSV row that would exceed the quota is rejected with an error; partial batch upload is not supported (all-or-nothing).
- **Late roster audit trail** - `LateRosterRequest` records the Director's identity and decision timestamp. Late-roster observers are flagged with `is_late_roster = true` in their audit scan entries.
- **M34 token invalidation** - `eel.training.observer_revoked.v1` is consumed by M34 to invalidate the observer's in-app session within the event-processing SLA (30 seconds). M34's cached token TTL is 60 seconds, providing a worst-case 90-second invalidation window on the M34 side.
- **Bulk revocation on org suspension** - when an ObserverOrganisation is set to `SUSPENDED`, a background job creates `BadgeRevocationEntry` rows for all active observers in that org. The CRL is refreshed within the 5-minute window.

## 8. Kenya statute alignment

- **Elections Act §44(2)** - Commission shall accredit citizens and organisations to observe elections; M43 operationalises this mandate.
- **Elections (General) Regulations 2012, Reg 73** - observer accreditation requirements; M43 implements the application, verification, and badge issuance workflow prescribed.
- **Kenya Data Protection Act §26 (minimisation)** - only ID hash and name hash stored in the accreditation DB; raw PII in T2 store restricted to accreditation officers.
- **Vienna Convention on Diplomatic Relations** - international observers from diplomatic missions are subject to different identity verification (passport hash + sending country authority letter); M43 supports this via the `authorising_body` field and the manual IEOM verification flow.

## 9. Honest caveats + external prerequisites

- Domestic observer organisations must be registered under Kenya's NGO Act or Societies Act. M43 does not verify NGO/Society registration; this is a manual check by the accreditation officer. A future integration with the NGO Coordination Board is noted as a gap.
- International mission accreditation requires coordination with the Ministry of Foreign Affairs for notification (not approval). This is a manual step logged by the accreditation officer; there is no MFA API integration.
- Bulk CSV roster upload (up to 5,000 rows) is validated row-by-row; the first validation error aborts the upload. Orgs with large rosters should upload in sub-batches if they expect validation failures.
- M34 token invalidation relies on the M43 event being processed and M34 being online. An observer using M34 in offline mode may continue to have an active in-app session for up to 60 seconds after revocation.

## 10. Test coverage + operational runbooks

Tests under `services/training-svc/tests/`: bulk CSV upload + quota enforcement, late roster Director approval flow, badge QR signing + CRL verification (shared test suite with M42), M34 token invalidation event.

Runbooks under `docs/runbooks/`:

- `training-svc.md` - observer module deploy + rollback.
- `late-roster-emergency.md` - late roster approval under time pressure on election eve.
- `org-suspension-bulk-revoke.md` - procedure for suspending an org and verifying bulk CRL update.

Related documents: spec at `docs/plans/modules/observer-accreditation-portal.md`; M42 at `docs/features/media-accreditation-portal.md`; capability catalogue at `services/training-svc/docs/CAPABILITIES.md`.

## Edge cases and known gaps

- An observer organisation that withdraws after badge issuance: the Commission suspends the org; all active `IndividualObserver` records are bulk-revoked; M34 tokens are invalidated. Badges already physically distributed cannot be recalled but will be rejected by scanners within 5 minutes of the CRL update.
- An observer who loses their device with the M34 app: M34 session token revocation is independent of M43 badge revocation. Revoking the M34 token (via M34 admin) is a separate action from revoking the M43 badge; both should be done simultaneously by the accreditation officer.
- Roster CSV validation failures: a CSV with 4,999 valid rows and 1 invalid row is rejected in its entirety. The error response lists all invalid rows with reasons. This prevents partial roster uploads that could cause quota ambiguity. Orgs should validate their CSV locally before upload using the CSV validation schema at `services/training-svc/docs/roster-schema.json`.
- International observer visas: IEBC does not process visas; observers from international missions must have independent immigration clearance. The accreditation portal notes this requirement but cannot verify it. M43 records a `visa_status_note_hash` field for the officer to document the visa situation.

---

*Module documentation version: 1.0. Last updated: 2026-08-22. Review before each election cycle to verify crate paths, OpenAPI contract versions, integration endpoint URLs, and statutory rule offsets remain current.*
