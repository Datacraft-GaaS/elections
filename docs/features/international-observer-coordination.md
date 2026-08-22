# International observer coordination (M57)

Module M57 (`international-observer-coordination`) owned by `governance-svc`. Manages international observer missions, pre-election and election-day briefings, structured observation reports, joint statements, and post-election analytical summaries. Each mission operates in a workspace-isolated tenant that cannot access data from other missions or from domestic observer workflows.

## Contents

1. What International Observer Coordination does (vs M43 Observer Accreditation)
2. Data plane (schema)
3. Mission workspace isolation
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What International Observer Coordination does (vs M43 Observer Accreditation)

- **M43** (Observer Accreditation Portal) = upstream accreditation: vets and badges domestic and international observers, issues observer credentials.
- **M57** = downstream coordination: manages the diplomatic and analytical lifecycle of accredited international missions after M43 has approved them.

M57 reads M43 accreditation records (via `eel.training.observer_accredited.v1`) and creates a mission workspace for each approved international body. Domestic civil-society observers are handled entirely in M43 and M34; M57 is for diplomatic missions (AU, EU, Commonwealth, Carter Center, etc.).

## 2. Data plane

Four aggregates (per `services/governance-svc/crates/intl-observer/src/models.rs` and the M57 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **ObserverMission** | Mission record: `mission_id`, `body_name`, `accreditation_id` (from M43), `lead_contact_name`, `workspace_id`, `country_count`, `observer_count`, `deployment_regions` (array of constituency codes), `status` (PENDING / ACTIVE / REPORTING / CLOSED). | B |
| **Briefing** | Pre-election or election-day briefing session: `briefing_id`, `mission_id`, `briefing_type` (PRE_ELECTION / ELECTION_DAY / POST_ELECTION), `scheduled_at`, `presenter_officer`, `materials_url` (MinIO path), `attendance_count`, `minutes_url`. | C |
| **ObservationReport** | Structured field report submitted by mission: `report_id`, `mission_id`, `reporting_phase` (OPENING / MIDDAY / CLOSING / FINAL), `constituency_codes` (observed), `findings_md`, `severity_flags` (array of `CREDIBLE_IRREGULARITY / SYSTEMIC_CONCERN / COMMENDATION`), `submitted_at`. | B |
| **JointStatement** | Coordinated post-election statement: `statement_id`, `mission_ids` (array), `draft_md`, `approved_by_missions` (array of mission_id + timestamp), `published_at`, `publication_url`. | B |

## 3. Mission workspace isolation

Each `ObserverMission` gets a dedicated `workspace_id` that is a row-security namespace in every M57 table. Postgres row-level security (RLS) policies enforce that a mission officer authenticated under one `workspace_id` cannot read another mission's `ObservationReport` or `Briefing`.

IEBC coordination officers hold the `OBSERVER_COORDINATION_OFFICER` SpiceDB role and can read across all workspaces. Mission officers authenticate via SAML federation (their own body's IdP) and are issued a JWT scoped to their `workspace_id`.

JointStatements cross workspace boundaries by explicit consent: each mission's lead contact must approve inclusion of their mission in the statement before the draft is visible to other missions.

## 4. Pipeline

Files on disk under `services/governance-svc/crates/intl-observer/src`: `lib.rs`, `models.rs`, `mission.rs`, `briefing.rs`, `report.rs`, `statement.rs`, `routes/`.

```
M43 emits eel.training.observer_accredited.v1 (international body)
      -> M57 creates ObserverMission; assigns workspace_id
      -> emit eel.governance.intl_mission_activated.v1
      -> coordination officer schedules Briefing
            -> materials uploaded to MinIO
            -> mission officers attend (attendance logged)
            -> minutes recorded; emit eel.governance.briefing_held.v1
      -> mission officers submit ObservationReports during election
            -> CREDIBLE_IRREGULARITY flag: emit eel.governance.credible_irregularity.v1
               -> ECC (M16) receives for situational awareness
            -> reports visible only to submitting mission + IEBC coordination officers
      -> post-election: coordination officer proposes JointStatement
            -> draft shared with named missions
            -> each mission lead approves (or opts out)
            -> approved statement published + URL stored
            -> emit eel.governance.joint_statement_published.v1
      -> mission closed; workspace archived (read-only)
```

## 5. API surface

Contract: `packages/openapi-contracts/governance/intl-observer.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/intl-observer/missions` | Coordination officer (JWT + SpiceDB `OBSERVER_COORDINATION_OFFICER`) | List all missions |
| `GET /v1/intl-observer/missions/{id}` | Mission officer (SAML JWT, workspace-scoped) | Mission detail |
| `POST /v1/intl-observer/missions/{id}/briefings` | Coordination officer | Schedule briefing |
| `PATCH /v1/intl-observer/missions/{id}/briefings/{bid}/record` | Coordination officer | Record attendance + minutes |
| `POST /v1/intl-observer/missions/{id}/reports` | Mission officer (workspace-scoped) | Submit observation report |
| `GET /v1/intl-observer/missions/{id}/reports` | Coordination officer | Read all reports for mission |
| `POST /v1/intl-observer/statements` | Coordination officer | Propose joint statement |
| `PATCH /v1/intl-observer/statements/{id}/approve` | Mission lead (workspace-scoped) | Approve inclusion in statement |
| `PATCH /v1/intl-observer/statements/{id}/publish` | Coordination officer | Publish approved statement |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Mission coordination console (Django) | `services/governance-svc/django/intl_observer_admin` | IEBC coordination officer: manage missions, schedule briefings |
| Mission portal (Astro) | `services/governance-svc/astro/mission-portal` | Mission officer: briefings, report submission, statement approval |

Languages: EN (primary); FR and PT available for mission portals (AU, EU contexts). WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **Row-level security**: Postgres RLS policies (`CREATE POLICY obs_mission_isolation ON ...`) enforce workspace isolation at the database layer, not just the application layer.
- **CREDIBLE_IRREGULARITY escalation SLA**: Prometheus alert fires if the event is not delivered to ECC (M16) within 120 seconds.
- **Report immutability**: once submitted, `ObservationReport` rows are insert-only; amendments create a new report version linked via `parent_report_id`. No in-place edits.
- **JointStatement consent chain**: publication is blocked by a Postgres trigger until all `mission_ids` listed in `approved_by_missions` have timestamps. The trigger cannot be bypassed by the application.
- **Audit shadow**: all aggregate mutations logged to `eel_governance_audit_shadow` with workspace_id, officer identity, and timestamp.
- **Materials integrity**: briefing material URLs point to MinIO objects whose `Content-MD5` is stored at upload; a nightly job re-checks all material hashes.

## 8. Kenya statute alignment

- **Elections Act §25 (observer access)**: observation reports and briefings support IEBC's obligation to facilitate observation by accredited international bodies.
- **Vienna Convention on Diplomatic Relations 1961**: mission workspace isolation protects diplomatic communication confidentiality; IEBC coordination officers cannot see inter-mission communication.
- **Kenya Data Protection Act §26 (minimisation)**: mission observer personal data (names, passport numbers) is stored in M43 only; M57 stores only `mission_id` and `lead_contact_name` for coordination purposes.
- **Elections Act §74 (prohibition on disclosure of observer reports before closure of polls)**: `ObservationReport` submitted during polling hours is embargoed in the API until `eel.rts.polls_closed.v1` is received.

## 9. Honest caveats + external prerequisites

- SAML federation with each international body's IdP requires bilateral metadata exchange per body. Some bodies (e.g., smaller Commonwealth missions) may not operate a SAML IdP; the mission portal supports a fallback username/password flow with MFA for those cases.
- JointStatement publication is an external act: the `publication_url` must be a URL the mission body controls; M57 does not host the public statement. It stores the URL and the internal approval chain.
- Observation reports are structured (severity flags + constituency codes + markdown findings), but the free-text `findings_md` field is not machine-analysed within M57. AI analysis of observation reports is not in scope.
- The election-day report embargo (statute alignment point 4) relies on receiving `eel.rts.polls_closed.v1` from the RTS. If this event is delayed, the embargo window extends, which may cause friction with mission communications teams.

## 10. Test coverage + operational runbooks

Coverage under `services/governance-svc/crates/intl-observer/tests/`: workspace isolation (integration, two-mission cross-access attempt must fail), CREDIBLE_IRREGULARITY escalation round-trip (pytest-httpserver), JointStatement consent trigger (integration), report immutability enforcement (property-based), report embargo (integration with mock RTS event).

Runbooks under `docs/runbooks/`:

- `governance-svc.md` (section: international-observer-coordination) -- deploy, rollback.
- `intl-mission-saml-federation.md` -- setting up SAML metadata for a new mission IdP.
- `credible-irregularity-escalation.md` -- what happens when an escalation fails to reach ECC.

Related documents: full spec at `docs/plans/modules/international-observer-coordination.md`; service capability catalogue at `services/governance-svc/docs/CAPABILITIES.md`; M43 at `docs/features/observer-accreditation-portal.md`.
