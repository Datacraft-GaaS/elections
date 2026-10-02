# Diaspora ballot physical logistics (M65)

Module M65 (`diaspora-ballot-physical-logistics`) owned by `inventory-svc`. Manages the physical custody chain for diaspora ballot materials: from the Commission headquarters through freight handlers and airlines to embassy polling stations, and back. Every custody transfer requires multi-factor authentication (MFA) by the receiving officer and generates a signed `CustodyEvent`. Pre-clearance manifests are submitted to KRA and relevant airlines before dispatch.

## Contents

1. What Diaspora Ballot Physical Logistics does (vs M24 Diaspora Voting)
2. Data plane (schema)
3. Chain of custody model
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What Diaspora Ballot Physical Logistics does (vs M24 Diaspora Voting)

- **M24** (Diaspora Voting) = the software workflow: BVR and BVI at embassy polling stations, result transmission from diaspora to M10.
- **M65** = the physical logistics layer: tracks the boxes of ballot papers, security seals, and election materials that must arrive at embassy polling stations before voting begins.

M65 and M24 are complementary. M65 ensures the physical materials are in the right place at the right time; M24 ensures the voters and the digital systems are ready. A polling station blocked in M65 (missing custody confirmation) emits a flag to M24 which marks that station as NOT_READY.

## 2. Data plane

Four aggregates (per `services/inventory-svc/crates/diaspora-logistics/src/models.rs` and the M65 spec):

| Aggregate | Purpose | Data class (ADR-0016) |
|---|---|---|
| **BallotContainer** | A sealed shipment unit: `container_id`, `serial_range_ref` (from M39 Ballot Printing Chain), `destination_embassy_code`, `contents_manifest_hash` (SHA-256 of contents list), `seal_number`, `state` (PACKED / PRE_CLEARED / DISPATCHED / IN_TRANSIT / ARRIVED / OPENED / RETURNED), `current_custodian_officer`. | A |
| **CustodyEvent** | Transfer of custody: `custody_id`, `container_id`, `from_custodian`, `to_custodian`, `transfer_type` (HANDOVER_TO_FREIGHT / AIRLINE_CHECK_IN / EMBASSY_RECEIPT / SEAL_BREAK / POST_POLL_RETURN), `mfa_method` (TOTP / HARDWARE_KEY / BIOMETRIC), `mfa_verified` (bool), `transfer_at`, `location_code`. | A |
| **PreClearanceManifest** | Customs and airline pre-clearance: `manifest_id`, `container_ids` (array), `flight_number`, `departure_airport`, `arrival_airport`, `kra_customs_ref`, `airline_booking_ref`, `clearance_status` (PENDING / CLEARED / REJECTED), `cleared_at`. | B |
| **ContainerAlert** | Alert for a container that is overdue or has a custody anomaly: `alert_id`, `container_id`, `alert_type` (OVERDUE / SEAL_TAMPER / MANIFEST_MISMATCH / MFA_FAILURE), `raised_at`, `resolved_at`. | B |

## 3. Chain of custody model

Every custody transition requires the receiving officer to authenticate with MFA before the `CustodyEvent` is created:

- **TOTP**: time-based one-time password (RFC 6238, M'Raihi et al., IETF 2011) via authenticator app; used by freight handlers and airline ground staff.
- **HARDWARE_KEY**: FIDO2 hardware security key (FIDO Alliance FIDO2 Specification, 2019); used by the Commission officers at headquarters and embassies.
- **BIOMETRIC**: fingerprint verification via the ABIS (M03) gRPC integration; used by the Commission officers at embassies.

A `CustodyEvent` with `mfa_verified = false` is rejected by the API. The container state does not advance without MFA confirmation.

## 4. Pipeline

Files on disk under `services/inventory-svc/crates/diaspora-logistics/src`: `lib.rs`, `models.rs`, `container.rs`, `custody.rs`, `preclearance.rs`, `alert.rs`, `routes/`.

```
M39 emits eel.inventory.serial_range_dispatched.v1 for diaspora
      -> M65 creates BallotContainer (state = PACKED)
      -> logistics officer creates PreClearanceManifest
            -> KRA customs ref obtained via KRA API or manual entry
            -> airline booking ref obtained from airline
            -> manifest submitted; clearance status updated
            -> emit eel.inventory.preclearance_submitted.v1
      -> KRA clears manifest (inbound confirmation):
            -> clearance_status = CLEARED
      -> container handed to freight handler (MFA: TOTP)
            -> CustodyEvent: HANDOVER_TO_FREIGHT
            -> state = DISPATCHED
            -> emit eel.inventory.container_dispatched.v1
      -> airline check-in (MFA: TOTP by ground staff)
            -> CustodyEvent: AIRLINE_CHECK_IN
            -> state = IN_TRANSIT
      -> arrival at destination embassy (MFA: HARDWARE_KEY or BIOMETRIC)
            -> CustodyEvent: EMBASSY_RECEIPT
            -> state = ARRIVED
            -> emit eel.inventory.container_arrived.v1 -> M24 station marked MATERIALS_READY
      -> overdue monitor (cron, 1h): if container not ARRIVED by T-72h:
            -> create ContainerAlert: OVERDUE
            -> emit eel.inventory.container_overdue.v1 -> ECC (M16) alert
      -> post-poll: container returned with unused ballots
            -> CustodyEvent: POST_POLL_RETURN
            -> state = RETURNED
```

## 5. API surface

Contract: `packages/openapi-contracts/inventory/diaspora-logistics.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /v1/diaspora-logistics/containers` | Logistics officer (JWT + SpiceDB `LOGISTICS_OFFICER`) | Create ballot container |
| `POST /v1/diaspora-logistics/containers/{id}/custody` | Receiving officer (MFA-validated JWT) | Record custody transfer |
| `GET /v1/diaspora-logistics/containers/{id}` | Logistics officer, ECC system | Container state + custody history |
| `GET /v1/diaspora-logistics/containers` | Logistics officer | List containers by state + destination |
| `POST /v1/diaspora-logistics/manifests` | Logistics officer | Create pre-clearance manifest |
| `PATCH /v1/diaspora-logistics/manifests/{id}/kra-clear` | KRA system (SVID) or logistics officer | Record KRA customs clearance |
| `GET /v1/diaspora-logistics/alerts` | ECC system (SVID), logistics officer | Active container alerts |
| `PATCH /v1/diaspora-logistics/alerts/{id}/resolve` | Logistics officer | Resolve container alert |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Diaspora logistics console (Django) | `services/inventory-svc/django/diaspora_logistics_admin` | Create containers, manage manifests, track custody |
| ECC integration (M16) | Container alert feed via Kafka | ECC map overlay for overdue containers |
| Embassy officer app (mobile) | `services/field-mobile-svc/` | Embassy officer MFA receipt confirmation |

Languages: EN + SW. WCAG 2.2 AA on web surfaces.

## 7. Integrity + anti-tamper

- **MFA gate**: the API enforces `mfa_verified = true` before accepting any `CustodyEvent`; this is a synchronous check against the MFA token store, not a deferred assertion.
- **Seal number tracking**: `BallotContainer.seal_number` is recorded at packing; if the receiving officer reports a broken seal, a `SEAL_TAMPER` alert is raised automatically and the container is quarantined pending investigation.
- **Contents manifest hash**: `BallotContainer.contents_manifest_hash` is SHA-256 of the packing list; the embassy officer verifies the hash against the physical packing list on arrival. A mismatch raises a `MANIFEST_MISMATCH` alert.
- **Overdue SLA monitoring**: containers not confirmed ARRIVED by T-72 hours before poll open trigger a PagerDuty alert and an ECC escalation event.
- **Audit shadow**: every custody event logged to `eel_inventory_audit_shadow` with MFA method, officer identity, and GPS location if available.
- **Post-poll return chain**: the return custody chain is mandatory; containers left with embassies without a `POST_POLL_RETURN` event after T+14 raise a `MANIFEST_MISMATCH` alert.

## 8. Kenya statute alignment

- **Elections Act §70 (ballot paper security)**: physical chain of custody for ballot papers is a statutory requirement; M65 is the digital record of that chain.
- **Customs and Excise Act Cap 472 (export/import of election materials)**: pre-clearance manifests satisfy the customs declaration requirements for diplomatic ballot shipments.
- **Vienna Convention on Diplomatic Relations 1961 Art 27 (diplomatic bag exemption)**: certain embassy-to-embassy shipments may qualify for diplomatic bag treatment; M65 records the `diplomatic_bag` flag on `PreClearanceManifest` when applicable, which changes the KRA customs procedure.
- **Elections Act §23 (diaspora voting regulations)**: the physical logistics chain is a prerequisite for compliant diaspora polling; M65 is referenced in the diaspora voting regulations as the tracking mechanism.

## 9. Honest caveats + external prerequisites

- KRA customs clearance API integration requires the Commission to obtain KRA API credentials for the diplomatic shipments endpoint. Where the API is unavailable, customs ref is entered manually by the logistics officer.
- Airline integration is not automated; airline booking refs are entered manually, and airline check-in `CustodyEvent` entries are created by the ground staff's TOTP-authenticated device. Airlines must be briefed on the MFA custody recording process.
- FIDO2 hardware key issuance to embassy officers requires physical distribution of hardware tokens before the election. Lost tokens require re-issuance via the key management runbook.
- The overdue alert threshold (T-72h) may be too tight for distant embassy locations (e.g., South America, Asia) with limited direct flights; the threshold is configurable per embassy in `services/inventory-svc/config/embassy_overdue_thresholds.toml`.
- Post-poll return logistics are not legally required in all jurisdictions; some host countries may require ballots to remain in-country until the petition window closes. M65 supports a `HOLD_IN_COUNTRY` flag on `BallotContainer` that suppresses the return overdue alert.

## 10. Test coverage + operational runbooks

Coverage under `services/inventory-svc/crates/diaspora-logistics/tests/`: MFA gate (integration: custody event without MFA must fail), seal tamper alert (integration), overdue monitor (chaos: advance time past T-72h, assert alert), manifest hash mismatch (property-based), post-poll return overdue alert suppression with HOLD_IN_COUNTRY flag (integration).

Runbooks under `docs/runbooks/`:

- `inventory-svc.md` (section: diaspora-ballot-physical-logistics) -- deploy, rollback, container state repair.
- `container-overdue-response.md` -- logistics officer steps when a container is flagged overdue.
- `seal-tamper-quarantine.md` -- investigation procedure for a seal tamper alert.

Related documents: full spec at `docs/plans/modules/diaspora-ballot-physical-logistics.md`; service capability catalogue at `services/inventory-svc/docs/CAPABILITIES.md`; M24 diaspora voting at `docs/features/diaspora-voting.md`; M39 ballot printing at `docs/features/ballot-printing-chain.md`.

### Container state machine

```
PACKED -> PRE_CLEARED -> DISPATCHED -> IN_TRANSIT -> ARRIVED -> OPENED -> RETURNED
                                                             -> HOLD_IN_COUNTRY (suppresses return overdue alert)
```

Any transition that fails MFA reverts to the previous state and raises a `ContainerAlert: MFA_FAILURE`. Three consecutive MFA failures on the same container trigger an automatic `SEAL_TAMPER` alert even if the seal is reported intact, because repeated authentication failure is itself an anomaly signal.

### Embassy readiness gate

M65 feeds an embassy readiness status to M24 via the `eel.inventory.container_arrived.v1` event. M24's embassy polling station will not open (status `NOT_READY`) until:

1. At least one `BallotContainer` for the embassy has `state = ARRIVED`.
2. The `contents_manifest_hash` has been verified by the embassy officer.
3. The presiding officer has acknowledged receipt via M63 `IssuanceReceipt`.

All three conditions must hold. This prevents polling from starting at an embassy where ballot materials are still in transit.
