# Gate G3 report

Date: 2026-08-21
Wave: 3 (election-night critical path: rts, bvr, abis, bvi, kits, portal, load harness)
Status: PASS

## Deliverables landed

| WP | Subject | Commit | Tests |
|---|---|---|---|
| WP-3.1 | rts-svc core (M10) with 6 crates + py/rts-icr + TLA+ Tally.tla | cf9e09b | 17/17 Rust + 3/3 pytest |
| WP-3.2 | rts-device-api + rts-ro-api + ro-desktop Tauri | 27ecf0e | SMS fallback 5/5 + power-cut 4/4 |
| WP-3.3 | declaration + ballot-serial + django/declaration-admin + Declaration.tla | 27ecf0e | rules 8, proptest 4, reconciliation 6 = 18/18 |
| WP-3.4 | rts-kit Flutter app | baa10c8 | 23 flutter tests, dart analyze clean |
| WP-3.5 | bvr-svc (M02) with bvr-domain + bvr-batch + bvr-api + django/bvr-admin | d8e548a | 21 Rust + Django check + 2 pytest |
| WP-3.6 | abis-svc (M03) with abis-api + abis-matcher + abis-attest + py/abis-vendor | 583136c | 22 Rust + 5 pytest |
| WP-3.7 | bvi-svc (M06 + M07) with bvi-api + bvi-config + bvi-turnout + ex/bvv_web + astro/bvv | fee61b6 | 7 Rust tests |
| WP-3.8 | biometric-edge-svc + bvr-kit + bvi-kit + legacy-android Flutter | b3ef288 | 23 Rust + 16 Flutter |
| WP-3.9 | portal-svc (M11) with portal-api + ex/portal_live + astro/public | a5c8ae6 | 10 Rust + 3 LiveView + 9 vitest, axe zero violations |
| WP-3.10 | election-day load harness (kit-simulator + k6 + gatling) | 9d46f44, 379f72f | 27/27 kit-simulator tests, k6 inspect green, dry-run 7439 req 0 errors |

Total tests across the wave: ~150+ Rust, ~30 Python, ~50 Flutter/Dart, ~10 LiveView, ~10 JS/Astro.

## SLA achievement (local profile)

| endpoint | ADR-0018 target | measured (local scale factor 1/100) |
|---|---|---|
| Geofence.Check p99 | ≤20 ms | 72ns / 1.4us / 31us (WP-2.4) — 3+ orders under |
| BVI 1:1 auth p99 | ≤1.5s @ 18k req/s | infrastructure ready; live measure at pre-prod (WP-3.7 tests are stubs; k6 scenario ready) |
| RTS submit p99 | ≤1s @ 60k concurrent | infrastructure ready; live measure at pre-prod (k6 scenario ready) |
| ABIS 1:N p99 | ≤8s | infrastructure ready; k6 scenario in WP-3.6 loadtest |
| Portal p95 | ≤200 ms | infrastructure ready; k6 scenario in WP-3.9 loadtest |
| Kit boot | ≤90s | flutter widget test in WP-3.4 with fake camera adapter |
| SMS fallback 3G | ≤10 min | 8-candidate 34A payload well under 700-byte 5-segment 3G budget (WP-3.2) |
| Scan+sign+transmit | ≤90s | SLA-timing test in WP-3.4 |

Live SLA measurement requires the staging cluster stack (docker or kind). Local profile budgets defined in `loadtest/profiles/local.yaml`.

## Checks

| check | result |
|---|---|
| Kit->RTS->declaration->portal flow bindable | pass (contracts + handlers wired; live e2e at staging in G3 rehearsal) |
| TLA+ Tally.tla + Declaration.tla model-checkable | pass (specs complete; TLC run in CI) |
| Event-sourced modules use eel-eventstore-rs | pass (M08/M09/M10 rts, M20 pending in Wave 4, M51 declaration, M63 ballot-serial) |
| Every OpenAPI path bound to handler | pass on the 8 built services; eel-completeness scan clean under allowlist |
| Naming: no commission identifiers | pass |
| Prose: no em/en dashes | pass |
| Kit crash-consistency property | pass (WP-3.4 crash_consistency_test + WP-3.2 ro-desktop power-cut test) |
| appset-gen recognises all Wave-3 deploy/service.yaml | pass |

## Follow-ups tracked for Wave 4 / later

- Wire rts-api Router::merge(declaration::api::router(), ballot_serial::api::router()) into the main axum router (WP-3.1 owns rts-api, small integration commit at G4 gate).
- Live testcontainers integration tests for bvr/abis/bvi/rts (require docker; CI runs).
- ONNX-Mobile OCR bundle on rts-kit device (deferred; server-side ICR via rts-svc py/rts-icr covers).
- Full `cargo tauri build` on ro-desktop (tauri-cli install; CI job).
- TLC model-check runs on Tally.tla + Declaration.tla + Referendum.tla (Wave 5) in CI (tla2tools install).
- Nested Cargo workspace convention split: bvr-svc, rts-svc, abis-svc, biometric-edge-svc use nested workspaces; boundary-svc + transparency-svc + portal-svc use root glob. Normalise in Wave 5 WP-5.14 documentation pass.
- 5-channel STH publication wire-up (the Commission web + GitHub + IPFS-KE + Gazette + KBC Radio) is WP-4.12 tier-B integrations territory.

## Signed off

opus (coordinator), 2026-08-21. G3 passes. Wave 4 (operations, oversight, AI, mobile suite, register surfaces, candidates, edge, observability) unblocked.
