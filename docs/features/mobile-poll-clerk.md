# Mobile poll clerk (M31)

Module M31 (`mobile-poll-clerk`) owned by `field-mobile-svc`. A companion Flutter app for poll clerks (~92,000 users). Provides a read-only mirror of the kit status dashboard from M06 (BVI), a voter-lookup fallback that requires DPO biometric attestation logged permanently, an embedded paper-backup workflow for kit failure, a kiosk mode for public register inspection, and an end-of-day report dual-signed by PO + DPO. M31 does not perform voter authentication.

## Contents

1. What mobile poll clerk does (vs M05 voter identification)
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

## 1. What mobile poll clerk does (vs M05 voter identification)

- **M05** = authenticates voters biometrically on the BVI kit.
- **M31** = the clerk's companion: shows kit status, provides a DPO-gated fallback voter lookup for kit failure, embeds the paper-backup workflow, and generates the end-of-day report. It never authenticates voters directly.

The separation is intentional: M31 cannot be used to bypass M05 biometric authentication without a permanently logged DPO attestation.

## 2. Data plane

Four aggregates (per `services/field-mobile-svc/crates/poll-clerk-api/src/models.rs`):

| Aggregate | Purpose | Data class |
|---|---|---|
| **KitStatusMirror** | station_id, kit_serial, battery_pct, abis_connected, voter_count_today, last_sync_at (read-only from M06) | D |
| **FallbackLookupLog** | lookup_id, station_id, dpo_officer_id, dpo_biometric_ref_hash, national_id_hash, reason_code, lookup_at | A |
| **PaperBackupEvent** | event_id, station_id, reason (KIT_FAILURE/POWER_OUTAGE/OTHER), started_at, ended_at, ballot_count_fallback, po_sig, dpo_sig | A |
| **EndOfDayReport** | report_id, station_id, date, total_voters_identified, spoilt_ballots, rejected_ballots, closing_time, po_sig, dpo_sig, anchored_in_p05 | A |

## 3. Ingest sources

| Source | Cadence | Adapter | Auth | Failure semantics |
|---|---|---|---|---|
| **M06 kit dashboard** (read-only) | Real-time gRPC stream | `integrations/bvi-dashboard/adapter` | SVID mTLS | Stale mirror served; staleness shown; no write path |
| **BVR back-end M02** (fallback lookup) | On demand (DPO-gated only) | Internal gRPC | JWT + SpiceDB `DPO` role + live biometric | Blocked if DPO biometric unavailable |
| **M20 inventory** (paper ballot stock) | At station setup | gRPC read | SVID mTLS | Cached; discrepancy alert if count mismatch |
| **HSM P06** (end-of-day report signing) | Once per day | `integrations/hsm/adapter` | PO + DPO JWT + HSM | Report blocked until dual-sign complete |

## 4. Pipeline

Files under `services/field-mobile-svc/mobile/poll-clerk/` and `crates/poll-clerk-api/src/`:

```
Poll clerk opens app at station setup (T-1 / election day)
      -> login: clerk JWT + device cert (issued at training, valid for election week)
      -> kit status mirror loaded from M06 (read-only gRPC stream)
      -> KitStatusMirror displayed: battery, ABIS status, voter count

Voter-lookup fallback (kit failure only):
      -> PO declares kit failure; triggers fallback mode
      -> DPO must provide biometric attestation:
            live biometric captured on DPO device
            biometric_ref_hash logged permanently in FallbackLookupLog
      -> clerk enters national_id (never stored raw; hash derived on device)
      -> GET BVR register via M02 gRPC read (audit-logged against DPO ID)
      -> national_id_hash + lookup_at + dpo_officer_id stored immutably
      -> fallback lookup usage triggers automatic M06 kit-failure report

Paper-backup workflow (kit failure):
      -> PaperBackupEvent created with reason code
      -> embedded procedure guide displayed (offline, bundled)
      -> ballot_count_fallback entered manually; signed by PO at end

Kiosk mode (register inspection):
      -> device locked to register inspection screen
      -> voter self-checks registration (national_id hash, no biometric)
      -> no write access in kiosk mode; back-button disabled

End-of-day report:
      -> clerk fills EoD form (totals from kit + manual counts)
      -> PO signs with JWT + PIN
      -> DPO countersigns with JWT + biometric
      -> EndOfDayReport persisted; eel.clerk.eod_report.v1 emitted
      -> anchored in P05 tlog within 60 s of dual-sign
      -> eel.inventory.stock_confirmed.v1 emitted to M20 (ballot-paper reconciliation)
```

Transactional outbox via `packages/eel-outbox-rs`. Debezium to `eel.clerk.v1`.

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/poll-clerk.yaml`. All writes enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /clerk/v1/kit/status` | Clerk (JWT) | Kit status mirror from M06 |
| `POST /clerk/v1/fallback/lookup` | DPO (JWT + biometric) | Voter fallback lookup (permanently logged) |
| `POST /clerk/v1/paper-backup/start` | PO (JWT) | Start paper backup workflow |
| `POST /clerk/v1/paper-backup/end` | PO + DPO (JWT + biometric) | End paper backup; log ballot count |
| `POST /clerk/v1/eod-report` | PO + DPO (JWT + HSM) | Submit dual-signed EoD report |
| `GET /clerk/v1/eod-report/{id}/proof` | Commission admin | P05 inclusion proof |
| `GET /clerk/v1/kiosk/register` | Public (kiosk mode) | Station register inspection (hash lookup only) |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Flutter app (clerk) | `services/field-mobile-svc/mobile/poll-clerk/` | Android 8+ / iOS 14+; offline-capable; kiosk mode; EN + SW |
| ECC console (M23 integration) | `services/ecc-svc/ex/ecc_web/` | Kit failure and fallback-lookup alerts visible to ECC |
| Commission admin (Django) | `services/field-mobile-svc/django/clerk-admin/` | EoD report archive, fallback-lookup audit |

Languages: EN + SW. Large-text mode for accessibility. Kiosk mode tested with no back-navigation path.

## 7. Integrity + anti-tamper

- **DPO biometric gate on fallback lookup** -- every fallback lookup requires a live DPO biometric; the `biometric_ref_hash` is stored permanently and immutably; no fallback lookup can be erased from the audit log.
- **No voter authentication in M31** -- M31 has no path to mark a voter as having voted; it can only read registration data in fallback mode. Authentication remains exclusively in M05.
- **Read-only kit mirror** -- `KitStatusMirror` is a read-only gRPC subscription to M06; M31 cannot write to the BVI kit.
- **Dual-sign EoD report** -- PO + DPO signatures required; single-signer submission rejected server-side; HSM-signed report anchored in P05.
- **Paper-backup immutability** -- `PaperBackupEvent` rows append-only; reason code and timestamps immutable after creation.
- **national_id_hash in fallback** -- raw national_id never stored or transmitted; SHA-256 hash derived on device; hash logged alongside DPO ID for audit.
- **Kiosk mode isolation** -- kiosk screen launched in Android Single App Mode / iOS Guided Access; no access to other app functions.

## 8. Kenya statute alignment

- **Elections Act s.44A (four-eyes)** -- EoD report dual-sign by PO + DPO satisfies four-eyes obligation for station closing record.
- **Elections Act s.72 (secrecy)** -- fallback lookup returns registration status only; no ballot or vote data; national_id_hash ensures no linkage to voting record.
- **Kenya Data Protection Act s.26 (minimisation)** -- national_id_hash only in FallbackLookupLog; DPO biometric_ref_hash (not raw biometric) stored.
- **Elections Act s.80 (closing procedures)** -- EndOfDayReport with dual-sign and P05 anchoring satisfies the statutory station-closing certification.

## 9. Honest caveats + external prerequisites

- The fallback lookup requires M02 read-replica connectivity. If M02 is unreachable and the kit is also down, the clerk has no electronic voter lookup. The paper register (printed pre-election from M02) is the offline fallback; M31 must display its location and usage instructions.
- DPO biometric device must be on the same local network as the clerk app or have internet access at fallback-lookup time. A DPO whose device has no connectivity cannot unblock the fallback. The station must have at least one device with connectivity for this path.
- End-of-day dual-sign requires both PO and DPO to have their devices powered and connected. Battery planning is an operational responsibility; M31 shows a low-battery warning below 20%.
- Kiosk mode depends on Android MDM or iOS Supervised Device enrolment for production hardening. Testing on unmanaged devices shows the back button remains active; MDM enrolment must be confirmed in the device procurement contract.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/crates/poll-clerk-api/tests/` and `mobile/poll-clerk/integration_test/`:
- Unit: DPO biometric gate, national_id_hash, EoD dual-sign enforcement, kiosk mode isolation, paper-backup event immutability.
- Property (`proptest`): fallback lookup log append-only; EoD report rejects single-signer; no raw national_id in any log.
- Contract (Pact): M06 kit status consumer; M02 voter lookup consumer; M20 stock reconciliation producer.
- Device: kiosk mode back-button test; low-battery warning; offline paper-backup guide availability.
- Security: fallback lookup without DPO biometric; EoD single-signer bypass; kiosk escape; OWASP ZAP.

Runbooks under `docs/runbooks/`:
- `kit-failure-fallback.md` -- fallback mode activation, DPO attestation, ECC notification, paper-register use.
- `eod-report-dual-sign.md` -- PO/DPO ceremony, connectivity requirements, P05 anchoring, late submission.
- `kiosk-mode-setup.md` -- MDM enrolment, Single App Mode, reset procedure.

Related: full spec at `docs/plans/modules/mobile-poll-clerk.md`; capability catalogue at `services/field-mobile-svc/docs/CAPABILITIES.md`.
