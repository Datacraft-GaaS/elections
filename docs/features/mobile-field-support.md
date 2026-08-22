# Mobile field support (M32)

Module M32 (`mobile-field-support`) owned by `field-mobile-svc`. A Flutter app for field support engineers (~200 users). Manages SEV1/2/3 ticketed incidents with SLA timers, signed remote diagnostic actions (kit-consent required), warranty claims routed to OEM (via M20 O04 path), spare-part requests via M20 inventory, escalation to Datacraft / Mechrevo / IEBC ICT L2/L3, shift handoff dual-sign, and an offline diagnostic runbook library. All diagnostic actions are signed and require explicit kit consent before execution.

## Contents

1. What mobile field support does (vs M20 inventory management)
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

## 1. What mobile field support does (vs M20 inventory management)

- **M20** = tracks equipment custody chain; manages spares stock at depots.
- **M32** = the field-side app for the engineer responding to a fault: creates tickets, runs diagnostics on a specific kit (with its consent), requests spares from M20, escalates to OEM, and hands off shifts.

M32 is a write consumer of M20 (spare requests) and a producer of warranty claims and custody events (kit recovery).

## 2. Data plane

Five aggregates (per `services/field-mobile-svc/crates/field-support-api/src/models.rs`):

| Aggregate | Purpose | Data class |
|---|---|---|
| **SupportTicket** | ticket_id (UUID7), sev (SEV1/2/3), kit_serial, station_code, fault_description, status (OPEN/IN_PROGRESS/RESOLVED/ESCALATED), engineer_id, created_at, sla_deadline | D |
| **DiagnosticAction** | action_id, ticket_id, kit_serial, action_type (REBOOT/LOG_PULL/FIRMWARE_CHECK/NETWORK_TEST), kit_consent_sig, engineer_sig, executed_at, outcome | A |
| **SpareRequest** | request_id, ticket_id, item_type, quantity, depot_code, requested_at, fulfilled_at, m20_order_id | D |
| **WarrantyClaim** | claim_id, ticket_id, kit_serial, oem (DATACRAFT/MECHREVO), fault_description, filed_at, oem_ticket_ref | D |
| **ShiftHandoff** | handoff_id, station_code, outgoing_engineer_id, incoming_engineer_id, open_tickets (JSON), notes_hash, outgoing_sig, incoming_sig, handed_off_at | A |

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **M06 kit telemetry** | Real-time push | `integrations/bvi-dashboard/adapter` | SVID mTLS | Stale telemetry; manual fault description required |
| **M20 spare stock** | On request | gRPC read | SVID mTLS | Offline cached last stock level; discrepancy on sync |
| **Kit consent signal** | Per diagnostic action | Kit REST endpoint (local LAN) | Kit device cert | Action blocked if consent not received within 30 s |
| **OEM escalation API** (Datacraft / Mechrevo) | Per warranty claim | `integrations/oem-escalation/adapter` | API key (OEM-specific) | Offline queue; sync when connectivity restored |

## 4. Pipeline

Files under `services/field-mobile-svc/mobile/field-support/` and `crates/field-support-api/src/`:

```
Alert or self-dispatch triggers engineer to station
      -> SupportTicket created with SEV level:
            SEV1 <= 30 min SLA (kit down on election day)
            SEV2 <= 4 h SLA (kit degraded pre-election)
            SEV3 <= 24 h SLA (non-critical fault)
      -> SLA timer starts; ECC console shows countdown

Diagnostic workflow:
      -> engineer arrives; connects to kit on local LAN
      -> app requests kit consent (POST kit REST endpoint)
            kit_consent_sig returned by kit firmware (Ed25519)
      -> DiagnosticAction created:
            action_type selected from runbook library
            engineer_sig applied (JWT + device cert)
            both sigs stored immutably
      -> action executed on kit; outcome logged
      -> if firmware_check fails: WarrantyClaim auto-drafted

Spare request:
      -> SpareRequest created; POST /field/v1/spares to M20 inventory
      -> M20 DispatchOrder created (auto-approved for SEV1 within 15 min)
      -> SpareRequest.fulfilled_at updated on M20 dispatch confirmation

Escalation:
      -> if SEV1 unresolved at 20 min: auto-escalate to Datacraft L2 (ticket + alert)
      -> if unresolved at 30 min: IEBC ICT national escalation
      -> if hardware defect confirmed: WarrantyClaim filed via OEM escalation adapter

Shift handoff:
      -> outgoing engineer lists open tickets + notes (hashed)
      -> incoming engineer reviews + signs (biometric or PIN)
      -> ShiftHandoff persisted; eel.field.shift_handoff.v1 emitted
      -> open tickets transferred to incoming engineer_id

Events emitted via packages/eel-outbox-rs:
      eel.field.ticket_created.v1
      eel.field.diagnostic_action.v1
      eel.field.spare_requested.v1
      eel.field.warranty_filed.v1
      eel.field.shift_handoff.v1
```

Debezium to `eel.field.support.v1`.

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/field-support.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /field/v1/tickets` | Engineer (JWT) | Create support ticket |
| `GET /field/v1/tickets` | Engineer / ECC (JWT) | Ticket list with SLA status |
| `PATCH /field/v1/tickets/{id}` | Engineer | Update status / resolution |
| `POST /field/v1/tickets/{id}/diagnostics` | Engineer (JWT + kit consent) | Run diagnostic action |
| `POST /field/v1/tickets/{id}/spares` | Engineer | Request spare from M20 |
| `POST /field/v1/tickets/{id}/warranty` | Engineer | File OEM warranty claim |
| `POST /field/v1/handoffs` | Outgoing + incoming engineer (JWT + biometric/PIN) | Shift handoff dual-sign |
| `GET /field/v1/runbooks` | Engineer (offline cache) | Diagnostic runbook library |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Flutter app (field engineer) | `services/field-mobile-svc/mobile/field-support/` | Android 8+ / iOS 14+; offline runbooks; local LAN diagnostic; SLA timer |
| ECC console (M23 integration) | `services/ecc-svc/ex/ecc_web/` | SEV1 escalation alerts, ticket status board |
| Commission admin (Django) | `services/field-mobile-svc/django/field-support-admin/` | Ticket archive, warranty claim tracking, handoff audit |

Languages: EN + SW. Runbook library bundled offline; no connectivity required for diagnosis step-by-step.

## 7. Integrity + anti-tamper

- **Kit consent required for diagnostics** -- every DiagnosticAction requires a kit-generated Ed25519 consent signature; actions without kit consent are rejected server-side; prevents remote access without the kit's knowledge.
- **Dual-sign shift handoff** -- ShiftHandoff requires both outgoing and incoming engineer signatures; single-signer rejected; open tickets cannot be abandoned without a signed handoff.
- **SLA timer immutability** -- `sla_deadline` set at ticket creation; cannot be extended without IEBC ICT manager signature and a log entry.
- **DiagnosticAction append-only** -- action records immutable once created; outcome field updated within the same transaction; no silent deletion.
- **OEM offline queue** -- warranty claims queued locally if OEM API unreachable; synced on reconnect; no claim is silently dropped.
- **Spare-request coupling to M20** -- SpareRequest creates a M20 DispatchOrder; the M20 custody chain ensures the spare is traceable from depot to engineer to kit.

## 8. Kenya statute alignment

- **Elections Act s.80 (election materials)** -- kit recovery custody events in M20 (triggered by M32 resolution) satisfy the material-return obligation.
- **PPADA 2015** -- OEM warranty claims and spare procurement reference M20 purchase records; audit trail available.
- **Kenya Data Protection Act s.26 (minimisation)** -- no voter data in any M32 record; fault descriptions are equipment-technical only; notes hashed before storage.
- **Elections (Technology) Regulations 2017** -- all diagnostic actions and firmware changes must be logged; M32 DiagnosticAction log satisfies the change-record obligation.

## 9. Honest caveats + external prerequisites

- Kit consent requires the kit and the engineer's device to be on the same local LAN or have reachable IP. In environments without LAN (mobile hotspot only), the kit REST endpoint may be unreachable. A Bluetooth fallback for consent signalling is planned but not implemented.
- Datacraft and Mechrevo escalation APIs are OEM-specific REST endpoints; the adapter is written to a documented template but the exact endpoint contracts are not finalised. The OEM MoU must specify API format and SLA before go-live.
- SEV1 auto-escalation at 20 minutes assumes ECC is staffed 24/7 during the election window. ECC staffing schedule must be confirmed before deployment; an unstaffed ECC console receiving a SEV1 alert provides no value.
- The offline runbook library is bundled at app release; any in-field runbook update requires an app OTA update or a manual download over the support portal. A runbook update after apps are deployed to 200 engineers requires coordinated OTA.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/crates/field-support-api/tests/` and `mobile/field-support/integration_test/`:
- Unit: SLA timer, kit consent verification, shift handoff dual-sign, spare-request M20 coupling, OEM offline queue.
- Property (`proptest`): DiagnosticAction append-only; SLA deadline immutable without manager sig; ShiftHandoff rejects single-signer.
- Contract (Pact): M20 spare dispatch consumer; M06 telemetry consumer; M23 ECC escalation producer.
- Device: local LAN diagnostic on emulated kit; offline runbook library access; no-connectivity OEM queue.
- Security: kit consent bypass; action without engineer sig; SLA extension without manager sig; OWASP ZAP.

Runbooks under `docs/runbooks/`:
- `sev1-kit-failure-response.md` -- escalation chain, spare dispatch, kit replacement, P05 custody anchor.
- `oem-escalation-flow.md` -- Datacraft L2/L3, Mechrevo, IEBC ICT escalation, warranty tracking.
- `shift-handoff-procedure.md` -- dual-sign ceremony, open-ticket transfer, ECC notification.

Related: full spec at `docs/plans/modules/mobile-field-support.md`; capability catalogue at `services/field-mobile-svc/docs/CAPABILITIES.md`.
