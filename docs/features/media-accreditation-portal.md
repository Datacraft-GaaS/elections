# Media accreditation portal (M42)

Module M42 (`media-accreditation-portal`) owned by `training-svc`. Accredits journalists and broadcasters for polling-station access; issues tamper-evident QR-coded digital badges; integrates with the Media Council of Kenya (MCK) membership API for journalist verification; refreshes the Certificate Revocation List (CRL) every 5 minutes; and shares CRL infrastructure and the badge-scanner Flutter app with M43 (observer accreditation).

## Contents

1. What M42 does (vs M43 observer accreditation)
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

## 1. What M42 does (vs M43 observer accreditation)

- **M43** = accredits civil society, domestic, and international observers.
- **M42** = accredits media organisations and individual journalists / broadcast crews. The two modules share the badge-scanner Flutter app and the CRL refresh infrastructure but maintain separate aggregates and separate MCK vs. observer-body integrations.

A media organisation must be registered before individual journalists can be accredited under it. Individual badge verification at a polling station uses the shared CRL, so a revoked badge is rejected within 5 minutes of revocation on any scanner.

## 2. Data plane

Four aggregates (per `services/training-svc/crates/training-api/src/models/media_accreditation.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **MediaOrganisation** | Registered media house: `org_id` (UUID7), `name`, `mck_membership_id`, `org_type` (`PRINT / BROADCAST / ONLINE / AGENCY`), `status` (`PENDING / VERIFIED / SUSPENDED / DEREGISTERED`), `mck_verified_at`, `election_cycle`. | B |
| **MediaAccreditation** | Individual accreditation: `accreditation_id` (UUID7), `org_id`, `journalist_id_hash` (national ID hash), `name_hash`, `badge_qr_token` (UUID7, signed), `access_zones[]` (`STATION / TALLYING_CENTRE / NATIONAL_TALLY`), `issued_at`, `valid_until`, `status` (`ACTIVE / SUSPENDED / REVOKED`). | A |
| **BadgeRevocationEntry** | CRL entry: `entry_id` (UUID7), `accreditation_id`, `revoked_at`, `reason` (`MISCONDUCT / LOST / SECURITY / ORGANISATION_SUSPENDED`), `revoked_by`. | A |
| **AccreditationAuditEntry** | Audit log of badge scans: `audit_id` (UUID7), `accreditation_id`, `station_code`, `scan_timestamp`, `outcome` (`VALID / REVOKED / EXPIRED / ZONE_MISMATCH`), `scanner_device_id`. | A |

## 3. Accreditation lifecycle

| Stage | Actor | Gate |
|---|---|---|
| Organisation registration | Media house submits application | MCK membership ID provided |
| MCK verification | System (MCK API) | `mck_membership_id` resolves to active MCK member |
| Individual journalist application | Journalist (OIDC) or org desk | Org must be `VERIFIED` |
| Commission review | Accreditation officer | Manual check for NPS clearance advisory |
| Badge issuance | System | Commission officer approves; `MediaAccreditation.status = ACTIVE` |
| Badge scan at station | Scanner app | CRL check + zone check + expiry check + QR signature verify |
| Revocation | Commission officer | Immediately added to CRL; badge invalid within 5 min |

## 4. Pipeline

```
media org applies via portal
      -> MediaOrganisation created (status = PENDING)
      -> MCK API query (integrations/mck-api/):
            -> if MCK member active: status -> VERIFIED; mck_verified_at set
            -> if not: status stays PENDING; officer notified for manual review
      -> journalist applies under verified org
            -> MediaAccreditation created (status = ACTIVE pending officer approval)
            -> officer approves: badge_qr_token generated (UUID7, HMAC-SHA256 signed)
            -> status -> ACTIVE; badge PDF generated (S3 presigned)
            -> email + SMS notification to journalist
      -> election day: journalist presents badge at station
            -> scanner app reads QR: badge_qr_token extracted
            -> HMAC signature verified (offline capable; key pre-loaded)
            -> CRL checked (local cache < 5 min old; else fetch from API)
            -> access_zones[] checked vs station type
            -> AccreditationAuditEntry recorded
      -> on revocation:
            -> BadgeRevocationEntry created
            -> accreditation status -> REVOKED
            -> CRL refresh pushed via SSE to all online scanners
            -> offline scanner will pick up on next 5-min CRL pull
      -> post-election: all accreditations expire at valid_until (election day + 2 days)
```

## 5. API surface

Contract: `packages/openapi-contracts/training/training-accreditation.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/media/organisations` | Media org desk | Register MediaOrganisation |
| `GET /v1/media/organisations/{org_id}` | Org desk or officer | Org status + MCK verification |
| `POST /v1/media/accreditations` | Journalist (OIDC) or org desk | Apply for individual accreditation |
| `GET /v1/media/accreditations/{accreditation_id}` | Journalist (OIDC) or officer | Accreditation detail |
| `POST /v1/media/accreditations/{accreditation_id}/approve` | Accreditation officer | Approve + issue badge |
| `POST /v1/media/accreditations/{accreditation_id}/revoke` | Accreditation officer | Revoke; add to CRL |
| `GET /v1/media/crl` | Scanner app (device SVID) | Fetch CRL (JSON list of revoked tokens); ETag caching |
| `GET /v1/media/verify/{badge_qr_token}` | Scanner app (device SVID) | Online badge verification (300ms SLA) |
| `GET /v1/media/audit` | Commission admin | Badge scan audit log; filter by station |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Media accreditation portal (Astro) | `services/training-svc/astro/media-accreditation` | Org registration + journalist application + badge download |
| Accreditation officer console (LiveView) | `services/training-svc/ex/accreditation_web` | Officer reviews applications; approves or rejects; revokes |
| Badge scanner app (Flutter, shared with M43) | `services/training-svc/apps/badge-scanner` | Scan + verify badges at stations; offline CRL cache |
| Commission admin panel (Django) | `services/training-svc/django/accreditation_admin` | Aggregate reporting; CRL status monitoring |

Languages: EN + SW on portal and console. Badge PDF is bilingual EN/SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **HMAC-SHA256 badge QR token** - `badge_qr_token` is signed with the election-cycle HMAC key (derived from `eel_kek_generations`). Offline scanners pre-load the key; verification does not require a network call. The key is rotated between election cycles.
- **CRL 5-minute TTL** - scanner app caches the CRL with ETag-based conditional GET. On cache miss or TTL expiry, the scanner fetches the full CRL. A revoked badge is invalid within 5 minutes on any online scanner; an offline scanner reflects revocation on its next CRL pull.
- **Zone enforcement** - `access_zones[]` is encoded in the badge QR payload (not just the DB). The scanner verifies the zone claim against the payload signature; it does not trust the displayed zone label alone.
- **Audit log immutability** - `AccreditationAuditEntry` rows are insert-only. A scan that cannot be logged (DB unreachable) is queued locally in the scanner app and flushed on reconnect.
- **NPS clearance** - the Commission's standard procedure is to cross-check accreditation applications with a National Police Service advisory list. The system provides an `nps_clearance_advisory` flag on the officer review UI; the NPS API integration is advisory only, not a hard gate.

## 8. Kenya statute alignment

- **Elections Act §44(3)** - Commission may accredit media to observe elections; M42 operationalises the accreditation function.
- **Media Council of Kenya Act 2013** - MCK is the statutory body for journalist accreditation; M42 integrates MCK membership as the primary journalist identity verification.
- **Kenya Data Protection Act §26 (minimisation)** - only ID hash and name hash stored in `MediaAccreditation`; raw PII (name, ID number) stored in `journalist_pii` T2 store with access restricted to accreditation officers.
- **Elections (Technology) Regulations 2017** - digital accreditation credentials must be verifiable and revocable; M42's HMAC QR + CRL satisfies this.

## 9. Honest caveats + external prerequisites

- MCK API integration (`integrations/mck-api/`) requires an MCK API key provisioned under an IEBC service account. The adapter is ready; the live key is an external procurement dependency.
- NPS clearance is advisory, not a hard gate. IEBC policy may change this; the current implementation reflects the advisory-only position.
- Badge scanner app requires Android 8.0+ or iOS 14+. QR scanning uses the device camera; low-light performance at polling stations depends on device hardware.
- Offline CRL operation means a device that is offline for more than 5 minutes may briefly accept a badge that has been revoked in the interim. The 5-minute window is a deliberate trade-off between field usability and revocation latency; it is documented in the threat model at `services/training-svc/docs/THREAT-MODEL.md`.
- CRL size scales with the number of accredited journalists. For 10,000 journalists, the CRL JSON is approximately 500 KB; this is within mobile data budgets on 3G. Above 50,000 accreditations, CRL delta compression (RFC 5280 delta-CRL) should be evaluated.

## 10. Test coverage + operational runbooks

Tests under `services/training-svc/tests/`: MCK API mock via `pytest-httpserver`, badge QR signing + verification round-trip, CRL ETag cache behaviour, offline mode simulation.

Runbooks under `docs/runbooks/`:

- `training-svc.md` - media accreditation module deploy + rollback.
- `badge-revocation-emergency.md` - bulk revocation on org suspension.
- `crl-staleness.md` - response if CRL endpoint is unreachable for >5 minutes.

Related documents: spec at `docs/plans/modules/media-accreditation-portal.md`; M43 at `docs/features/observer-accreditation-portal.md`; capability catalogue at `services/training-svc/docs/CAPABILITIES.md`.
