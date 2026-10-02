# Mobile field support (M32)

Module M32 (`mobile-field-support`) owned by `field-mobile-svc`. A Flutter Android/iOS app for Datacraft engineers and the Commission IT field technicians. Supports ticket creation and resolution with signed diagnostics, safe remote actions (kit-consent required), warranty logging under O04, escalation matrix, offline runbook access, and shift handoff with dual-officer sign-off.

## Contents

1. What mobile field support does (vs related modules)
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

## 1. What mobile field support does (vs related modules)

- **M11 Internal field support** = centralised helpdesk and escalation management; M32 is the field engineer's mobile endpoint into M11.
- **M20 Inventory management** = asset custody chain; M32 reads kit status from M20 and writes `DISCREPANCY` events when a physical fault is found.
- **M32** = the engineer's on-the-ground toolset: diagnostics, remote action consent, ticket workflow, warranty logging, runbooks, and shift handoff.

M32 does not modify voter records, result tallies, or custody chain events directly. All writes are through M11 (tickets), M20 (discrepancy events), or the shift handoff endpoint.

## 2. Data plane

Six local aggregates in the Flutter Isar+SQLCipher store:

| Aggregate | Purpose | Sync cadence |
|---|---|---|
| **SupportTicket** | Field ticket: ticket_id (UUID7), kit_serial, fault_type, description, status, engineer_sig. | Immediate sync |
| **DiagnosticReport** | Signed system diagnostics: report_id, kit_serial, diagnostics_json, engineer_sig, ts_hlc. | Immediate sync |
| **RemoteActionConsent** | Consent record before remote action: action_id, kit_serial, dpo_consent_sig, engineer_id, action_type, ts_hlc. | Immediate sync before action |
| **WarrantyLog** | O04 warranty event: log_id, kit_serial, warranty_ref, event_type (CLAIM/REPLACE/REPAIR), engineer_sig. | Sync on reconnect |
| **ShiftHandoff** | End-of-shift record: handoff_id, outgoing_engineer_sig, incoming_engineer_sig, open_tickets (array), ts_hlc. | Immediate sync |
| **RunbookCache** | Offline runbook content: runbook_id, title, content_markdown, version, cached_at. | Pre-election bulk; delta push |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| M11 ticket assignments | Real-time push | Kafka event -> push | Device cert | Queued offline; synced on connectivity |
| M20 kit status | Pull + push | REST | Device cert | Cached with staleness badge |
| DPO consent signal (M31) | On demand | REST | M31 device cert | Remote action blocked without consent |
| Runbook content | Pre-election bulk | Signed Markdown | Device cert | Bundled in app binary as hard-offline fallback |
| O04 warranty API | On demand | REST | SVID mTLS | Warranty log queued if O04 unavailable |

## 4. Pipeline

```
fault reported (engineer observes or alert received from M11):
  engineer opens M32 app; creates SupportTicket
  -> kit_serial scanned (QR or barcode)
  -> DiagnosticReport generated: CPU, memory, storage, network, biometric sensor health
  -> engineer_sig applied to report (device cert)
  -> SupportTicket and DiagnosticReport synced to M11
  -> M20 DISCREPANCY event if physical fault found

remote action (e.g., software reset, config push):
  engineer selects remote action type
  -> RemoteActionConsent required:
       engineer requests consent from DPO (via M31 push)
       DPO reviews and signs consent (dpo_consent_sig)
       consent must be received before action proceeds
  -> action executed; result logged in DiagnosticReport
  -> eel.support.remote_action.v1 emitted

warranty event:
  engineer selects "Log warranty event" for a kit
  -> WarrantyLog created with warranty_ref from O04
  -> O04 API called (or queued if offline)
  -> M20 unit status updated (WARRANT_CLAIMED)

escalation:
  ticket unresolved after SLA threshold (configurable per fault_type)
  -> escalation matrix: engineer -> county coordinator -> national IT lead
  -> M11 notified via eel.support.escalation.v1

shift handoff:
  outgoing engineer selects "Shift handoff"
  -> open tickets listed for review
  -> outgoing_engineer_sig applied
  -> incoming engineer scans QR, verifies, applies incoming_engineer_sig
  -> ShiftHandoff synced; M11 notified
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/field-support.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /support/v1/tickets` | Engineer device (cert) | Create support ticket |
| `POST /support/v1/diagnostics` | Engineer device (cert) | Submit signed diagnostic report |
| `POST /support/v1/remote-actions` | Engineer device (cert) | Request remote action (consent required) |
| `POST /support/v1/consent/{action_id}` | DPO (M31 device cert) | Grant or deny remote action consent |
| `POST /support/v1/warranty` | Engineer device (cert) | Log warranty event |
| `POST /support/v1/handoffs` | Engineer device (cert, dual-sig) | Submit shift handoff |
| `GET /support/v1/runbooks/{type}` | Engineer device (cert) | Fetch runbook content |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Field support Flutter app (Android/iOS) | `services/field-mobile-svc/flutter/field-support/` | Ticket, diagnostics, remote action consent, warranty, handoff, runbooks |
| M11 ticket management (internal web) | `services/support-admin-svc/` | M11 receives M32 tickets and escalations |

Languages: EN + SW. Material You accessibility. WCAG 2.2 AA on all screens. Offline-first: runbooks bundled; diagnostics available without connectivity.

## 7. Integrity + anti-tamper

- **Signed diagnostics** -- every `DiagnosticReport` is signed by the engineer's device cert; unsigned reports are rejected by M11; the signature binds the diagnostic to the engineer who collected it.
- **Kit-consent required for remote actions** -- `RemoteActionConsent` with a DPO signature is a mandatory prerequisite; the field-mobile-svc API verifies the consent sig against the DPO's registered M31 device cert before executing any remote action.
- **Shift handoff dual-sig** -- `ShiftHandoff` requires both outgoing and incoming engineer signatures; a single-signer handoff is rejected; open tickets cannot be orphaned without a signed handoff.
- **Warranty log to M20** -- warranty events update the M20 custody chain (`WARRANT_CLAIMED`); the WarrantyLog and M20 event share the same kit_serial and timestamp, making them jointly auditable.
- **Offline runbook integrity** -- runbook content is signed at bulk push time; the app verifies the signature before rendering; tampered runbook content is not displayed.

## 8. Kenya statute alignment

- **Elections (Technology) Regulations 2017** -- IT support events and diagnostic logs retained for 7 years as part of the election audit trail.
- **O04 warranty terms** -- warranty events logged against the O04 contract reference; the WarrantyLog is the authoritative record for warranty claim disputes.
- **DPA 2019 s.26 (minimisation)** -- diagnostic reports contain system health data only; no voter data or biometric data is included; the DiagnosticReport schema enforces this at the API level.
- **EOA 2016 s.11 (accountability)** -- signed diagnostics and shift handoffs create an accountable chain of custody for IT support actions on election equipment.

## 9. Honest caveats + external prerequisites

- The O04 warranty API endpoint must be confirmed with the hardware vendor before election day; the adapter and stub are ready but the live endpoint URL is pending contract finalisation.
- Remote action consent from the DPO requires M31 connectivity; if the DPO's M31 device is offline, the consent request is queued and the remote action is blocked until consent is received; this may delay critical repairs.
- The escalation matrix thresholds (SLA per fault_type) must be configured by the Commission IT before election day; defaults are provided but may not match operational reality.
- Shift handoff requires both engineers to be physically present with the incoming engineer's device; if the incoming engineer is remote, the handoff must use an alternative procedure (documented in the runbook).
- The DiagnosticReport JSON schema covers the biometric kit's documented diagnostic interface; any firmware update that changes the diagnostic API requires a corresponding app update.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/flutter/field-support/test/` and `services/field-mobile-svc/crates/field-support-api/tests/`: unit tests for diagnostic report builder, consent validator, warranty logger, escalation timer, handoff dual-sig enforcer; widget tests for all screens; integration tests with M11 stub, M20 discrepancy stub, DPO consent M31 stub, O04 warranty stub; e2e Flutter integration test covering fault report, diagnostic, remote action consent, warranty log, and shift handoff; chaos test for O04 API unavailable (verifies queue).

Runbooks under `docs/runbooks/`:

- `field-support-degradation.md` (create): O04 API down; DPO consent unavailable; M11 sync failure; shift handoff engineer absent.

Dashboards: `platform/observability/field-mobile-svc/grafana-fieldsupport.json` (ticket creation rate, resolution time, remote action count, warranty claim count, handoff completion rate). SLO: diagnostic sync <60 s; remote action consent response <5 min; shift handoff completion before shift end.

Escalation matrix (condensed): SEV1 unresolved at 20 min -> Datacraft L2 (auto-ticket); 30 min -> the Commission ICT national (auto-notify); 60 min -> the Commission Commissioner briefing note. SEV2 unresolved at 4 h -> county coordinator + L2. SEV3 unresolved at 24 h -> ops review.

Runbook library: all runbooks signed at pre-election bulk push; library covers BVR kit failure, biometric sensor replacement, network adapter swap, scanner calibration, and firmware rollback; new runbooks can be pushed via OTA without app update.

Related: `docs/plans/modules/mobile-field-support.md`; `services/field-mobile-svc/docs/CAPABILITIES.md`.
