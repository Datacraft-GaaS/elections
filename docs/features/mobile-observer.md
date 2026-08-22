# Mobile observer (M34)

Module M34 (`mobile-observer`) owned by `field-mobile-svc`. A Flutter Android/iOS app for accredited domestic and international election observers. Supports signed station visit records, incident report attachments, a sampling verification checklist, mission PDF generation, AG mirror within 24 hours, and a USSD/SMS signal path for systemic incidents when data connectivity is unavailable.

## Contents

1. What mobile observer does (vs related modules)
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

## 1. What mobile observer does (vs related modules)

- **M12 CRMS** = accredits observers; M34 verifies observer credential at app login.
- **M34** = election-day observer tooling: visit logs, incident reports, sampling checklists, and the AG mirror pathway.
- **M23 Grievance channels** = public complaint intake; observers who observe systemic incidents can initiate an M23 CONDUCT referral from within M34.

M34 does not modify any voter, result, or custody record. All writes are append-only observer logs that feed the AG mirror and the IAB systemic incident dashboard.

## 2. Data plane

Five local aggregates in the Flutter Isar+SQLCipher store:

| Aggregate | Purpose | Sync cadence |
|---|---|---|
| **VisitRecord** | Signed station visit: visit_id (UUID7), observer_id, station_id, arrival_ts, departure_ts, observer_sig. | Immediate sync |
| **IncidentReport** | Incident with attachments: incident_id (UUID7), visit_id, category, description, attachment_hashes (P05 tlog refs), observer_sig, ts_hlc. | Immediate sync |
| **SamplingChecklist** | Standardised verification checklist per visit: checklist_id, visit_id, items (array of checked/unchecked), observer_sig. | Immediate sync |
| **MissionPDF** | Observer mission summary: pdf_id, observer_id, election_cycle, visit_count, incident_count, generated_at, pdf_hash. | On-demand generation |
| **SystemicIncident** | High-severity pattern: incident_ids (array), constituency_id, severity, ag_mirror_sent (bool), m23_ref. | On escalation |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| CRMS observer credential (M12) | At app login | REST | JWT (observer device cert) | Login blocked if CRMS unreachable and no cached credential |
| Photo/video attachments (observer device) | Per incident | Camera / gallery | Device cert | Stored in Isar+SQLCipher offline; hashed and anchored on sync |
| Checklist templates | Pre-election bulk | Signed JSON | Device cert | Cached; stale if >14 days |
| AG mirror endpoint | At EOD and on systemic incident | REST (HTTPS) | JWT + SpiceDB AG_MIRROR | Retry until acked; MissionPDF generated from local data if AG endpoint unavailable |
| M23 referral path | On observer escalation | REST | Device cert | Exactly-once referral_id |

## 4. Pipeline

```
observer arrives at station:
  opens M34 app; station QR scan
  -> CRMS credential verified (or cached)
  -> VisitRecord created (arrival_ts); observer_sig applied
  -> eel.observer.visit_started.v1 emitted

checklist completion:
  observer works through SamplingChecklist items:
       ballot box seals intact?
       voter roll accessible to agents?
       observers present from multiple parties?
       Form 34A displayed publicly?
       accessibility accommodations in place?
  -> SamplingChecklist persisted; observer_sig applied
  -> eel.observer.checklist_completed.v1 emitted

incident report:
  observer taps "Report incident" -> category -> description -> attachments
  -> IncidentReport(incident_id = UUID7) created
  -> attachments stored offline; hashes anchored in P05 tlog on sync
  -> observer_sig applied
  -> eel.observer.incident_reported.v1 emitted
  -> if category = SYSTEMIC:
       SystemicIncident created
       ag_mirror_sent initiated within 1 h
       M23 CONDUCT referral option presented to observer

departure:
  observer taps "Leave station"
  -> VisitRecord(departure_ts) updated
  -> MissionPDF generated on-device from visit + checklist + incidents

AG mirror (within 24 h):
  field-mobile-svc batch job aggregates all VisitRecord, IncidentReport,
  SamplingChecklist for the polling day
  -> signed aggregate submitted to AG mirror endpoint (REST)
  -> SystemicIncident.ag_mirror_sent = true on ack
  -> eel.observer.ag_mirror_sent.v1 emitted

USSD/SMS signal for systemic incident (no data):
  observer sends USSD `*400*9#` or SMS keyword to IEBC number
  -> field-mobile-svc ecc adapter creates SystemicIncident from USSD/SMS
  -> IAB systemic incident dashboard notified within 5 min
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/observer.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /observer/v1/visits` | Observer device (cert) | Start station visit |
| `PATCH /observer/v1/visits/{id}` | Observer device (cert) | Close visit (departure_ts) |
| `POST /observer/v1/incidents` | Observer device (cert) | Report incident with attachment hashes |
| `POST /observer/v1/checklists` | Observer device (cert) | Submit sampling checklist |
| `POST /observer/v1/mission-pdf` | Observer device (cert) | Generate mission PDF |
| `GET /observer/v1/incidents` | IAB / AG (JWT + SpiceDB) | Incident feed |
| `GET /observer/v1/systemic` | IAB / AG (JWT) | Systemic incident feed |
| `POST /observer/v1/ag-mirror` | field-mobile-svc batch (SVID) | Submit AG mirror batch |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Observer Flutter app (Android/iOS) | `services/field-mobile-svc/flutter/observer/` | Visit log; checklist; incident; mission PDF |
| IAB systemic incident dashboard (Phoenix LiveView) | `services/field-mobile-svc/ex/observer_web/` | Real-time systemic incident map; AG mirror status |
| AG mirror portal (read-only, Astro) | `services/field-mobile-svc/astro/ag-mirror/` | AG views visit, checklist, and incident data within 24 h of polling day |

Languages: EN + SW + French (for international observers). WCAG 2.2 AA on all web surfaces; Flutter app follows Material You guidelines.

## 7. Integrity + anti-tamper

- **Observer Ed25519 signature** -- every VisitRecord, IncidentReport, and SamplingChecklist is signed by the observer's device cert; unsigned records are rejected by the sync endpoint; the observer cannot repudiate a signed report.
- **Attachment P05 anchor** -- attachment hashes are anchored in the P05 transparency log within 60 s of sync; the AG or a court can verify the attachment has not been altered since the observer submitted it.
- **AG mirror within 24 h** -- the batch job has a 24-hour SLA measured from polling day close; a breach triggers an alert to the IEBC AG liaison; the batch is retried until acked.
- **USSD/SMS systemic signal** -- the USSD/SMS path is independent of the Flutter app and the data network; an observer without data connectivity can still trigger the systemic incident detection pipeline.
- **Exactly-once incident events** -- `IncidentReport.incident_id` is a UUID7 generated on the device at the moment of report creation; duplicate incident_ids on sync return the cached ack; network retries cannot create duplicate incidents.
- **MissionPDF on-device generation** -- the PDF is generated locally from the signed local Isar records; it cannot be post-hoc modified; the pdf_hash is stored and anchored independently.

## 8. Kenya statute alignment

- **EOA 2016 ss.34-36 (observer rights)** -- M34 implements the statutory observer toolset; VisitRecord and IncidentReport are the digital equivalents of the paper observation forms prescribed by EOA.
- **EOA 2016 s.11 (AG access)** -- AG mirror within 24 h satisfies the statutory access obligation; the AG read-only portal provides county-scoped access.
- **Elections Act s.44 (observer accreditation)** -- M34 verifies CRMS-issued accreditation at login; non-accredited devices are rejected.
- **DPA 2019 s.26 (minimisation)** -- incident attachments store hashes only on the server; raw media stored only on the observer's device and purged 90 days after election; USSD/SMS signals contain no voter PII.
- **CRPD Article 33** -- observer monitoring function includes accessibility compliance verification items in the SamplingChecklist; M34 collects data that contributes to the IEBC's CRPD reporting obligation.

## 9. Honest caveats + external prerequisites

- Observer CRMS accreditation must be issued before election day; late accreditations that are not in the CRMS cache at election time block the observer from logging in until connectivity is restored.
- The AG mirror REST endpoint must be provisioned and authenticated before election day; the endpoint URL and mutual TLS certificate are pending AG office confirmation.
- Attachment storage on the observer's device is bounded by device storage; high-resolution video attachments can fill a low-end device; the app recommends photo over video and warns when storage is low.
- The USSD `*400*9#` systemic incident path shares the short-code with M23 and M29; the CCK allocation must cover all three use cases under a single commercial agreement.
- French localisation for international observers requires a professional translation review before each election cycle; machine translation is not adequate for legal and electoral terminology.
- MissionPDF generation on-device requires offline PDF rendering; the app uses a bundled PDF library; any PDF specification change requires an app release.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/flutter/observer/test/` and `services/field-mobile-svc/crates/observer-api/tests/`: unit tests for visit record builder, incident exactly-once guard, checklist serialiser, MissionPDF generator, AG mirror batch assembler, USSD/SMS systemic signal parser; widget tests for all screens; integration tests with CRMS stub, P05 tlog stub, AG mirror stub, M23 referral stub; e2e Flutter integration test covering visit, checklist, incident with attachment, USSD systemic signal, and AG mirror; chaos test for AG mirror endpoint unavailable (verifies retry).

Runbooks under `docs/runbooks/`:

- `observer-degradation.md` (create): CRMS credential cache miss; AG mirror endpoint unavailable; USSD systemic signal delivery failure; attachment sync failure.

Dashboards: `platform/observability/field-mobile-svc/grafana-observer.json` (visit count, incident count, systemic incident count, AG mirror status, USSD signal count). SLO: AG mirror within 24 h of polling day close; incident sync <60 s; USSD systemic signal delivery <5 min.

Related: `docs/plans/modules/mobile-observer.md`; `services/field-mobile-svc/docs/CAPABILITIES.md`.
