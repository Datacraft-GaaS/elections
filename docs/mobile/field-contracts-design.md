# Field Contracts Design (M1.2) — `packages/openapi-contracts/field/`

Status: design of record for the five Wave-M1 field contracts, authored
2026-09-04 from the module spec API tables (`docs/plans/modules/mobile-*.md`).
The YAML files are generated mechanically from this document; any divergence
between this file and the YAMLs is a bug — fix the YAML.

Files to create (paths exactly as the module specs mandate):

| File | App / Module | Paths | Tier | Auth model |
|---|---|---|---|---|
| `field/poll-clerk.yaml` | M31 poll-clerk | 9 × `/v1/clerk/*` | 2 | JWT+SpiceDB (DPO/CLERK/KIOSK_OPERATOR) |
| `field/party-agent.yaml` | M30 party-agent | 10 × `/v1/agent/*` | 2 | JWT+SpiceDB (PARTY_AGENT/PARTY_CHIEF_AGENT) |
| `field/observer.yaml` | M34 observer | 11 × `/v1/observer/*` | 3 (bulk=4) | JWT+SpiceDB (OBSERVER/OBSERVER_CHIEF) |
| `field/field-support.yaml` | M32 field-support | 10 × `/v1/support/*` | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) |
| `field/citizen-verify.yaml` | M29 citizen-verify | 12 × `/v1/citizen/*` | 1/2/4 per table | public / JWT(OTP) / JWT+captcha |

---

## 1. Conventions (applied to every operation in all five files)

House style = `field-mobile-bff/bff.yaml`; Spectral ruleset =
`tools/spectral/.spectral.yaml` (enforced via `just lint`). New files are the
strictest reading of both:

1. **OpenAPI 3.1.0, self-contained** — local `components` per file, no
   cross-file `$ref`s (house pattern; `_shared/` refs are unused by any
   existing field contract).
2. **Paths verbatim from the module spec tables.** Bare `/v1/<role>/*`, *not*
   `/api/v1/`. Documented deviation from Spectral rule `eel-path-prefix`: 71
   existing paths already use bare `/v1/` (incl. `field-mobile-bff/openapi.yaml`),
   and plan.md §4.2 mandates these exact path strings. Not a new violation class.
3. **Every operation carries:**
   - `x-eel-tier: <int>` from the module table column;
   - `x-eel-auth:` ∈ {`public`, `JWT`, `JWT+SpiceDB`} (Spectral enum);
   - `x-eel-sla: {p50, p95, p99}` in ms (reads ≤ 300 p99; writes ≤ 700 p99;
     bulk/rollup reads ≤ 2000 p99).
4. **Every mutating operation** declares an inline required header parameter
   (inline, not `$ref` — Spectral matches on `name/in/required`):
   ```yaml
   - name: Idempotency-Key
     in: header
     required: true
     description: Client-generated UUIDv7 for idempotent request deduplication (TTL 72 h)
     schema: {type: string, format: uuid, x-eel-data-class: E}
   ```
5. **Every staff-role operation** (all of M30/M31/M32/M34) also declares the
   `X-Eel-Attest` header (Play Integrity / App Attest token, ADR-0021),
   `minLength: 32`, data class E. Citizen-verify public endpoints do not.
6. **Error responses** are `$ref`s to local problem+json response components
   (400/401/403/404/409/422/429 as applicable per endpoint).
7. **Tier-2 success responses** (200/201/202) carry both `application/json` and
   `application/cbor` with the same schema (Spectral tier-2 content-negotiation
   rule). Tier 3/4: JSON only.
8. **Tier-1 GET 200 responses** carry integrity headers (Spectral tier-1 rule):
   `X-Eel-Signature`, `X-Eel-Merkle-Proof`, `X-Eel-Snapshot-Id`.
9. **Data classes** (`x-eel-data-class` on every schema property):
   - B — voter identity hashes (`id_hash`, biometric hashes)
   - C — station/constituency codes, GPS, results data
   - D — staff identifiers (officer/agent/engineer UUIDs), free-text notes
   - E — device/app/technical metadata (attest tokens, idempotency keys, device hashes)
10. **No `x-handler-bound`** on any path — contract-first; handlers land in
    M1/M2. Each unbound path gets a `.completeness-allowlist.yaml` row
    (`<file>:<line>:openapi_unbound_path`, reason "contract-first spec; handler
    pending M1/M2 (docs/plans/deferral-plan.md)", expiry `2026-12-31`) — the
    training-medical precedent. Line = the `  /path:` line number.
11. **No fabricated endpoints or fields.** Schemas below are derived from the
    module spec notes; where a spec is silent, the field set is minimal and
    generic (UUIDv7 ids, RFC3339 timestamps, `object` with
    `additionalProperties: true` for opaque payloads).

## 2. Shared components (identical block in every file)

```yaml
components:
  securitySchemes:
    bearerAuth: {type: http, scheme: bearer, bearerFormat: JWT}
  schemas:
    ProblemDetail:            # RFC 9457; type/title/status required
      {type: object, required: [type, title, status], properties:
        {type: {type: string, format: uri}, title: {type: string},
         status: {type: integer}, detail: {type: string},
         instance: {type: string, format: uri}, trace_id: {type: string}}}
  responses:                  # all application/problem+json → ProblemDetail
    BadRequest:        400 · validation failed
    Unauthorized:      401 · missing/invalid JWT or attestation
    Forbidden:         403 · SpiceDB role denied
    NotFound:          404 · resource not found
    Conflict:          409 · idempotency collision or state conflict
    UnprocessableEntity: 422 · semantic validation error
    TooManyRequests:   429 · rate limit (policy bundle) exceeded
```

Integrity header components (tier-1 files only): `X-Eel-Signature` (cosign
signature over response body), `X-Eel-Merkle-Proof`, `X-Eel-Snapshot-Id` (uuid).

---

## 3. poll-clerk.yaml (M31) — 9 paths, tier 2

| # | Op | Path | Notes → schema driver |
|---|---|---|---|
| 1 | POST | `/v1/clerk/check-in` | biometric + attestation check-in; 409 (shift already open), 429 |
| 2 | GET | `/v1/clerk/station/{code}/dashboard` | no-PII kit dashboard mirror from BVI; 404 |
| 3 | POST | `/v1/clerk/voter-lookup` | DPO attestation hash + justification; rate-limited; audited; 429 |
| 4 | POST | `/v1/clerk/incidents` | structured incident (shared shape, §5) |
| 5 | POST | `/v1/clerk/queue` | add/next/priority; priority needs audited reason; 409 |
| 6 | GET | `/v1/clerk/queue/{station_code}` | queue status, no identities; 404 |
| 7 | POST | `/v1/clerk/end-of-day` | Ed25519-signed on device; verified server-side |
| 8 | GET | `/v1/clerk/kiosk/register` | BVV lookup, kiosk mode, staff-assisted OTP; query `id_hash`(B), `station_code`(C); 429 |
| 9 | POST | `/v1/clerk/kiosk/objection` | objection on citizen's behalf, consent attested |

Schemas (`!` = required):

- `CheckInRequest { station_code!: str≤32(C), officer_id!: uuid(D), biometric_hash!: str64hex(B), gps!: Gps(C), device_id_hash: str64hex(E) }`
- `CheckInResponse { check_in_id!: uuid, station_code!(C), shift_status!: enum[OPEN](D), checked_in_at!: rfc3339 }`
- `StationDashboard { station_code!(C), turnout_count!: int, voters_expected: int, queue_depth!: int, kit_battery_level: f32 0..1, connectivity_status!: enum[online,degraded,offline](D), forms_synced!: bool, incidents_open!: int, updated_at!: rfc3339 }`
- `VoterLookupRequest { id_hash!: str64hex(B), station_code!(C), dpo_attestation_hash!: str64hex(D), justification!: str≤512(D) }`
- `VoterLookupResponse { registered!: bool, polling_station_name: str(C), ward: str(C), constituency: str(C), lookup_audit_ref!: uuid(E) }` — never raw IDs/names (bff.yaml precedent)
- `QueueManageRequest { station_code!(C), action!: enum[add,next,priority](D), position_token: str(E), priority_reason: str≤256(D, required iff action=priority) }`
- `QueueManageResponse { station_code!(C), queue_depth!: int, position: int, applied_at!: rfc3339 }`
- `QueueStatus { station_code!(C), queue_depth!: int, waiting!: int, served_today!: int, priority_waiters!: int, updated_at!: rfc3339 }`
- `EndOfDayRequest { station_code!(C), officer_id!: uuid(D), report!: EodReport, signature!: str(base64 Ed25519)(E), hlc: str(E) }`
- `EodReport { voters_served!: int, incidents_count!: int, forms_submitted!: int, anomalies: object }`
- `EndOfDayResponse { eod_id!: uuid, station_code!(C), status!: enum[ACCEPTED](D), submitted_at!: rfc3339 }`
- `KioskRegisterResponse { registered!: bool, polling_station_name: str(C), polling_station_code: str(C), ward: str(C), needs_otp!: bool, otp_ref: uuid(E) }`
- `KioskObjectionRequest { station_code!(C), id_hash!: str64hex(B), objection_type!: enum[rectification,transfer,removal](D), consent_attested!: bool(D), consent_evidence_ref: uuid(E), signature!: str(E) }`
- `KioskObjectionResponse { objection_id!: uuid, status!: enum[FILED](D), created_at!: rfc3339 }`

## 4. party-agent.yaml (M30) — 10 paths, tier 2

| # | Op | Path | Notes → schema driver |
|---|---|---|---|
| 1 | POST | `/v1/agent/check-in` | GPS-verified; notifies BVI back-end; 409, 429 |
| 2 | GET | `/v1/agent/station/{code}/dashboard` | aggregate, no PII; 404 |
| 3 | GET | `/v1/agent/forms/34a/{id}` | signed Form 34A + Merkle proof; 404 |
| 4 | POST | `/v1/agent/forms/34a/{id}/countersign` | attach countersignature; 409 |
| 5 | POST | `/v1/agent/forms/34a/{id}/dispute` | mandatory reason; escalates to war room; 409 |
| 6 | GET | `/v1/agent/rollup/34b/{constituency}` | chief agent rollup; 404 |
| 7 | GET | `/v1/agent/rollup/34c` | chief agent national rollup; query `election_id`(E); 404 |
| 8 | POST | `/v1/agent/rollup/34b/{constituency}/dispute` | dispute a rollup; 409 |
| 9 | POST | `/v1/agent/incidents` | shared incident shape (§5) |
| 10 | GET | `/v1/agent/incidents/{id}` | incident status; 404 |

Schemas:

- `AgentCheckInRequest { station_code!(C), agent_id!: uuid(D), party_code!: str≤16(D), gps!: Gps(C), device_id_hash: str64hex(E) }`
- `AgentCheckInResponse { check_in_id!: uuid, checked_in_at!: rfc3339, bvi_notified!: bool(E) }`
- `AgentStationDashboard { station_code!(C), voters_processed!: int, incident_count!: int, status!: enum[OPEN,CLOSED,ADJOURNED](D), form_34a_status: enum[DRAFT,COUNTERSIGNED,DISPUTED](D), updated_at!: rfc3339 }`
- `Form34ADocument { id!: uuid, station_code!(C), presiding_officer_id!: uuid(D), created_at!: rfc3339, status!: enum[DRAFT,COUNTERSIGNED,DISPUTED](D), data: object, signature!: str(E), merkle_proof: [str64hex](E), snapshot_id: uuid(E) }`
- `CountersignRequest { agent_id!: uuid(D), signature!: str(base64 Ed25519)(E) }`
- `CountersignResponse { id!: uuid, status!: enum[COUNTERSIGNED](D), countersigned_at!: rfc3339 }`
- `DisputeRequest { agent_id!: uuid(D), reason!: str≥8≤1024(D), evidence_refs: [uuid](E) }`
- `DisputeResponse { id!: uuid, status!: enum[DISPUTED](D), escalated_to!: enum[war_room](D), disputed_at!: rfc3339 }`
- `Rollup34B { constituency_code!(C), station_count!: int, totals!: object, signature!: str(E), generated_at!: rfc3339 }`
- `Rollup34C { election_id!: uuid(E), county_count!: int, totals!: object, signature!: str(E), generated_at!: rfc3339 }`
- `IncidentStatus { incident_id!: uuid, status!: enum[OPEN,IN_REVIEW,RESOLVED,DISMISSED](D), created_at!: rfc3339, updated_at!: rfc3339 }`

## 5. Shared incident shape (M30/M31/M34 use identical schemas)

- `IncidentRequest { station_code!(C), category!: enum[SECURITY,PROCEDURAL,EQUIPMENT,OTHER](D), severity!: enum[LOW,MEDIUM,HIGH,CRITICAL](D), description!: str≤2048(D), gps: Gps(C), evidence_refs: [uuid](E) }`
- `IncidentResponse { incident_id!: uuid, status!: enum[OPEN](D), created_at!: rfc3339 }`
- `Gps { lat!: f64, lon!: f64 }` (class C)

## 6. observer.yaml (M34) — 11 paths, tier 3 (bulk = tier 4)

| # | Op | Path | Tier | Notes → schema driver |
|---|---|---|---|---|
| 1 | GET | `/v1/observer/portal/results/{station_code}` | 3 | per-station results + signed Form 34A image ref; 404 |
| 2 | GET | `/v1/observer/portal/results/{station_code}/proof` | 3 | Merkle-inclusion proof → `MerkleProof`; 404 |
| 3 | GET | `/v1/observer/portal/results/bulk/{constituency}` | 4 | rate-limited bulk → `BulkResults`; 429, 404 |
| 4 | POST | `/v1/observer/stations/{code}/visit` | 3 | GPS check-in; 409 |
| 5 | POST | `/v1/observer/stations/{code}/visit/sign-off` | 3 | sign off on leaving; 409 |
| 6 | POST | `/v1/observer/stations/{code}/verify` | 3 | proof verified client-side |
| 7 | POST | `/v1/observer/stations/{code}/verify/countersign` | 3 | category-gated; 403, 409 |
| 8 | POST | `/v1/observer/incidents` | 3 | shared incident shape (§5) |
| 9 | POST | `/v1/observer/incidents/{id}/systemic` | 3 | OBSERVER_CHIEF; immediate IEBC response; 403, 409 |
| 10 | GET | `/v1/observer/end-of-mission` | 3 | report data → `EndOfMissionReport` |
| 11 | GET | `/v1/observer/end-of-mission/pdf` | 3 | 200 content `application/pdf` (binary) |

Schemas:

- `StationResults { station_code!(C), results!: object, form_34a_image_ref!: str(uri)(E), signature!: str(E), generated_at!: rfc3339 }`
- `MerkleProof { station_code!(C), merkle_root!: str64hex(E), leaf_index!: int(E), proof!: [str64hex](E) }` — client verifies; server never asserts
- `BulkResults { constituency_code!(C), stations!: [StationResults], generated_at!: rfc3339 }`
- `VisitCheckInRequest { observer_id!: uuid(D), gps!: Gps(C) }`
- `VisitCheckInResponse { visit_id!: uuid, checked_in_at!: rfc3339 }`
- `VisitSignOffRequest { visit_id!: uuid, observer_id!: uuid(D), signature!: str(E) }`
- `VisitSignOffResponse { visit_id!: uuid, signed_off_at!: rfc3339, duration_seconds!: int }`
- `VerifyRecordRequest { observer_id!(D), station_code!(C), merkle_root_verified!: str64hex(E), leaf_index!: int(E), proof_verified!: bool(E), findings: object(D), signature!: str(E) }`
- `VerifyRecordResponse { verification_id!: uuid, created_at!: rfc3339 }`
- `SystemicFlagRequest { observer_id!: uuid(D), reason!: str≥8≤1024(D) }`
- `SystemicFlagResponse { incident_id!: uuid, flagged_systemic!: true(D), escalated_at!: rfc3339 }`
- `EndOfMissionReport { mission_id!: uuid, stations_visited!: [str(C)], verifications_count!: int, incidents_count!: int, report!: object(D), generated_at!: rfc3339 }`

## 7. field-support.yaml (M32) — 10 paths, tier 2

| # | Op | Path | Notes → schema driver |
|---|---|---|---|
| 1 | GET | `/v1/support/tickets` | query: `cluster_code`(C), `severity`(D), `status`(D) |
| 2 | POST | `/v1/support/tickets/{id}/take` | take/assign; 409 (already taken) |
| 3 | POST | `/v1/support/tickets/{id}/escalate` | L2/L3/Mechrevo/IEBC ICT; 409 |
| 4 | GET | `/v1/support/kits/{serial}` | kit health telemetry; 404 |
| 5 | POST | `/v1/support/kits/{serial}/action` | signed, audited, kit-consent required; 202; 409 |
| 6 | POST | `/v1/support/warranty` | warranty claim |
| 7 | GET | `/v1/support/spares` | query: `cluster_code`(C), `part_type`(D) |
| 8 | GET | `/v1/support/oncall` | query: `from`, `to` (date) |
| 9 | POST | `/v1/support/handoff` | shift handoff, sign-off required; 409 |
| 10 | POST | `/v1/support/shift-report` | signed end-of-shift |

Schemas:

- `Ticket { ticket_id!: uuid, kit_serial!: str≤64(E), cluster_code!(C), severity!: enum[LOW,MEDIUM,HIGH,CRITICAL](D), status!: enum[OPEN,TAKEN,ESCALATED,CLOSED](D), created_at!: rfc3339, sla_deadline: rfc3339 }`
- `TicketList { tickets!: [Ticket], total!: int }`
- `TakeTicketRequest { engineer_id!: uuid(D), note: str≤512(D) }`
- `TakeTicketResponse { ticket_id!: uuid, assigned_to!: uuid(D), taken_at!: rfc3339 }`
- `EscalateRequest { engineer_id!(D), target_level!: enum[L2,L3,MECHREVO,IEBC_ICT](D), reason!: str≤1024(D) }`
- `EscalateResponse { ticket_id!: uuid, escalated_to!: enum[L2,L3,MECHREVO,IEBC_ICT](D), escalated_at!: rfc3339 }`
- `KitHealth { kit_serial!(E), battery_level!: f32 0..1(E), storage_used_pct!: f32 0..100(E), connectivity!: enum[online,degraded,offline](D), firmware_version: str≤32(E), last_heartbeat_at!: rfc3339, anomalies: [str](D) }`
- `KitActionRequest { engineer_id!(D), action!: enum[reboot,wipe_cache,sync_now,factory_reset](D), signature!: str(E), kit_consent_ref!: uuid(E) }`
- `KitActionResponse { action_id!: uuid, status!: enum[ACCEPTED](D), accepted_at!: rfc3339 }`
- `WarrantyClaimRequest { kit_serial!(E), issue_description!: str≤2048(D), evidence_refs: [uuid](E), engineer_id!: uuid(D) }`
- `WarrantyClaimResponse { claim_id!: uuid, status!: enum[OPEN](D), created_at!: rfc3339 }`
- `SparesList { spares!: [{part_type!(D), quantity!: int(E), location_code!(C)}], as_of!: rfc3339 }`
- `OncallSchedule { shifts!: [{date!(D), engineer_id!(uuid D), role!(str D), contact_ref: str(D)}] }`
- `HandoffRequest { outgoing_engineer_id!(D), incoming_engineer_id!(D), open_tickets!: [uuid](E), notes: str≤2048(D), signature!: str(E) }`
- `HandoffResponse { handoff_id!: uuid, created_at!: rfc3339 }`
- `ShiftReportRequest { engineer_id!(D), shift!: {started_at!: rfc3339, ended_at!: rfc3339}, tickets_closed!: int, summary: str≤2048(D), signature!: str(E) }`
- `ShiftReportResponse { report_id!: uuid, submitted_at!: rfc3339 }`

## 8. citizen-verify.yaml (M29) — 12 paths, tier 1/2/4

Auth note: the module table says "JWT or public + captcha" for
`POST /v1/citizen/reports`. Spectral's `x-eel-auth` enum has no composite, so
the operation declares `JWT` (stronger default) and the description documents
the public+captcha fallback channel.

| # | Op | Path | Tier | Auth | Notes → schema driver |
|---|---|---|---|---|---|
| 1 | GET | `/v1/citizen/register` | 2 | JWT (OTP-verified) | query `id_hash`(B); rate 20/day/session; 429 |
| 2 | POST | `/v1/citizen/register/objection` | 2 | JWT (OTP-verified) | rectification/transfer/removal via BVR |
| 3 | GET | `/v1/citizen/polling-station` | 1 | public | query `id_hash`(B); cached; integrity headers |
| 4 | GET | `/v1/citizen/results/{station_code}` | 1 | public | + signed Form 34A image ref; integrity headers |
| 5 | GET | `/v1/citizen/results/{station_code}/proof` | 1 | public | `MerkleProof` (§6); integrity headers |
| 6 | GET | `/v1/citizen/results/bulk/{constituency_code}` | 4 | public | rate-limited; 429 |
| 7 | GET | `/v1/citizen/candidates` | 1 | public | query `election_id`(E), `position`(D), `area_code`(C); integrity headers |
| 8 | GET | `/v1/citizen/civic` | 1 | public | signed content bundle; integrity headers |
| 9 | POST | `/v1/citizen/reports` | 2 | JWT (public+captcha fallback) | optional evidence refs |
| 10 | GET | `/v1/citizen/reports/{id}` | 2 | JWT (own reports) | status only; 403 if not owner |
| 11 | POST | `/v1/citizen/notifications/subscribe` | 2 | JWT | no PII in payloads (FR-M30-025 pattern) |
| 12 | GET | `/v1/citizen/app/verify` | 1 | public | signing-key fingerprint + build info; integrity headers |

Schemas:

- `RegisterLookupResponse { registered!: bool, polling_station_name: str(C), polling_station_code: str(C), ward: str(C), constituency: str(C), verified_at!: rfc3339 }`
- `RegisterObjectionRequest { id_hash!: str64hex(B), objection_type!: enum[rectification,transfer,removal](D), statement!: str≤2048(D), evidence_refs: [uuid](E) }`
- `RegisterObjectionResponse { objection_id!: uuid, status!: enum[FILED](D), created_at!: rfc3339 }`
- `PollingStationResponse { polling_station_name!(C), polling_station_code!(C), ward!(C), constituency!(C), address: str(C) }`
- `CitizenResults` = `StationResults` (§6, identical shape)
- `CandidateDirectory { candidates!: [{candidate_id!: uuid(E), name!: str(D), party_name: str(D), symbol_ref: str(uri)(E), position!: str(D)}] }`
- `CivicBundle { bundle_id!: uuid(E), version!: str≤32(E), content_refs!: [str(uri)](E), signature!: str(E), merkle_root!: str64hex(E) }`
- `CitizenReportRequest { category!: enum[PROCEDURAL,SECURITY,EQUIPMENT,MISINFORMATION,OTHER](D), description!: str≤4096(D), gps: Gps(C), evidence_refs: [uuid](E), captcha_token: str(E, required on public channel) }`
- `CitizenReportResponse { report_id!: uuid, status!: enum[RECEIVED](D), created_at!: rfc3339 }`
- `CitizenReportStatus { report_id!: uuid, status!: enum[RECEIVED,UNDER_REVIEW,PUBLISHED,DISMISSED](D), created_at!: rfc3339, updated_at!: rfc3339 }`
- `SubscribeRequest { platform!: enum[android,ios](E), device_token!: str≤512(E), topics: [str](E) }` — push payloads carry event type + record id only
- `SubscribeResponse { subscription_id!: uuid, created_at!: rfc3339 }`
- `AppVerifyResponse { app_signing_key_sha256!: str64hex(E), build_version!: str≤32(E), min_supported_version!: str≤32(E), attestation_required!: bool(E) }`

---

## 9. Verification (after YAML authoring)

```bash
# YAML parses
python3 -c "import yaml,sys; [yaml.safe_load(open(f)) for f in sys.argv[1:]]; print('yaml ok')" packages/openapi-contracts/field/*.yaml
# path counts match this design (9/10/11/10/12)
for f in packages/openapi-contracts/field/*.yaml; do echo "$f: $(grep -c '^  /' $f)"; done
# every operation has the three x-eel extensions
grep -c "x-eel-tier\|x-eel-auth\|x-eel-sla" packages/openapi-contracts/field/*.yaml   # = 3 × path count per file
# every POST declares Idempotency-Key inline
# (manual: one inline header param per mutating op; Spectral eel-idempotency-key-post enforces)
# completeness scanner: new unbound paths allowlisted with expiry, scan green
cargo build --release -p eel-completeness
./target/release/eel-completeness scan --fail-on-new
```

## 10. Open questions (do not block YAML authoring)

1. **Spectral `/api/v1/` prefix rule vs module-spec paths** — pre-existing repo
   inconsistency (71 bare `/v1/` paths). Options: fix the ruleset to allow a
   `x-eel-path-prefix-exempt` tag, or migrate all field contracts to `/api/v1/`.
   Decision needed before M2 apps generate clients; tracked here, not in the YAMLs.
2. **`priority_reason` conditional requirement** (queue manage) — OpenAPI 3.1
   `if/then` supported by Spectral? If not, enforce server-side + document.
3. **Tier-3 CBOR** — ruleset only mandates tier-2/5; keep tier 3/4 JSON-only
   until a client needs otherwise (kit apps are the likely future consumers).
