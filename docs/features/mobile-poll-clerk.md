# Mobile poll clerk (M31)

Module M31 (`mobile-poll-clerk`) owned by `field-mobile-svc`. A Flutter Android/iOS companion app for Presiding Officers (DPOs) and Deputy Presiding Officers. Mirrors the M20 kit dashboard at station level, surfaces M25 special-workflow prompts, provides offline reference materials and voter-lookup fallback, guides paper-backup workflows when electronic systems fail, supports kiosk mode for shared-device use, and produces a dual-signed EOD report submitted to M09.

## Contents

1. What mobile poll clerk does (vs related modules)
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

## 1. What mobile poll clerk does (vs related modules)

- **M20 Inventory management** = authoritative kit custody chain; M31 mirrors kit status for the DPO but does not write custody events directly (it calls M20 via API).
- **M25 Special voter workflows** = defines category-specific workflow rules; M31 renders the prompts and collects officer signatures.
- **M31** = the DPO's operational hub for polling day: kit status, voter check-in assist, special workflows, paper backup guidance, and EOD sign-off.

M31 writes to M09 (EOD report), M20 (kit arrival event via REST), and M25 (assisted vote signatures). All writes are offline-queued and synced on connectivity.

## 2. Data plane

Five local aggregates in the Flutter Isar+SQLCipher store:

| Aggregate | Purpose | Sync cadence |
|---|---|---|
| **KitStatusMirror** | M20 kit status for this station: unit_id, type, status, last_synced_at. | Pull every 5 min; push on M20 event |
| **VoterLookupLog** | Fallback voter lookup: voter_ref_hash, outcome (FOUND/NOT_FOUND), fallback_used (bool), ts_hlc. | Sync on reconnect |
| **SpecialWorkflowLog** | M25 workflow event: category, voter_ref_hash, outcome, officer_sig, ts_hlc. | Immediate sync |
| **PaperBackupGuidance** | Offline reference: procedure_id, title, content_markdown, version. | Pre-election bulk; delta push |
| **EODReport** | End-of-day report: station_id, total_voters, total_ballots_issued, discrepancies (JSON), dpo_sig, deputy_sig, ts_hlc. | One-time sync at EOD |

## 3. Ingest sources

| Source | Cadence | Format | Auth | Failure semantics |
|---|---|---|---|---|
| M20 kit status push | Every 5 min + on event | Kafka event -> push | Device cert | Cached KitStatusMirror served offline with staleness badge |
| BVR voter lookup (M02) | On demand | REST | Device cert | FALLBACK if offline: DPO uses paper roll; VoterLookupLog(fallback_used=true) |
| M25 special workflow definitions | Pre-election bulk | Signed JSON | Device cert | Cached; stale if >7 days |
| Paper backup guidance | Pre-election bulk | Signed Markdown | Device cert | Bundled in app binary for hard-offline scenario |
| M09 EOD submission | At station close | REST | Device cert | Retry until received; EOD blocked until M09 acks |

## 4. Pipeline

```
station open:
  DPO logs in (device cert + PIN)
  -> KitStatusMirror pulled from M20
  -> special workflow definitions confirmed current
  -> kiosk mode activated (locks app to poll clerk screen)

voter check-in assist:
  DPO searches voter by name or national ID
  -> BVR lookup via REST (or paper roll fallback)
  -> VoterLookupLog created; fallback_used flagged if offline

special workflow:
  DPO selects category (REMAND / REFUGEE / IDP / DISABILITY / HOSPITAL)
  -> M25 workflow prompts rendered:
       DISABILITY: four-eyes prompt -> assistant selection -> officer_sig
       IDP: double-cast guard check via M25 API
       REMAND: dual-biometric unlock prompt (two officer PINs)
  -> SpecialWorkflowLog persisted; synced to M25

kit event (arrival / discrepancy):
  DPO confirms kit arrival or logs discrepancy
  -> M20 REST call: POST /inventory/v1/events
  -> KitStatusMirror updated

paper backup guidance:
  network failure triggers guidance banner
  -> DPO selects failure type (BVR offline / biometric failure / power loss)
  -> PaperBackupGuidance rendered from local Isar store

EOD:
  DPO enters final counts; reviews discrepancies
  -> EODReport drafted; DPO signs (dpo_sig)
  -> Deputy Presiding Officer countersigns (deputy_sig) on same device
  -> EODReport submitted to M09 via REST
  -> eel.pollclerk.eod_submitted.v1 emitted; P05 Merkle anchor within 60 s
```

## 5. API surface

Contract: `packages/openapi-contracts/field-mobile/poll-clerk.yaml`. All write endpoints enforce `Idempotency-Key`.

| Method + path | Persona | Purpose |
|---|---|---|
| `POST /pollclerk/v1/voter-lookup` | DPO device (cert) | BVR voter lookup (logged) |
| `POST /pollclerk/v1/workflow-events` | DPO device (cert) | Log special workflow event |
| `POST /pollclerk/v1/eod-reports` | DPO device (cert, dual-sig) | Submit EOD report to M09 |
| `GET /pollclerk/v1/kit-status/{station_id}` | DPO device (cert) | Kit status mirror from M20 |
| `GET /pollclerk/v1/guidance/{type}` | DPO device (cert) | Paper backup guidance content |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| Poll clerk Flutter app (Android/iOS) | `services/field-mobile-svc/flutter/poll-clerk/` | All DPO functions; kiosk mode; offline-first |
| EOD review screen (within Flutter app) | Same app | Final count entry; dual-sig capture |

Languages: EN + SW. Material You accessibility; minimum touch target 48dp; high-contrast mode. WCAG 2.2 AA on all screens. Kiosk mode prevents app-switching during polling hours.

## 7. Integrity + anti-tamper

- **Dual-signed EOD report** -- both DPO and Deputy sign before the EODReport is submitted; a single-signer submission returns HTTP 403; the server verifies both signatures against the station officer roster.
- **Offline VoterLookupLog** -- every lookup is logged whether online or offline; fallback paper roll use is logged as fallback_used=true and is visible to the IAB audit dashboard; the IAB can compare paper roll usage rates across stations.
- **Kiosk mode** -- once activated, the app cannot be closed or switched without a supervisor PIN; this prevents the DPO's device from being used for non-election activities during polling.
- **Guidance content integrity** -- paper backup guidance is signed at the time of the pre-election bulk push; the app verifies the signature before rendering; unsigned guidance is not displayed.
- **EOD P05 anchor** -- the EOD report hash is anchored in the P05 transparency log within 60 s of M09 ack; courts can verify the report has not been altered since submission.

## 8. Kenya statute alignment

- **Elections (General) Regulations 2012 r.55 (DPO duties)** -- M31 implements the statutory DPO toolset: voter check-in assist, special workflows, and EOD report.
- **Elections Act s.44A (assisted voting)** -- four-eyes SpecialWorkflowLog with officer_sig satisfies the statutory requirement for assisted vote documentation.
- **DPA 2019 s.26 (minimisation)** -- VoterLookupLog stores voter_ref_hash only; raw national IDs are not persisted; EODReport stores aggregate counts, not individual voter records.
- **Elections (Technology) Regulations 2017** -- dual-signed EOD report retained for 7 years; offline fallback logging retained for audit.

## 9. Honest caveats + external prerequisites

- Kiosk mode effectiveness depends on the device's Android/iOS kiosk API; some Android device manufacturers override kiosk mode through proprietary system apps; device procurement must verify kiosk API compliance before deployment.
- The BVR offline fallback requires a pre-loaded encrypted voter roll subset on the device; the roll subset size is bounded by the station's expected voter count; large stations (>2 000 voters) may exceed the KES-10k device storage budget.
- The dual-sig EOD flow requires both DPO and Deputy to be physically present with the device; if the Deputy is absent, the EOD report cannot be submitted; the IEBC field operations must define the escalation path for this scenario.
- Paper backup guidance content must be reviewed and approved by IEBC legal and operations before each election cycle; outdated guidance creates a compliance risk.

## 10. Test coverage + operational runbooks

Coverage under `services/field-mobile-svc/flutter/poll-clerk/test/` and `services/field-mobile-svc/crates/poll-clerk-api/tests/`: unit tests for kit mirror, voter lookup fallback logger, special workflow renderer, dual-sig EOD builder; widget tests for all screens including kiosk mode; integration tests with M20 stub, M25 workflow stub, M09 EOD ack stub, SoftHSM dual-sig; e2e Flutter integration test covering full polling day flow from station open through special workflows, paper fallback, and EOD dual-sig; chaos test for BVR offline during voter lookup.

Runbooks under `docs/runbooks/`:

- `poll-clerk-degradation.md` (create): BVR offline; M20 kit sync fail; biometric failure; EOD submission retry.

Dashboards: `platform/observability/field-mobile-svc/grafana-pollclerk.json` (voter lookup rate, fallback rate, special workflow count, EOD submission status). SLO: 99.9% EOD report submitted within 30 min of station close; kit mirror sync lag <10 min.

Kiosk mode recovery: if the kiosk mode PIN is lost, recovery requires a county coordinator to physically present their JWT-enrolled device and enter a time-limited recovery code; there is no remote recovery path; the PIN recovery process is part of the pre-election officer training.

Special workflow double-cast guard for IDP (M25): when M31 triggers an IDP workflow, the double-cast guard query goes to M25 API; if M25 is unreachable, the IDP workflow is blocked with a logged `GUARD_UNAVAILABLE` event; the DPO cannot bypass the guard without a commissioner emergency override (HSM-signed).

EOD report retention: EndOfDayReport and its dual-signature are retained for 7 years per Elections (Technology) Regulations 2017; the P05 anchor provides court-admissible proof of submission time independent of the governance-svc database.

VoterLookupLog IAB visibility: the fallback paper-roll usage rate per station is visible in the IAB audit dashboard; stations with fallback usage > 20% of total lookups are automatically flagged for post-election inquiry; the flag does not affect same-day results.

Related: `docs/plans/modules/mobile-poll-clerk.md`; `services/field-mobile-svc/docs/CAPABILITIES.md`.
