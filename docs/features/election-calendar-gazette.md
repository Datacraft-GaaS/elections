# Election calendar and gazette (M52)

Module M52 (`election-calendar-gazette`) owned by `portal-svc` (crate `gazette`). Publishes and manages the statutory election calendar and gazette notices: enforces a statutory rule engine for legal deadlines, assembles and submits notices to the Government Printer, enables public and institutional subscriptions to calendar events, and is the final step in the declaration-to-certificate-to-gazette pipeline fed by M51.

## Contents

1. What M52 does (vs M51 declaration and M11 public portal)
2. Data plane (schema)
3. Statutory rule engine
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M52 does (vs M51 declaration and M11 public portal)

- **M51** = result declaration and certification; it emits `eel.declaration.result_declared.v1` and `eel.declaration.certificate_issued.v1` which M52 consumes to trigger gazette notice publication.
- **M11** = public portal (general citizen-facing information); it embeds M52's calendar as a data feed but does not own the gazette logic.
- **M52** = the authoritative source for all statutory election dates, gazette notices, and Government Printer submissions. It owns the statutory rule engine that computes derived deadlines (nomination close, campaign period end, silence period) from a set of base calendar events.

M52 is downstream of nearly every operational module (M49, M50, M51, M35 CVR gazette). It does not compute election results; it announces them.

## 2. Data plane

Four aggregates (per `services/portal-svc/crates/gazette/src/models.rs`):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **CalendarEvent** | A statutory date: `event_id` (UUID7), `election_cycle`, `event_type` (`NOMINATION_OPEN / NOMINATION_CLOSE / CAMPAIGN_START / CAMPAIGN_END / SILENCE_PERIOD_START / SILENCE_PERIOD_END / ELECTION_DAY / RESULTS_DEADLINE / PETITION_WINDOW_CLOSE / RUNOFF_DATE`), `computed_date`, `rule_id` (FK to StatutoryRule), `gazette_notice_id` (FK to GazetteNotice, if gazette is required), `is_amended` (boolean), `amendment_reason_hash`. | B |
| **GazetteNotice** | A formal public notice: `notice_id` (UUID7), `election_cycle`, `notice_type` (`ELECTION_NOTICE / RESULT_DECLARATION / CERTIFICATE_ISSUE / PARTY_DEREGISTRATION / RECALL_PETITION / CVR_REMOVAL / REFERENDUM`), `notice_text_s3_key`, `notice_sha256`, `hsm_signature`, `gov_printer_submission_ref`, `gov_printer_ack_at`, `published_at`, `status` (`DRAFT / SUBMITTED / PUBLISHED / WITHDRAWN`). | A |
| **StatutoryRule** | A rule that derives a CalendarEvent date from a base event: `rule_id` (UUID7), `rule_name`, `base_event_type`, `derived_event_type`, `offset_days` (signed; negative = before base), `offset_basis` (`CALENDAR / BUSINESS`), `legal_source` (`Elections Act §X` or `Reg X`), `election_cycle_applicability[]`. | B |
| **Subscription** | Public or institutional subscription to calendar updates: `subscription_id` (UUID7), `subscriber_email_hash`, `subscriber_type` (`CITIZEN / PARTY / MEDIA / INSTITUTION`), `event_types[]`, `delivery_channel` (`EMAIL / SMS / WEBHOOK`), `active` (boolean), `created_at`. | C |

## 3. Statutory rule engine

The rule engine computes derived `CalendarEvent` dates from base dates using `StatutoryRule` rows. Examples:

| Rule name | Base event | Derived event | Offset | Legal source |
|---|---|---|---|---|
| Nomination close | ELECTION_DAY | NOMINATION_CLOSE | -45 days | Elections Act §24(1) |
| Campaign start | NOMINATION_CLOSE | CAMPAIGN_START | +1 day | Elections (General) Regs Reg 71 |
| Silence period | ELECTION_DAY | SILENCE_PERIOD_START | -1 day | Elections Act §44B |
| Petition window close | ELECTION_DAY | PETITION_WINDOW_CLOSE | +28 days | Elections Act §86 |
| Results deadline | ELECTION_DAY | RESULTS_DEADLINE | +7 days | Elections Act §87 |

Rules are versioned per `election_cycle`. A rule change (e.g., Parliament amends an offset) requires a new `StatutoryRule` row with the new `election_cycle_applicability`; old cycles retain the old rule. This ensures historical calendar events are not retroactively recomputed.

The rule engine is implemented in `services/portal-svc/crates/gazette/src/rule_engine.rs` and is property-tested via `proptest` against boundary conditions (leap years, long weekends for business-day offsets).

## 4. Pipeline

```
base CalendarEvent created by Commission admin (ELECTION_DAY for a cycle)
      -> StatutoryRule engine runs: derives all dependent CalendarEvents
      -> GazetteNotice drafted per event requiring gazette
      -> notice text rendered (EN + SW) from template + event data
      -> Commissioner HSM-signs notice (hsm_signature)
      -> notice submitted to Gov Printer API (integrations/gov-printer/gazette-report-poster)
            -> gov_printer_submission_ref stored
            -> gov_printer_ack_at recorded on acknowledgement webhook
      -> on Gov Printer acknowledgement: status -> PUBLISHED; published_at set
      -> emit eel.portal.gazette_notice_published.v1
      -> Subscription delivery:
            -> EMAIL subscribers notified via integrations/smtp/
            -> SMS subscribers notified via integrations/ussd-gateway/
            -> WEBHOOK subscribers called via integrations/webhook-fanout/

M51 pipeline integration:
      -> eel.declaration.result_declared.v1 consumed
            -> GazetteNotice(RESULT_DECLARATION) auto-created + drafted
            -> fast-path: auto-submitted within 30 min (SLA)
      -> eel.declaration.certificate_issued.v1 consumed
            -> GazetteNotice(CERTIFICATE_ISSUE) auto-created + submitted
      -> eel.governance.by_election_created.v1 consumed
            -> CalendarEvent(ELECTION_DAY) created for by-election
            -> rule engine derives all by-election deadlines
      -> eel.governance.referendum_activated.v1 consumed
            -> CalendarEvent(ELECTION_DAY) created for referendum
            -> referendum-specific rules applied (different offset set)
      -> eel.bvr.gazette_published.v1 consumed (M35 CVR removals)
            -> GazetteNotice(CVR_REMOVAL) forwarded to Gov Printer
```

## 5. API surface

Contract: `packages/openapi-contracts/portal/election-calendar-gazette.yaml` and `packages/openapi-contracts/gazette/` (shared listing). All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /v1/calendar/events` | Public (60/hr) | List CalendarEvents for current or specified cycle |
| `POST /v1/calendar/events` | Commission admin | Manually create base CalendarEvent (triggers rule engine) |
| `PATCH /v1/calendar/events/{event_id}` | Commissioner + HSM | Amend a CalendarEvent (is_amended = true; reason recorded) |
| `GET /v1/gazette/notices` | Public (60/hr) | List published GazetteNotices |
| `GET /v1/gazette/notices/{notice_id}` | Public | GazetteNotice detail + Gov Printer ref |
| `POST /v1/gazette/notices` | Commission admin | Manually draft GazetteNotice |
| `PUT /v1/gazette/notices/{notice_id}/publish` | Commissioner + HSM | Sign + submit to Gov Printer |
| `POST /v1/calendar/subscriptions` | Public (OIDC or email) | Subscribe to calendar updates |
| `DELETE /v1/calendar/subscriptions/{subscription_id}` | Subscriber | Unsubscribe |
| `GET /v1/calendar/rules` | Commission admin | List StatutoryRule rows + legal sources |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Gazette admin console (Django) | `services/portal-svc/django/portal-admin` | Commission admin creates events; Commissioner signs and submits |
| Public gazette portal (Astro) | `services/portal-svc/astro/gazette` | Citizens browse calendar + gazette notices; subscribe |
| Rule engine management (Django) | `services/portal-svc/django/portal-admin` | Admin manages StatutoryRule rows per election cycle |

Languages: EN + SW on all surfaces. Gazette notices: EN + SW bilingual. Subscription emails/SMS: EN + SW. WCAG 2.2 AA.

## 7. Integrity + anti-tamper

- **HSM-signed gazette notices** - `GazetteNotice.hsm_signature` is verifiable against the Commissioner public key in P05. A citizen or court can verify that a gazette notice was signed by the Commissioner and has not been tampered with.
- **Rule engine immutability** - `StatutoryRule` rows are versioned per election cycle; old cycle rules are not modified. A rule change for a future cycle does not alter past CalendarEvents.
- **CalendarEvent amendment audit** - amendments to CalendarEvents set `is_amended = true` and store the reason hash. The original computed date is retained; only `computed_date` is updated. The amendment is a visible record, not a silent overwrite.
- **Gov Printer acknowledgement** - M52 does not set `status = PUBLISHED` until the Gov Printer acknowledges the submission. If acknowledgement does not arrive within 30 minutes, an alert fires and ops retries manually.
- **Notice withdrawal** - a withdrawn gazette notice (`status = WITHDRAWN`) is never deleted; it is marked as superseded by the replacement notice. This follows the Government Printer's gazette correction procedure.
- **Subscription PII minimisation** - only `subscriber_email_hash` is stored in the `Subscription` table. The plaintext email is in the T2 subscriber store (access restricted to portal admin). SMS subscriptions store only the phone number hash.

## 8. Kenya statute alignment

- **Elections Act §24** - statutory timelines for elections; M52's `StatutoryRule` engine encodes each timeline with a `legal_source` reference.
- **Elections Act §44B** - electoral silence period; `StatutoryRule` derives `SILENCE_PERIOD_START` from ELECTION_DAY - 1.
- **Kenya Gazette Act Cap 8** - Government Printer publication requirements; M52's Gov Printer integration operationalises the mandatory gazette publication.
- **Elections (Technology) Regulations 2017** - IEBC must publish election results through official channels; M52 is the official channel for gazette publication.
- **Kenya Data Protection Act §26** - subscriber email/phone hashed; plaintext restricted to portal admin with access logged in `eel_portal_audit_shadow`.

## 9. Honest caveats + external prerequisites

- Gov Printer API integration (`integrations/gov-printer/gazette-report-poster`) requires the Government Printer to provide a machine-readable submission API. As of the spec date, the Government Printer uses a manual submission process (email + PDF). The adapter is written to a documented schema; if the live API is not provisioned, the adapter generates a structured email submission with the same content.
- Subscription delivery via SMS requires a Safaricom or Africa's Talking bulk SMS account. The adapter is at `integrations/ussd-gateway/adapter`; the live credentials are a procurement dependency.
- Statutory rule offsets are encoded from the Elections Act and Regulations as understood at spec time. Any subsequent amendment by Parliament requires an admin to update the `StatutoryRule` row and re-run the rule engine for the affected cycle. The system does not auto-detect legislative changes.
- Gazette notice text is generated from templates in `services/portal-svc/templates/gazette/`. These templates must be reviewed by Commission legal before each election cycle to ensure they reflect the current statutory language.

## 10. Test coverage + operational runbooks

Tests under `services/portal-svc/crates/gazette/tests/`: rule engine arithmetic via `proptest` (leap years, business-day offsets, edge cases at election day), Gov Printer submission mock via `pytest-httpserver`, subscription delivery idempotency, notice amendment audit trail.

Runbooks under `docs/runbooks/`:

- `portal-svc.md` - gazette module deploy + rollback.
- `gov-printer-down.md` - manual submission fallback when Gov Printer API is unavailable.
- `statutory-rule-update.md` - procedure for updating rules after legislative change.
- `gazette-correction.md` - notice withdrawal + replacement procedure per Gov Printer correction rules.

Related documents: spec at `docs/plans/modules/election-calendar-gazette.md`; M51 at `docs/features/result-declaration-certification.md`; M35 at `docs/features/continuous-voter-registration.md`; capability catalogue at `services/portal-svc/docs/CAPABILITIES.md`.
