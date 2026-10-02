# Poll Clerk App — Complete Reference

**Module:** M31 (Mobile Poll Clerk) · **Plan:** [`docs/mobile/plan.md` §5.1](plan.md) · **Status:** M2 complete, M3/M4/M5 pending
**Code:** [`services/field-mobile-svc/apps/poll-clerk/`](../../services/field-mobile-svc/apps/poll-clerk)
**Contract:** `packages/openapi-contracts/field/poll-clerk.yaml` (9 paths) · **BFF:** `mobile-bff §4.1`

The Poll Clerk app is the primary field tool used by the Presiding Officer (PO) at a polling station on election day. It runs on a kiosk-locked Android device provided by the Commission, and is the authoritative interface for queue management, voter identity verification, incident escalation, and end-of-day reporting. The app is the technical enforcement point for the **DPA 2019** lawful-basis and the **Elections Act** identity-verification chain — every action it takes is biometrically attested, audit-logged, and reconciled against the BVI back-end.

---

## Roles

| Role | SpiceDB permission | Allowed screens |
|---|---|---|
| `CLERK` | `clerk.operate` | All operator screens |
| `DPO` | `clerk.attest_lookup` | Lookup attestation, EOD dual-signature |
| `KIOSK_OPERATOR` | `clerk.kiosk` | BVV lookup, citizen-facing kiosk screens only |
| `SUPERVISOR` | `clerk.elevate` | (read-only dashboard) |

---

## Features

### F1 — Bootstrap + MFA Sign-In (FR-001–003)

On first launch the app calls `M3BootstrapService.bootstrap()` which loads:
- the **policy bundle** (rate limits, allowed categories, escalation chain),
- the **content bundle** (manual, forms, Elections Act clauses, contacts),
- **BVI station status** (BFF `/v1/mobile/bff/station/{id}/status`),
- **device attestation verdict** (EelAttestationClient → Play Integrity / Key Attestation).

Sign-in is **two-factor**: (1) OIDC PKCE against Keycloak (agent ID + password) returns an OAuth code, which the BFF exchanges for an app JWT; the exchange requires an attestation verdict >= `WARN`. (2) Biometric prompt (`local_auth`) on every foreground-resume. A **5-fail PIN lockout** is enforced in secure storage (encrypted shared prefs) and persists across app restarts (30-min lockout per ADR-0018).

If attestation fails (rooted device, emulator, debug build), the app is locked at the **HardAttestationFailedScreen** and shows a support code + the Commission support number. No guest mode exists.

### F2 — Reference Library (FR-004–007, 025)

Content bundle (manual EN+SW, Form 34A/B/C PDFs, Elections Act clauses, contact card) ships in `assets/content/` as a versioned, ed25519-signed archive. On first launch the bundle is loaded into an **SQLite FTS5 index** (BM25 search, ICU NFC normalization) which is queried for full-text search in ≤ 1 s on a mid-tier Android (AC-M31-05). PDF export uses the `pdf` package with a pinned template version. Updates only happen via signed MDM push — the app refuses any unsigned bundle, and keeps the N-1 version as a fallback.

### F3 — Station Dashboard (FR-008–009, 027)

The dashboard mirrors BVI back-end state (turnout, auth count, kit battery, connectivity, GPS fix) via a **WebSocket** to BFF `/api/v1/mobile/bff/events` filtered by station, with a 30 s poll fallback when the socket is down. The screen is **strictly read-only** — there are no actions on the dashboard. State changes (turnout delta, kit-low warnings) are surfaced as `EelStatusChip` and `EelBanner` with last-good timestamp. Stale data is annotated with "as of HH:MM" and a stale-warn icon.

### F4 — Voter Register Lookup Fallback (FR-010–012)

When the BVV (biometric voter verification) system is unavailable, the poll clerk falls back to a **manual register lookup**. This is a sensitive operation: the clerk must (1) provide written justification in a textarea, (2) trigger a DPO biometric attestation prompt (different staff member, fresh biometric capture), and only then is the request submitted. The client enforces a **token-bucket rate limit** (capacity from policy bundle, refill continuous) per station-hour; the server is the authority (429 → `rate-limited` state with reset time). Offline → outbox `lookup_request` type with `pending-info` status; on completion push, the result banner appears with station + status only (no PII beyond necessary).

### F5 — Queue Management (FR-013–015)

Voter queue with **priority sub-queue** (elderly, pregnant, disabled). The `PriorityEngine` is a two-tier comparator: priority flag first, then ticket number. Tickets are issued with a numeric code; live position + ETA use an **EWMA service-time** (α=0.3, seeded 90 s). The "Call next" action is the primary CTA, requires the PO role, and is audit-logged locally + outboxed. A metrics panel (collapsible) shows served/hour, current wait, and SLA breaches.

### F6 — Structured Incident Reporting (FR-016–018)

Seven categories: `INTIMIDATION`, `BRIBERY`, `TURNED_AWAY`, `MATERIALS`, `KIEMS_DOWN`, `UNAUTHORIZED_PERSON`, `OTHER`. Free-text description, photo/video evidence (`image_picker` ≤ 60 s, 25 MB per file), SHA-256 + Isar blob storage. Upload to BFF `POST /api/v1/mobile/bff/incidents` with multipart + `Idempotency-Key: <incident uuid7>`. **Escalation FSM**: PO → RO → county → war-room; level is immutable once set > 0 (DB CHECK + UI lock); signature = Ed25519 over canonical JSON (RFC 8785 semantics).

### F7 — Paper-Backup Workflow (FR-019–020)

Step wizard (`EelStepFlow`, 7 steps) for KIEMS-down: paper auth steps, Form 34A guide, spoilt-ballot serial log, digital reconciliation checklist. Each step is a checklist item with optional serial-number input. The spoilt-ballot log is append-only. The reconciliation checklist creates an outbox item on completion.

### F8 — Kiosk Mode / BVV Assist (FR-021–022)

Role-gated route, only available to `KIOSK_OPERATOR`. **BVV register lookup** with staff-assisted OTP (the staff member fetches the OTP for the citizen via the BVI adapter). **Objection filing** uses a 3-screen consent capture: (1) DPA lawful-basis screen, (2) citizen oral consent checkbox + recording of consent metadata, (3) witness biometric (`local_auth`) + witness signature (signature pad → vector hash, raw discarded per kit pattern). All stored locally then outboxed to M07 path.

### F9 — End-of-Day Report (FR-023–024, 026)

`EodCompiler` reads local tables (check-in, incidents, spoilt log, agent presence) → draft. **Dual signature**: PO signs, DPO signs (two biometric prompts, both Ed25519 signatures recorded). Submit with UUID7 idempotency key. Queued offline. The screen blocks session close until confirmed (no accidental early-lockout).

### F10 — Security Envelope (FR-029–030, 032)

- **mTLS pinning** to BFF + Keycloak roots
- **Captive-portal UX**: detect & show explicit "sign in to Wi-Fi" screen rather than silent failure
- **FLAG_SECURE** on lookup + incident screens (no screenshots, no recents thumbnail)
- **SQLCipher/Isar** with hardware-key-protected master key
- **Tamper events** (root detection, debugger attach, signature mismatch) trigger outbox event + RASP heartbeat

### F11 — Battery Discipline (FR-028)

≤ 5 %/h foreground, ≤ 2 %/h background, Doze respected. Foreground service only runs for queue polling; sync uses WorkManager with constraints.

---

## Screen-by-screen UX

| Screen | Components |
|---|---|
| **Login** | `EelScaffold` (centered), `EelSealStamp` (64dp), `EelTextField` (agent ID, password), `EelButton` (primary sign-in), attestation verdict indicator |
| **Station Dashboard** | 2-column stat grid (turnout, auth, battery, connectivity), `EelStatusChip` 7-state matrix, `EelBanner` for degraded/offline, `EelSyncIndicator` top-right |
| **Reference Library** | Tab nav (Manual/Forms/Clauses/Contacts), search bar with `EelTextField`, results in `EelResultCard` with snippet highlight |
| **Lookup Fallback** | 3-step `EelStepFlow` (ID → justification → attestation); `EelTextField` for ID hash + justification; `EelBanner` shows rate-limit remaining |
| **Queue** | Single `ListView` with priority section pinned top, ticket dialog, EWMA ETA, call-next `EelButton` (bottom-right) |
| **Incident** | 7-category icon grid (48dp), `EelTextField` (description), `EelChips` for evidence, escalation selector with post-submit lock |
| **Paper Backup** | 7-step `EelStepFlow` wizard with checklist per step |
| **Kiosk** | High-contrast citizen mode; staff controls in app bar overflow; 3-screen consent capture |
| **EOD** | Auto-compiled preview with edit provenance; dual-signature sheet; receipt with UUID7 |
| **Settings** | Connection diagnostics, force-resync, language, oversized text toggle |

---

## State machine

`Unauthenticated → Bootstrapping → HardAttestationFailed | Authenticated → Active | Locked(PIN) | Banned(Tamper)`.

Within `Active`: `Idle` ↔ `OnShift` → `EodDraft` → `EodSigned` → `Closed`.
Escalation chain is `None → PO → RO → County → WarRoom`, each transition append-only.

---

## Algorithms

- **Token-bucket rate limiter** (station, hour): capacity = policy bundle limit; refill continuous; display remaining + reset.
- **EWMA service-time ETA** (α = 0.3, seeded 90 s).
- **BM25 search** over FTS5; snippet extraction; ICU normalization (NFC) before index.
- **Canonical JSON hashing** (RFC 8785) for signature payloads.
- **Backoff**: full jitter (master §2.5).

---

## Events emitted

- `clerk_checked_in`
- `clerk_incident`
- `end_of_day_report`
- `voter_lookup_audit` (with DPO attestation fingerprint)

---

## Dependencies

- `eel_design` — design system (`EelScaffold`, `EelButton`, `EelResultCard`, `EelStatusChip`, `EelBanner`, `EelSealStamp`, `EelTextField`, `EelChips`, `EelStepFlow`, `EelSecureScreen`, `EelLoadingState`)
- `eel-core-flutter` — session, attestation, secure store, outbox, idempotency
- `flutter_appauth` — OIDC PKCE
- `local_auth` — biometric
- `mobile_scanner` — barcode/QR
- `image_picker`, `video_compress`, `pdf`, `printing` — evidence + forms
- `sqlite3` (FTS5), `isar` (encrypted), `flutter_secure_storage`
- `connectivity_plus`, `web_socket_channel`, `dio`

---

## Testing

- **Unit**: priority engine ordering, rate limiter, EOD aggregation, search benchmark, backoff, ETA, canonical JSON.
- **Widget**: state-matrix goldens for every screen, text-scale 2.0, hit-target 48dp.
- **Integration**: check-in → lookup → queue → incident → EOD happy path; offline incident + reconnect dedup; background-kill at each FSM state; JWT mid-expiry silent refresh.
- **Contract**: Pact for 9 poll-clerk paths + BFF paths.
- **Security**: mitmproxy captive, rooted AVD block, forensic extraction shows no plaintext.
- **Load**: k6 92k check-ins/60 min vs BFF.
- **A11y**: TalkBack scripts for queue + incident.

---

## Definition of done

All FR-001…032 in module spec M31 ticked; `poll-clerk.yaml` bound to BFF; allowlist entries for any placeholders removed; README rewritten to match reality; ODPC kiosk-review note tracked as open question with owner; AC-M31-01…15 rows ticked with test IDs.
