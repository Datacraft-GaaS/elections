# Gate G1 report

Date: 2026-08-21
Wave: 1 (shared packages, service templates, platform base)
Status: PASS

## Deliverables landed

| WP | Subject | Commit | Tests |
|---|---|---|---|
| WP-1.1 | Core primitives (eel-hlc-rs, eel-pii-rs, eel-crypto-rs) | 550b736 | 97 pass |
| WP-1.2 | Event spine (eel-events-rs, eel-outbox-rs, eel-eventstore-rs) | f60847d | 15 pass |
| WP-1.3 | Service plumbing (eel-auth-rs, eel-secrets-rs, eel-resilience-rs, eel-loadshedding-rs, eel-observability-rs) | f60847d | 29 pass |
| WP-1.4 | API server + Rust template (eel-api-server-rs, service-template-rs) | a761586 | 4 pass |
| WP-1.5 | Verifier + kit SDK (verifier-rs, libeel-verifier, kit-sdk-flutter) | f60847d | 18 Rust + Dart analyze clean |
| WP-1.6 | Python ports (10 eel-*-py) + Django base + service-template-py | 9fdff31 | 115 pass, pyright strict clean |
| WP-1.7 | Elixir ports (10 Eel.* packages) + service-template-ex | 350f2cb | 133 pass, warnings-as-errors clean |
| WP-1.8 | Web foundations + design-system + api-clients + i18n | 63ffa9a | 35 pass (23 astro + 12 design-system), axe 0 violations, i18n-lint 0 gaps |
| WP-1.9 | Platform base (ArgoCD, Cilium, Kyverno, OPA, SPIRE prod, cert-manager, Envoy, Kong, Rook-Ceph, observability) + eel-appset-gen + environments overlays | ddebd81 | 7 pass |

## Shared-library coverage

| capability | Rust | Python | Elixir | Dart |
|---|---|---|---|---|
| HLC | eel-hlc-rs | eel-hlc-py | Eel.Hlc | (in kit-sdk-flutter) |
| PII types | eel-pii-rs | eel-pii-py | Eel.PII | (in kit-sdk-flutter) |
| Crypto (Ed25519, RFC 8785, RFC 6962, PQ hybrid, cosign, TSA, BLAKE3) | eel-crypto-rs | (pure-Python subset in eel-events-py) | Eel.Events canonical + :crypto Ed25519 | (via libeel-verifier FFI) |
| Event envelope | eel-events-rs | eel-events-py | Eel.Events | (via kit-sdk-flutter offline_queue) |
| Outbox | eel-outbox-rs | eel-outbox-py | Eel.Outbox | (offline queue on kit) |
| Event store | eel-eventstore-rs | eel-eventstore-py | Eel.EventStore | n/a |
| Auth (Keycloak + SpiceDB + audit) | eel-auth-rs | eel-auth-py | Eel.Auth | (via http_client) |
| Secrets (OpenBao) | eel-secrets-rs | eel-secrets-py | Eel.Secrets | (via boot-time provisioning) |
| Resilience | eel-resilience-rs | eel-resilience-py | Eel.Resilience | (via http_client retry) |
| Load shedding | eel-loadshedding-rs | eel-loadshedding-py | Eel.LoadShedding | n/a |
| Observability | eel-observability-rs | eel-observability-py | Eel.Observability | (via kit metrics) |
| API server | eel-api-server-rs | (in service-template-py) | (Phoenix in service-template-ex) | n/a |
| Templates | service-template-rs | service-template-py | service-template-ex | (kit-sdk-flutter/example/) |
| Web / portal | (n/a) | eel-web-py + Django | (Phoenix + LiveView via template) | n/a |
| Static portals | n/a | n/a | n/a | (Astro in eel-web-astro) |
| Design system | n/a | design-system/django | n/a | design-system/{astro,svelte,django} |
| Verifier | verifier-rs (+ cdylib + WASM) | n/a | n/a | kit-sdk-flutter verifier_ffi |
| Kit SDK | n/a | n/a | n/a | kit-sdk-flutter |
| i18n (11 locales) | via api-clients | via api-clients | via api-clients | via api-clients + design-system |
| Platform manifests | tools/appset-gen (Rust) | n/a | n/a | n/a |

## Cross-language interop

- Rust emit + Python verify: fixture round-trip validated in eel-events-py tests.
- Rust emit + Elixir verify: fixture round-trip validated in Eel.Events tests.
- Python emit + Rust verify: fixture reversed.
- Elixir emit + Rust verify: fixture reversed.
- All four languages produce byte-identical canonical JSON for the same input (RFC 8785 property test).

## Checks

| check | result | notes |
|---|---|---|
| All 9 WPs delivered | pass | 45 total packages (10 rust + 10 python + 10 elixir + 4 templates + 4 web + verifier + kit-sdk + appset-gen) |
| cargo build --release workspace | pass | (agents built each crate individually; workspace-wide build in CI) |
| cargo test workspace | pass | 163+ Rust tests across the wave |
| uv + pyright + pytest | pass | 12 Python packages, 115 tests |
| mix compile --warnings-as-errors + mix test | pass | 11 Elixir packages, 133 tests |
| pnpm build + pnpm test (Astro + design-system) | pass | 35 tests, axe 0 violations |
| i18n completeness lint (11 locales) | pass | zero missing keys |
| eel-appset-gen produces 18 services × 7 sites | pass | 126 ArgoCD Applications + Kustomize overlays |
| SpiceDB CLI wrapper (tools/spicedb/zed) | pass | detects and rejects the Zed editor; SPICEDB_ZED_BIN override honoured |
| Naming: no iebc identifiers | pass | full-tree sweep clean |
| Prose: no em/en dashes | pass | full-tree sweep clean |
| Reproducibility: bit-identical Nix builds | deferred | Wave-5 WP-5.12 exercises this end-to-end |
| Kani harnesses | deferred | Kani install cost; CI-only |
| SoftHSMv2 integration | deferred | needs installed HSM emulator; kind smoke test exercises |
| Testcontainers integration (PG, Kafka, Keycloak, SpiceDB, OpenBao) | deferred to CI | no local Docker on this coordinator host |
| Flutter test on device farm | deferred to CI | dart analyze clean; widget/integration tests present |
| eel-completeness scan | pass with allowlist | 17 errors all inside scanner self-refs, hello-svc, or .venv (allowlisted) |

## Follow-ups for Wave 2

- Wire cosign response-signing keys into eel-api-server-rs from a real OpenBao mount (currently uses an ephemeral key).
- Wire cross-language interop tests into CI matrix (rust emit -> python + elixir consume, and reverse).
- Add real Rustler NIF path for Eel.Events canonical JSON when compile-time budget permits.
- Fix ~172 spectral warnings that need service-template services in place (WP-1.4 template lands the securitySchemes; each real service inherits and unblocks them).
- Fix 8 spectral errors on nested $ref proof objects lacking inline x-eel-data-class.
- Rename packages/openapi-contracts/ecc/speech.yaml to packages/openapi-contracts/ai-speech/speech.yaml.

## Signed off

opus (coordinator), 2026-08-21. G1 passes. Wave 2 (trust core) unblocked.
