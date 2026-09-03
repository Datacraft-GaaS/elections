# Mobile Applications Master Implementation Plan

Status: plan of record for completing every eel mobile app. Companion audit:
`docs/mobile/status.md`. Per-app plans live in this directory (`<app>.md`), one per
app; this file is the master. The implementing agent MUST read this file first,
then the per-app plan for the app being worked, then the module spec
(`docs/plans/modules/<module>.md`) and the linked OpenAPI contract before writing code.

Authoring date: 2026-09-03. Owner: field-mobile platform team. Effort totals are
in engineer-days (ED) assuming one agent working sequentially; per-app files carry
their own estimates.

---

## 1. Ground truth: what exists and what must change

18 Flutter apps under `services/*/apps/` plus the SDK. Current verdicts (evidence
in `docs/mobile/status.md`):

| App | Module | Tier | Verdict | Plan file |
|---|---|---|---|---|
| packages/kit-sdk-flutter | shared SDK | — | production-grade, needs attestation/pinning hardening | kit-sdk-flutter.md |
| packages/kit-sdk-flutter/example | SDK demo | — | minimal | kit-sdk-flutter.md |
| services/bvr-svc/apps/cvr-field-officer | M35 | T2 | production-grade, gaps | cvr-field-officer.md |
| services/field-mobile-svc/apps/personnel | M33 | T1 | production-grade, gaps | personnel.md |
| services/inventory-svc/apps/custody-mobile | M65 | T2 | production-grade, backend unbound | custody-mobile.md |
| services/rts-svc/apps/rts-kit | M08 | T1 | production-grade, SMS fallback stub | rts-kit.md |
| services/training-svc/apps/lms-offline | M38 | T3 | production-grade, API path mismatch | lms-offline.md |
| services/training-svc/apps/accred-scanner | M42/43 | T3 | production-grade, CRL endpoint missing | accred-scanner.md |
| services/biometric-edge-svc/apps/bvi-kit | M05 | T1 | partial (simulated NTS/GNSS/capture) | bvi-kit.md |
| services/biometric-edge-svc/apps/diaspora-kit | M24 | T2 | partial (fabricated HSM signature) | diaspora-kit.md |
| services/biometric-edge-svc/apps/bvr-kit | M01 | T1 | partial (MRZ/IPRS/sign/upload stubs) | bvr-kit.md |
| services/field-mobile-svc/apps/poll-clerk | M31 | T2 | scaffold (4/5 screens stub, auth fake, attestation stub) | poll-clerk.md |
| services/field-mobile-svc/apps/observer | M34 | T2 | scaffold (4/4 screens stub) | observer.md |
| services/field-mobile-svc/apps/party-agent | M30 | T2 | scaffold (4/4 screens stub) | party-agent.md |
| services/field-mobile-svc/apps/field-support | M32 | T2 | scaffold (4/4 screens static) | field-support.md |
| services/field-mobile-svc/apps/citizen-verify | M13/M29 | T1 | **does not exist**; module spec requires it | citizen-verify.md |
| services/ai-platform-svc/apps/kit-l1 | M26/28 | T2 | anomaly plugin, needs integration | kit-l1.md |
| services/biometric-edge-svc/apps/legacy-android | WP-3.8 | T3 | shim, dead buttons | legacy-android.md |
| services/hello-svc/apps/hello-flutter | CI validator | — | intentionally minimal | hello-flutter.md |

Two structural truths drive everything below:

1. **The field-mobile persona apps have no real backend path.** `mobile-bff`
   (`services/field-mobile-svc/crates/mobile-bff`) serves echo stubs
   (`src/spec.rs`), and 18 of its OpenAPI paths are unbound (allowlist entries,
   `docs/plans/deferral-plan.md`). No app work is "done" until its endpoints are
   real and Pact-verified.
2. **The repo's own gates must pass.** `tools/completeness scan`, `just ci`,
   `docs/plans/11-completeness-audit.md` steps 9.1–9.9, and TRACEABILITY
   coverage. Every work item below is scoped so its completion moves these gates.

---

## 2. Target architecture (all apps)

### 2.1 Layered structure (standard for every app)

```
services/<svc>/apps/<app>/
  lib/
    main.dart                  # bootstrap only
    l10n/                      # generated ARBs (flutter gen-l10n)
    src/
      core/
        bootstrap.dart         # startup FSM (§2.4)
        session.dart           # auth state, token lifecycle, biometric gate
        api_client.dart        # EelHttpClient factory: pins, retries, idempotency
        local_db.dart          # Isar + SQLCipher schema, WAL-journal helper
        outbox.dart            # typed outbox over kit-sdk queue; drain loop
        connectivity.dart      # connectivity_plus + captive-portal probe
        app_config.dart        # MDM/config fetch + --dart-define overrides
        tamper.dart            # RASP + screen-capture + tamper events
      features/<feature>/
        <feature>_screen.dart
        <feature>_controller.dart   # Riverpod Notifier (immutable state)
        <feature>_repository.dart   # API + cache + outbox composition
      widgets/                 # app-local widgets composed from eel_design
      app_router.dart          # go_router, guarded redirects
  assets/
    config/<flavor>.json       # endpoints, feature flags, pin set id
    content/                   # manuals, forms, runbooks (signed bundles)
  android/  ios/               # native shells + platform channels
  test/                        # unit + widget (§6)
  integration_test/            # device tests (§6)
  test/ (golden/               # golden files)
```

Rules:
- Riverpod (`hooks_riverpod`) is the only state management. No inherited widgets
  for state, no global mutable singletons except `main()` bootstrap.
- `go_router` with a `redirect` that reads the bootstrap/auth FSM; deep links
  only within the app's own scheme.
- All networking through kit-sdk `EelHttpClient` (dio). No bare `Dio()` or
  `http.post` outside `core/api_client.dart`. (Today cvr-field-officer,
  accred-scanner, lms-offline, field-support violate this — fix during M2/M3.)
- All persistence through `core/local_db.dart` (Isar collections declared once;
  SQLCipher key from `EelSecureStore`, hardware-backed keystore).
- All writes that matter go through the outbox: journal-before-act (§2.5).

### 2.2 Shared packages (build in Wave M0)

| Package | Purpose | Contents |
|---|---|---|
| `packages/design-system/flutter/` (new: `eel_design`) | Flutter consumption of `packages/design-system/tokens.json` | code-gen'd `EelTheme` (light+dark) from tokens.json via build script; components listed §5.2 |
| kit-sdk-flutter (extend) | attestation, crypto, queue, telemetry, FFI | real Play Integrity/App Attest clients, SPKI-pinning adapter, `EelEncryptedIsarStore`, push abstraction, polygon geo utils (already has geolocator) |
| `packages/eel-core-flutter/` (new) | cross-app composition | bootstrap FSM, session/auth (OIDC PKCE + `local_auth` biometric), config, connectivity, tamper, outbox drain, evidence upload, PDF render, search index, problem+json parsing |

kit-sdk stays device-tier; eel-core-flutter is app-tier. Both pure Dart except
documented platform channels.

### 2.3 Data and identity contracts

- IDs: UUIDv7 generated client-side (`uuid` v4.5 `Uuid().v7()`), validated
  server-side; used as `Idempotency-Key` on every POST/PATCH.
- Time: kit-sdk HLC (`eel_hlc`) attached to queued records; server timestamps
  authoritative for audit display (EC-MOB-08); client clock informational only.
- Events: CloudEvents 1.0 with the `eel_*` envelope exactly as
  `packages/event-schemas/**/v1.schema.json` (see
  `field_mobile/tamper/device_tamper_detected/v1.schema.json` for the shape).
  Apps never publish to Kafka directly; the BFF does, via the transactional
  outbox (invariant 1 in `README.md`).
- Errors: RFC 9457 `problem+json` end-to-end; app parses `type`, `title`,
  `status`, `detail`, `instance` into `ApiProblem` and maps to localized copy.

### 2.4 Startup FSM (every app; implement once in eel-core-flutter)

```
cold → integrityCheck → attestation → config → auth → roleRoute → ready
              │              │            │        │
              ▼              ▼            ▼        ▼
        blocked(rasp)  blocked(attest)  blocked  blocked(auth)
        + tamper event + support code
```

- `integrityCheck`: root/jailbreak/emulator/Frida/Xposed heuristics; on fail,
  emit `eel.field_mobile.device_tamper_detected.v1` (best effort, from a
  one-shot unpinned channel) then hard-block. AC-M31-15 style: block ≤ 2 s.
- `attestation`: kit-sdk client (§3.1); verdict cached 15 min with device-key
  binding; `MEETS_NONE`/root → hard fail for staff roles (FR-M31-003,
  FR-M30-003, FR-M33-004 supervisor tier; citizen app is soft-warn).
- `config`: GET `/api/v1/mobile/bff/config` (bff.yaml `getAppConfig`); returns
  per-app feature flags, rate-limit policy bundle, version floor, pin-set id,
  station/cluster binding. Cached last-good in secure store; offline uses cache.
- `auth`: OIDC Authorization Code + PKCE against Keycloak (realm `eel`), then
  local step-up: platform biometric (`local_auth`) + 6-digit PIN; 5 PIN fails →
  30-min lockout (persisted, monotonic). Attestation verdict is bound into the
  session token request (backend rejects mismatch).
- `roleRoute`: SpiceDB-declared role from token → route table (below).

### 2.5 Offline-first write path (journal-before-act)

Every state-changing user action follows:

1. Serialize record + HLC + UUID7 → append to Isar `outbox` collection
   (`state=pending`, previous-hash chained). This is the WAL moment (EC-MOB-01).
2. Mutate UI state optimistically.
3. Drain loop (foreground timer + connectivity trigger): send with
   `Idempotency-Key`; on 2xx mark `synced`; on 4xx (except 408/429) mark
   `failed` + user-visible; on network/5xx mark `pending` and schedule retry.
4. Retry policy: exponential backoff **with full jitter**, base 1 s, factor 2,
   cap 60 s, per-entry budget 8 attempts then `dead` + surfaced in a sync panel.
5. Server dedup: backend stores key→response for 7 days; re-send returns cached
   201 (EC-DAT-01); same key + different body → 422 surfaced as conflict
   (EC-DAT-02).
6. Queue bounds: per-type capacity (default 5 000) with LRU-by-type eviction
   only for `content` types, never audit types; storage-full → refuse evidence
   uploads, keep text (EC-MOB-02).

### 2.6 Connectivity UX

- States: online / degraded(2G) / offline / captive-portal / untrusted(TLS fail).
- Probe: after `connectivity_plus != none`, HEAD `https://<bff>/healthz` through
  the pinned client; TLS pin failure → "network blocked" screen, zero tokens
  emitted (EC-CON-06). Captive-portal detection = pin/DNS failure pattern →
  "network blocked" banner with instructions (FR-M31-029).
- 2G: banner "limited connectivity"; large uploads ask for consent (EC-CON-03).

---

## 3. Security architecture (all apps)

### 3.1 Device attestation

- kit-sdk gains `PlayIntegrityAttestationClient` (Android) and
  `AppAttestAttestationClient` (iOS) behind the existing `EelAttestationClient`.
- Android: Play Integrity classic request → backend verdict endpoint
  `POST /api/v1/mobile/bff/attestation/verdict` (new; binds nonce, device
  integrity verdict, license) → signed verdict cached by the app.
- iOS: DeviceCheck App Attest: generate key → attestation object → backend
  verifies with Apple → assertion on each subsequent attest.
- Emulator/dev: `EelAttestationClientStub` only compiled into `dev` flavor.
- The three persona-app `AttestationService` stubs (always-pass) are **deleted**
  and replaced by the kit-sdk client; auth blocks on failure (security P0).

### 3.2 Runtime self-protection (RASP)

- Checks: root indicators (su binaries, magisk), emulator props, debugger
  attach (`TracerPid`), Frida (named pipes/ports/maps), Xposed, re-sign
  (signature check vs embedded cert SHA-256 at startup).
- Reaction: session terminate, wipe in-memory keys, tamper event (schema
  above), support code screen. Tested on rooted AVD (AC gate).

### 3.3 TLS pinning

- kit-sdk `EelHttpClient` gains a custom `HttpClientAdapter` doing SPKI pinning
  against a pin-set fetched in config and mirrored in
  `assets/config/pins-<set>.json`; dual-pin window for rotation (2 pins valid
  during overlap; `rotate-pins.sh` manages).
- Pin failure → connection refused, `connectivity.untrusted` state, no
  token headers. Verified with mitmproxy rogue-CA harness (AC-M31-07).

### 3.4 Local data at rest

- Isar collections opened with AES-256 key from `flutter_secure_storage`
  (hardware keystore / Secure Enclave). kit-sdk ships `EelEncryptedIsarStore`
  that wraps key lifecycle + schema migration (Isar 3.x).
- FLAG_SECURE (Android) / `UIScreen.isCaptured` observation (iOS) wrapper
  widget `EelSecureScreen` applied to: incident, lookup, kiosk, countersign,
  diagnostics, tally-verify screens (per-module FRs).
- No PII in logs: kit-sdk redactor is mandatory on all telemetry; a CI
  log-scanner fixture enforces (EC-SEC-16).

### 3.5 Push notifications

- FCM (Android) / APNs (iOS). Payloads carry **only** event type + record id;
  content fetched authenticated on tap (FR-M30-025, FR-M33-005).
- Token registration endpoint: `POST /api/v1/mobile/bff/push-tokens` (new in
  bff.yaml); re-registration on app foreground if 410 (EC-MOB-05).

---

## 4. Backend integration plan (field-mobile-svc first)

### 4.1 mobile-bff: replace echo handlers with real ones

`services/field-mobile-svc/crates/mobile-bff/src/spec.rs` currently returns
`json!({...})` echoes. Implementation order (all endpoints then bind their
OpenAPI paths, removing allowlist entries):

| Endpoint group | Handler work | Upstream |
|---|---|---|
| `/api/v1/mobile/bff/config` | per-app config: feature flags, policy bundle (rate limits), version floor, pin-set id, station binding | Postgres config store + ECC policy bundle |
| `/api/v1/mobile/bff/sessions` + `/refresh` | OIDC code exchange (Keycloak), attestation verdict verification, SpiceDB role check, issue app session JWT (5-min TTL + refresh) | Keycloak, SpiceDB |
| `/api/v1/mobile/bff/voter/verification` | register lookup fallback; DPO attestation hash + justification required; per-station token bucket; emit `eel.field.voter_lookup_audit.v1` | bvr-svc registers, outbox |
| `/api/v1/mobile/bff/station/status` | dashboard mirror: turnout, kit battery, connectivity, queue depth (no PII) | bvi-svc turnout/identifications |
| `/api/v1/mobile/bff/forms/sync` | signed Form 34A fetch + Merkle proof pass-through | rts-svc, transparency-svc |
| `/api/v1/mobile/bff/incidents` | incident ingest; idempotency cache; emit `incident_reported` via outbox (schema exists) | outbox → Kafka |
| `/api/v1/mobile/bff/dispatch/assignments[/{id}]` | proxy dispatch-api with role scope | dispatch-api |
| `/api/v1/mobile/bff/observer/access-pass|observations` | accreditation check; observation ingest | training-svc accreditation, transparency |
| `/api/v1/mobile/bff/tamper-events` | tamper ingest → M28 AI monitoring + audit shadow | outbox |
| `/api/v1/mobile/bff/push-tokens` | FCM/APNs token registry (new path, add to bff.yaml) | push router |
| SSE `/api/v1/mobile/bff/events` (exists in router.rs) | keep; used by dashboards | internal broadcast |

Requirements for every handler: SpiceDB permission check, UUID7 idempotency
store (Postgres, 72 h), problem+json errors, OTel spans, PII wrappers (tier
typed), rate limits from policy bundle, contract tests per path
(Pact + Karapace), idempotency tests, DAST pass (audit step 9.5).

### 4.2 Contracts to author/fill

| File | Action |
|---|---|
| `packages/openapi-contracts/field/poll-clerk.yaml` | create (referenced by M31; today the dir does not exist). Paths = the 9 `/v1/clerk/*` endpoints in the module spec |
| `packages/openapi-contracts/field/party-agent.yaml` | author to match M30 table (10 paths) |
| `packages/openapi-contracts/field/observer.yaml` | author to match M34 table (11 paths) |
| `packages/openapi-contracts/field/field-support.yaml` | author to match M32 table (10 paths) |
| `packages/openapi-contracts/field/citizen-verify.yaml` | author to match M13/M29 (see citizen-verify.md) |
| `packages/openapi-contracts/field-mobile-bff/{openapi,bff}.yaml` | bind 18 paths as handlers land; delete allowlist rows with expiry |
| `packages/openapi-contracts/custody/openapi.yaml` | bind 9 paths in inventory-svc |
| `packages/openapi-contracts/cvr/register.yaml` | bind 4 paths (field-notifications already scaffolded) |

Rule from status.md item 14: bindings are only added where handlers are real —
never fabricate.

### 4.3 Other backends touched

- **training-api**: add `GET /api/v1/training/accreditation/crl` (badge CRL
  bundle, signed) and align LMS submission path
  `/api/v1/training/lms/assessments/{assessment_id}/submissions` (fix
  lms-offline client which posts `/v1/assessments` today).
- **inventory-svc**: custody endpoints (packing events, container registry,
  seal verification) + spares lookup used by field-support.
- **rts-device-api**: diaspora deferred-batch upload endpoint
  (`/api/v1/rts/device/diaspora-batch`) + real SMS-fallback status responses.
- **bvi-svc**: kiosk register lookup endpoint for poll-clerk kiosk mode (M07
  BVV adapter); observer station polygon endpoints.
- **Push router** (new crate `push-router` in field-mobile-svc): FCM/APNs
  senders, topic per role, no-PII payload enforcement (schema-validated).

---

## 5. UX / UI / ergonomics standard

### 5.1 Principles (from repo design-system meta)

- WCAG 2.2 AA always; colour never the only signal; colour-blind-safe palette
  from tokens (already `wcag: AA` tagged).
- Field-first ergonomics: primary action within thumb arc on one-handed
  phones; 48 dp minimum touch targets (tokens `touch.min` = 44 px → enforce 48
  dp in Flutter per module FRs); reachability map per screen documented.
- Offline is normal, not error: every screen has an explicit offline state;
  queues are visible ("3 items waiting · last sync 14:32").
- Destructive/irreversible actions (dispute, close, freeze) require
  confirm + biometric step-up; show what will happen before it happens.
- Error copy: what happened, what the user should do now, error id for
  support. Localized from `packages/i18n` `errors.json`.

### 5.2 eel_design component library (Flutter)

Build these once; apps must not hand-roll:

- `EelTheme` (tokens.json → ThemeData + ThemeExtension: spacing 4/8 grid,
  radius, motion durations, focus ring width/colour, dark mode).
- `EelScaffold` (safe-area + banner slot + bottom action bar slot).
- `EelButton` (primary/secondary/destructive/ghost, loading, 48 dp),
  `EelTextField` (label always visible, error slot, helper), `EelPinField`,
  `EelSelect`, `EelSearchField`.
- `EelStatusChip` (severity/connectivity), `EelSlaTimerChip` (port from
  field-support, generalize), `EelSyncIndicator` (queue depth + last sync),
  `EelBanner` (info/warn/critical, accessible label), `EelSealStamp` (port),
  `EelVoiceGuideButton` (port), `EelMinHitTarget` (kit-sdk).
- States: `EelEmptyState`, `EelLoadingState`, `EelErrorState(problem)`,
  `EelOfflineState`, `EelPermissionRationale`.
- `EelSecureScreen` (FLAG_SECURE/isCaptured), `EelStepFlow` (numbered wizard
  with resume), `EelResultCard` (signed-data display with proof status).

Motion: tokens `motion` only (150/250 ms, standard easing); no decorative
animation on capture/verification flows (predictability under stress).

Typography: tokens scale; all text scales to 200 % without clipping (verified
by golden tests at 1.0/1.5/2.0 scale).

### 5.3 Screen-state completeness (definition)

Every screen in every app implements the full state matrix or is explicitly
N/A in the app plan:

`loading · loaded · empty · offline · error(problem) · permission-denied ·
unauthenticated · version-blocked · tamper-blocked`

Plus per-feature states (e.g. `attestation-failed`, `rate-limited`,
`duplicate-conflict`). Golden files for each state, generated in CI.

---

## 6. i18n and a11y standard

- Locales: `en`, `sw` mandatory everywhere; the 9 additional Kenyan locales
  (guz kam ki kln luo luy mas mer tuv) for all user-facing persona/kit apps
  (poll-clerk already has 11 — replicate); FR/AR for custody + diaspora voter
  screens (M65 FR-M65-010, M24 FR-M24-007).
- Source of truth: `packages/i18n/locales/<lc>/*.json`; a build tool
  (`packages/i18n/tools`) emits per-app ARBs; **no service-level duplication**
  (i18n README). ICU plurals; pseudo-locale pass (`verification/a11y-i18n/pseudo-locale-check.sh`).
- ARB completeness gate in CI: every key present in every locale; missing → red.
- A11y gates: `axe-core` sweep (web views / golden semantic tree), TalkBack and
  VoiceOver traversal scripts for primary flows, focus-order test per screen,
  contrast checks from tokens, `EelMinHitTarget` assertions in widget tests.

---

## 7. Performance budgets (enforced in CI where measurable)

| Budget | Target | Verification |
|---|---|---|
| Cold start → first frame | ≤ 2 s on Android 10 mid-tier (KES-10k class) | integration_test timer on device farm |
| Full-text search first result | ≤ 1 s (reference manual / runbooks) | flutter_test benchmark on fixture index |
| Verification cycle (scan→fetch→verify→countersign) | ≤ 30 s LTE (party-agent), ≤ 60 s (observer) | integration_test with mocked transport timing |
| Heap | ≤ 15 MB steady on 1 GB RAM device (EC-MOB-03) | device-farm memory gate |
| Battery foreground | ≤ 5 %/h; background sync ≤ 2 %/h | profiled soak on device farm |
| APK size | ≤ 40 MB per app (kits ≤ 60 MB with native libs) | CI size gate |
| Queue drain after 48 h offline, 500 items | ≤ 3 min on 3G | netem harness |

---

## 8. Test strategy (all apps)

| Layer | Harness | Required per app |
|---|---|---|
| Unit | `flutter_test` | domain logic: rate limiters, queue priority, EOD arithmetic, polygon geo, SLA math, signature wrappers. ≥ 70 % line coverage on `core/` + `features/*/repository.dart` |
| Widget | `flutter_test` + goldens | every screen × state matrix; hit-target and text-scale goldens |
| Integration | `integration_test` on device farm (Android 10/13 + iOS 15) | primary flows end-to-end incl. offline queue → reconnect dedup; background-kill at every FSM transition (EC-MOB-01) |
| Contract | Pact (dart pact broker consumer) | every endpoint the app calls; `can-i-deploy` gate |
| Security | rooted AVD + Frida scripts + mitmproxy | attestation hard-fail, RASP, pinning, no-PII-logs |
| A11y/i18n | `verification/a11y-i18n` sweep | traversal scripts + pseudo locale |
| Network chaos | toxiproxy profiles (2G/3G/flap/captive) | retry/idempotency/dedup behaviour |
| Load | k6 (`loadtest/`) | BFF endpoints per module AC (e.g. AC-M31-10) |

Test IDs: every test name carries FR/AC tags, e.g.
`testWidgets('lookup fallback [FR-M31-010,AC-M31-03]')`; TRACEABILITY rows map
test ids (step 9.3 of the audit).

---

## 9. Definition of Done — global (applies to every app and every phase)

An app is DONE when all of the following are true. Copy this checklist into
each per-app plan's final section and tick with evidence links.

1. **Features**: every FR in the module checklist implemented; each has ≥ 1
   FR-tagged test.
2. **Acceptance criteria**: every AC in the module spec demonstrated by a
   named test or documented infra-gated manual run with exact blocker (repo
   convention: print the missing prerequisite, never fail silently).
3. **Backend**: every app-called endpoint exists in a bound OpenAPI path with
   passing Pact + idempotency + DAST; allowlist entry removed.
4. **Code quality**: `flutter analyze` 0 issues; `flutter test` green;
   `dart format` clean; no TODO/FIXME/stub strings outside allowlist
   (`eel-completeness scan --fail-on-new` green).
5. **Security**: real attestation wired (no stub clients in release flavor);
   SPKI pinning active; RASP active; FLAG_SECURE screens applied; Isar opened
   with hardware-backed key; no PII in logs/traces (pii-lint green).
6. **Offline**: journal-before-act everywhere; background-kill harness green
   across FSM states; 48 h offline drill script green.
7. **A11y/i18n**: full state matrix goldens; TalkBack/VoiceOver traversal
   scripts pass; pseudo-locale passes; all locales complete.
8. **Performance**: budgets in §7 measured and green.
9. **Build/artifacts**: reproducible release build (byte-identical across 3
   CI runs); SBOM (SPDX) generated; cosign-signed; provenance JSON in
   `dist/mobile/<app>/`.
10. **Deployable**: flavor-pinned APK/AAB (Android) + IPA where in scope;
    MDM package uploaded; version floor registered; `scripts/mobile/install.sh`
    verified on ≥ 2 physical devices.
11. **Telemetry**: app sessions visible in telemetry-ingest; Grafana dashboard
    + SLO alerts registered under `platform/observability/field-mobile-svc/`.
12. **Runbook**: `docs/runbooks/<svc>-mobile.md` updated (degradation, sync,
    force-update, tamper response).
13. **Traceability**: module rows in `docs/plans/TRACEABILITY.yaml` point at
    the new tests; completeness checker module score ≥ gate threshold.
14. **Docs**: app README matches reality (no phantom flavors/features).

---

## 10. Scripts, CI, deployment, fleet management

### 10.1 New directory: `scripts/mobile/`

All bash, `set -euo pipefail`, shared helpers in `scripts/mobile/lib/common.sh`
(logging, flutter version check, app registry table). The app registry
(`apps.txt`) maps app → path → flavors → targets → MDM app id.

| Script | Purpose (details per subcommand in header comment) |
|---|---|
| `bootstrap.sh` | One-time dev/CI env: check/install pinned Flutter (`.flutter-version`), Android SDK 34, JDK 17, Xcode CLT, cosign, syft, fastlane; fetch signing material from OpenBao (`vault kv get -field=... secret/eel/mobile/signing`) into `android/key.properties` (never committed); `flutter doctor -v` gate |
| `codegen.sh [app\|all]` | `pub get` + `dart run build_runner build` (Isar schemas, freezed, gen-l10n) |
| `analyze.sh [app\|all]` | `flutter analyze --fatal-infos` |
| `test.sh [app\|all] [--coverage]` | `flutter test` (+ coverage to `dist/mobile/coverage/`) |
| `build.sh APP --flavor dev\|staging\|production [--target-platform]` | reproducible build: `--dart-define=EEL_BUILD_EPOCH=...` from git commit time; APK/AAB + IPA; writes `dist/mobile/<app>/manifest.json` (git SHA, versions, SPKI pin-set id, SHA-256 of artifacts, SBOM path) |
| `sbom.sh APP` | syft SPDX+CycloneDX per app into `dist/mobile/<app>/sbom/` |
| `sign.sh APP` | Android: `apksigner verify` + cosign sign artifacts; iOS: codesign + notarization via fastlane; writes cosign bundle into manifest |
| `verify-reproducible.sh APP` | builds 3× into temp dirs, compares SHA-256 (gate for DoD #9) |
| `install.sh APP --device <serial\|all>` | adb install -r / `xcrun devicectl device install app`; honors flavor |
| `smoke.sh APP --device <serial>` | cold-launch, wait for telemetry session event (via `curl telemetry-ingest`), drive login via `integration_test` driver, report PASS/FAIL JSON |
| `deploy.sh --env staging\|production [--app APP\|all] [--track internal\|mdm]` | Android Enterprise: publish AAB to private managed Google Play via Custom App Publishing API (service account); iOS: fastlane `pilot`/ABM custom app upload; sets version floor in MDM profile; writes deployment record to `operations/mdm/deployments.log` |
| `rotate-pins.sh --set <id>` | staged SPKI rotation: push dual-pin set via BFF config, wait for fleet adoption (monitor.sh), then drop old pin |
| `monitor.sh [--env staging\|production] [--watch]` | fleet health: pulls per-app session counts, sync backlog p95, crash-free rate, attestation-tier histogram, version skew from telemetry-ingest + Grafana; `--watch` loops 60 s; `--annotate` pushes Grafana annotation |
| `a11y-sweep.sh APP` | runs `verification/a11y-i18n` traversal + axe + pseudo checks against the app's golden semantic snapshots |
| `kill-drill.sh APP --device` | background-kill harness: drives app through FSM states, `adb shell am kill` at each, relaunch, assert no dup/no loss (EC-MOB-01) |
| `offline-drill.sh APP --device` | enables airplane mode + queued ops, restores network, asserts dedup (EC-CON-01) |

### 10.2 justfile recipes (append to root `justfile`)

```
mobile-bootstrap:        @scripts/mobile/bootstrap.sh
mobile-codegen app="all":
mobile-analyze app="all":
mobile-test app="all":
mobile-build app FLAVOR="staging":
mobile-sign app:
mobile-verify-repro app:
mobile-deploy env="staging" app="all":
mobile-install app device:
mobile-smoke app device:
mobile-monitor env="staging":
mobile-a11y app:
mobile-drills app device:      # kill-drill + offline-drill
mobile-ci:                     # analyze + test + build(staging) + sbom for all apps
```

### 10.3 CI (Woodpecker)

- Extend `.woodpecker/test.yaml` `test-flutter` step: add
  `flutter analyze --fatal-infos` before `flutter test`.
- New `.woodpecker/mobile.yaml`:
  - `mobile-build`: matrix over apps × {analyze, test}; then build staging
    flavor for changed apps (path filter), upload artifacts + SBOM.
  - `mobile-sign`: on tag — sign.sh + verify-reproducible.sh + cosign.
  - `mobile-device-nightly` (nightly, needs device farm): install.sh +
    smoke.sh + kill-drill + offline-drill on the 2 reference devices.
  - `mobile-a11y`: a11y-sweep on changed apps.
- Completeness: `tools/completeness` scanner gains a Dart rule classifying
  `Text('... stub')` / `Center(child: Text('... stub'))` literals as
  `dart_stub_ui` (fix the blind spot found in the audit); all findings must be
  allowlisted with expiry or the build is red.

### 10.4 Monitoring & fleet management (operations)

- **Telemetry path**: kit-sdk `TelemetryService` (exists: hash-chained,
  redacting, adaptive-sampled) → `POST /v1/telemetry/ingest-batch`
  (telemetry-ingest crate exists) → OTel → Grafana.
- Dashboards to author under `platform/observability/field-mobile-svc/`:
  `sessions.json` (DAU by role, session duration, check-ins),
  `sync-health.json` (queue depth p95, drain latency, dead-letter count),
  `attestation.json` (tier histogram, tamper events, RASP blocks),
  `crash.json` (crash-free sessions, ANR, top exceptions — no PII),
  `version-skew.json` (fleet versions vs floor), `connectivity.json`
  (network mix, captive-portal hits, pin failures).
- SLO alerts `*-slo-alerts.yaml`: sync dead-letter > 0 for 10 min;
  attestation failures > 1 %/5 min; version-floor violation > 0;
  BFF p99 over module SLA; crash-free < 99.5 %.
- **MDM** (`operations/mdm/`): per-app profiles already described; add
  `apps/<app>.policy.json` (version floor, managed-config keys: base URL,
  station binding, role, feature flags); force-update flow implements
  EC-MOB-06 (block start ≤ 48 h, queue survives update, N-1 migration tested).
- **Device lifecycle**: tie install/deploy to `operations/mdm/lifecycle.md`
  stages (`device.enrolled` → deploy.sh records deployment event to
  inventory-svc).

---

## 11. Delivery waves (sequence + exit criteria)

Dependency order: foundations first, then backends, then apps, then hardening.
Apps inside a wave are parallelizable. Estimates are sequential EDs for one
agent; scale by parallelizing across apps.

### Wave M0 — Foundations (≈ 10 ED)

1. `packages/design-system/flutter` (eel_design) + tokens codegen + 20 core
   widgets + golden rig.
2. `packages/eel-core-flutter`: bootstrap FSM, session/OIDC+PKCE, config,
   connectivity, outbox drain, evidence upload, problem+json, tamper.
3. kit-sdk: SPKI pinning adapter; `EelEncryptedIsarStore`; push abstraction;
   real Play Integrity/App Attest clients (backend verdict endpoint stubbed
   until M1 lands — dev flavor only).
4. `scripts/mobile/*` + justfile recipes + `.woodpecker/mobile.yaml`.
5. Scanner: `dart_stub_ui` rule; CI analyze step.

**Exit**: `just mobile-ci` green for hello-flutter + one migrated app
(cvr-field-officer as pilot); reproducible-build gate green; scanner rule live.

### Wave M1 — Backend closure (≈ 12 ED)

1. mobile-bff real handlers (§4.1) + push-router crate + Keycloak/SpiceDB
   wiring + idempotency store.
2. Contracts: field/*.yaml authored; bff.yaml bindings as handlers land.
3. training-api CRL + LMS path fixes; inventory custody bindings.
4. k6 profiles for BFF endpoints; Grafana dashboards + alerts.

**Exit**: Pact suite green for all persona-app endpoints; 18 bff paths bound
(allowlist cleared); AC-M31-10 style load evidence for check-in + incidents.

### Wave M2 — Persona apps (≈ 45 ED; the biggest wave)

poll-clerk (12), party-agent (10), observer (10), field-support (8),
personnel-completion (5). Full FR coverage per app plan; all DoD items.

**Exit**: per-app DoD checklists signed with evidence; MDM staging rollout;
UAT scripts for DPO/clerk/agent/observer roles.

### Wave M3 — Kits (≈ 30 ED)

bvr-kit (8), bvi-kit (10), diaspora-kit (5), rts-kit completion (4),
custody-mobile backend binding + locales (3). Hardware-gated items follow the
repo convention: everything testable in CI is tested; device-only items are
listed with exact hardware prerequisite and a self-check script.

**Exit**: kit DoD lists green except explicit device-gated rows; ABIS ingest
round-trip proven with kit-simulator.

### Wave M4 — citizen-verify (new app) + long tail (≈ 15 ED)

citizen-verify app (M13/M29) per its plan (8); lms-offline completion (2);
accred-scanner completion (2); kit-l1 integration into bvi/rts (2);
legacy-android decision execution (1).

**Exit**: citizen-verify DoD green incl. 10M-user read path contract tests
against BFF + CDN caching; long-tail apps green.

### Wave M5 — Hardening & rollout (≈ 12 ED)

1. 48 h offline drills, toxiproxy chaos, background-kill sweeps — all apps.
2. Rooted-AVD security gate; mitmproxy captive gate.
3. Reproducible-build verification all apps; SBOM + cosign in release pipeline.
4. Full a11y/i18n sweep; fix findings.
5. MDM production rollout runbook + staged cohort plan; monitor.sh dashboards
   live; runbooks finalized.

**Exit**: `just ci` green; `eel-completeness scan --strict` zero
non-allowlisted findings in all `apps/` paths; audit steps 9.2 (artefacts) and
9.7 (SLA) evidenced for every app; gate-G7 style report
(`docs/audit/mobile-completeness.md`) written.

---

## 12. Risks and dependencies

| Risk | Mitigation |
|---|---|
| BFF endpoints slip → apps blocked | Contracts authored first (contract-first per repo policy); apps develop against Pact mocks; binding is the backend team's exit criterion |
| Hardware (FAP60, AntIris, RFID readers, printers) unavailable | All logic behind driver interfaces with synthetic transports (existing pattern); device-gated items self-check with `--check-hardware` mode and exact prerequisite messages |
| Keycloak/SpiceDB/OpenBao not reachable in CI | dev doubles exist per repo conventions (`hello-svc` pattern); tests use testcontainers where possible |
| Play Integrity verdict needs GCP project | verdict endpoint abstracted; CI uses software tier with explicit `attestation_tier=ci-software` marking (matches legacy-android pattern) |
| Isar 3.x maintenance risk | all collections behind `local_db.dart`; migration path documented (N-1 → N test required) |
| 11-locale translation lag | en+sw are release blockers; other 9 locales gated as "in progress" banner + English fallback (never missing keys — CI enforces key presence with English fallback content) |
| Reproducible Flutter builds | pinned toolchain via `.flutter-version` + Nix (flake.nix exists); epoch/commit darts-defines; verify-reproducible gate |

---

## 13. Per-app plan index

| Plan file | App |
|---|---|
| kit-sdk-flutter.md | packages/kit-sdk-flutter (+example) |
| cvr-field-officer.md | bvr-svc/apps/cvr-field-officer |
| bvr-kit.md | biometric-edge-svc/apps/bvr-kit |
| bvi-kit.md | biometric-edge-svc/apps/bvi-kit |
| diaspora-kit.md | biometric-edge-svc/apps/diaspora-kit |
| legacy-android.md | biometric-edge-svc/apps/legacy-android |
| rts-kit.md | rts-svc/apps/rts-kit |
| custody-mobile.md | inventory-svc/apps/custody-mobile |
| poll-clerk.md | field-mobile-svc/apps/poll-clerk |
| party-agent.md | field-mobile-svc/apps/party-agent |
| observer.md | field-mobile-svc/apps/observer |
| field-support.md | field-mobile-svc/apps/field-support |
| personnel.md | field-mobile-svc/apps/personnel |
| citizen-verify.md | field-mobile-svc/apps/citizen-verify (new) |
| lms-offline.md | training-svc/apps/lms-offline |
| accred-scanner.md | training-svc/apps/accred-scanner |
| kit-l1.md | ai-platform-svc/apps/kit-l1 |
| hello-flutter.md | hello-svc/apps/hello-flutter |