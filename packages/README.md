# packages/

Shared cross-cutting libraries. Every service consumes these; nothing else lives here.

Each package has its own README. This file is the index.

---

## Core primitives (Rust)

| Package | Language | Purpose | Tier | Exposed API surface | Consumers |
|---|---|---|---|---|---|
| `eel-crypto-rs` | Rust | Ed25519, BLAKE3, RFC 8785, Merkle, PKCS#11 pool, PQ hybrids, TSA, cosign | Platform | `ed25519`, `blake3`, `rfc8785`, `merkle`, `constant_time`, `pkcs11_pool`, `pq`, `tsa`, `cosign` modules | `eel-events-rs`, `verifier-rs`, all signing paths |
| `eel-hlc-rs` | Rust | Hybrid Logical Clock; wire form `physical_ms.counter.node` | Platform | `Hlc`, `HlcClock`, `WallClock` trait, `HlcError` | `eel-events-rs`, `eel-eventstore-rs`, all services |
| `eel-events-rs` | Rust | CloudEvents envelope, signing, schema lookup, consumer | Platform | `Envelope`, `DataClass`, `EventBuilder`, `EventConsumer`, `GapBuffer`, `KarapaceClient` | All Rust services |
| `eel-eventstore-rs` | Rust | Append-only event store, snapshot, replay | Platform | `EventStore`, `Aggregate` trait, `StoredEvent`, `EventReplay`, `SnapshotTailReplay` | All event-sourced Rust services |
| `eel-outbox-rs` | Rust | Transactional outbox; sole Kafka publish path | Platform | `Outbox`, `OutboxEnvelope`, `OutboxSchema` | All Rust services |
| `eel-pii-rs` | Rust | Five-tier PII type wrappers (A-E) with audit sink | Platform | `PIIRestricted`, `PIISensitive`, `PIIPublic`, `PIIOperational`, `PIIEphemeral`, `Purpose`, `PIIAuditSink` | All Rust services handling personal data |
| `eel-resilience-rs` | Rust | Circuit breaker, bulkhead, retry, timeout, idempotency, degradation | Platform | `CircuitBreaker`, `Bulkhead`, `with_retry`, `IdempotencyStore`, `DegradationFlag`, `DegradationMode` | All Rust services |
| `eel-loadshedding-rs` | Rust | CoDel-style priority load shedder Tower layer | Platform | `LoadSheddingLayer`, `Priority`, `CoDelConfig`, `ShedMetrics` | All Rust API services |
| `eel-auth-rs` | Rust | JWT/JWKS verify, SpiceDB ReBAC, audit emit | Platform | `JwksVerifier`, `Subject`, `SpiceDbClient`, `Decision`, `AuditSink` | All Rust services |
| `eel-observability-rs` | Rust | OTel init, SLO macros, audit shadow, PII log scrubber | Platform | `init_otel`, `OtelConfig`, `SloDefinition`, `PiiScrubberLayer`, `audit_shadow_sql` | All Rust services |
| `eel-secrets-rs` | Rust | OpenBao dynamic Postgres creds, transit encrypt, PKI | Platform | `PgDynamicCred`, `Transit`, `AdminCred`, `LeafCert` | All Rust services |

## Service plumbing (Rust)

| Package | Language | Purpose | Tier | Exposed API surface | Consumers |
|---|---|---|---|---|---|
| `eel-api-server-rs` | Rust | Opinionated axum builder: auth + SpiceDB + rate limit + idempotency + RFC 9457 + signing | Platform | `ApiServerBuilder`, `ApiServer`, `Problem`, `ProblemType` | All Rust services |
| `service-template-rs` | Rust | `cargo-generate` template + conformance test harness | Platform | `conformance` module (feature-gated) | New Rust service scaffolding, CI |
| `verifier-rs` | Rust | RFC 6962 Merkle + STH + cosign verifier; Dart FFI + WASM | Platform | `Sth`, `InclusionProof`, `verify_inclusion`, `verify_consistency`, C ABI | `kit-sdk-flutter` (FFI), browser verifier (WASM) |

## Python mirrors

| Package | Language | Purpose | Tier | Exposed API surface | Consumers |
|---|---|---|---|---|---|
| `eel-hlc-py` | Python | HLC timestamp; wire-compatible with Rust | Platform | `Hlc`, `HlcClock`, `HlcClockProtocol` | All Python services |
| `eel-events-py` | Python | CloudEvents envelope, signing, consumer middleware | Platform | `Envelope`, `DataClass`, `EventBuilder`, `EventConsumer`, `KarapaceClient` | All Python services |
| `eel-eventstore-py` | Python | Append-only event store (SQLAlchemy) | Platform | `EventStore`, `Aggregate`, `StoredEvent`, `OptimisticConcurrencyError` | Event-sourced Python services |
| `eel-outbox-py` | Python | Transactional outbox; sole Kafka publish path | Platform | `emit`, `MAX_INLINE_BYTES`, `OutboxSchema` | All Python services |
| `eel-pii-py` | Python | Five-tier PII wrappers + SQLAlchemy types | Platform | `PIIRestricted`, `PIISensitive`, `PIIPublic`, `PIIOperational`, `PIIEphemeral`, Ecto type decorators | All Python services handling personal data |
| `eel-auth-py` | Python | JWT/JWKS verify, SpiceDB, audit; Starlette middleware | Platform | `JwksCache`, `verify_token`, `Subject`, `SpiceDBClient`, `AuthMiddleware`, `require_permission` | All Python FastAPI services |
| `eel-resilience-py` | Python | Circuit breaker, bulkhead, retry, idempotency, degradation | Platform | `CircuitBreaker`, `Bulkhead`, `retry_with_jitter`, `IdempotencyMiddleware`, `DegradationFlag` | All Python services |
| `eel-loadshedding-py` | Python | CoDel-inspired load shedding Starlette middleware | Platform | `LoadSheddingMiddleware`, `Priority`, `shed_response` | All Python API services |
| `eel-observability-py` | Python | OTel init, SLO helpers, audit shadow, PII scrubber | Platform | `init_otel`, `SloTimer`, `generate_audit_shadow_trigger`, `PiiScrubberFilter` | All Python services |
| `eel-secrets-py` | Python | OpenBao dynamic Postgres creds, transit encrypt | Platform | `PgDynamicCred`, `Transit`, `OpenBaoClient` | All Python services |
| `eel-web-py` | Python | Django reusable app: auth middleware, SpiceDB mixin, base templates, HTMX, session store | Service | `AuthMiddleware`, `SpiceDBMixin`, `EelBaseTemplate`, `partial_response`, `PgSessionStore` | Django admin and oversight surfaces |
| `service-template-py` | Python | FastAPI cookiecutter template + conformance harness | Platform | `conformance` module | New Python service scaffolding, CI |

## Elixir mirrors

| Package | Language | Purpose | Tier | Exposed API surface | Consumers |
|---|---|---|---|---|---|
| `eel-hlc-ex` | Elixir | HLC timestamp; wire-compatible with Rust/Python | Platform | `Eel.Hlc`, `Eel.Hlc.Clock` GenServer | All Elixir services |
| `eel-events-ex` | Elixir | CloudEvents envelope, signing, Broadway consumer | Platform | `Eel.Events.Envelope`, `Eel.Events.Builder`, `Eel.Events.Consumer` | All Elixir services |
| `eel-eventstore-ex` | Elixir | Append-only event store (Ecto) | Platform | `Eel.EventStore`, `Eel.EventStore.StoredEvent` | Event-sourced Elixir services |
| `eel-outbox-ex` | Elixir | Transactional outbox via Ecto.Multi | Platform | `Eel.Outbox.emit/3`, `Eel.Outbox.Envelope` | All Elixir services |
| `eel-pii-ex` | Elixir | Five-tier PII wrappers + Ecto types | Platform | `Eel.PII.Restricted`, `Eel.PII.Sensitive`, `Eel.PII.Public`, `Eel.PII.AuditSink` | All Elixir services handling personal data |
| `eel-auth-ex` | Elixir | JWT/JWKS verify, SpiceDB, audit; Plug middleware | Platform | `Eel.Auth.JwksCache`, `Eel.Auth.Plug`, `Eel.Auth.SpiceDB`, `Eel.Auth.RequirePermission` | All Elixir Phoenix services |
| `eel-resilience-ex` | Elixir | Circuit breaker, bulkhead, retry, idempotency, degradation | Platform | `Eel.Resilience.CircuitBreaker`, `Bulkhead`, `Retry`, `Degradation` | All Elixir services |
| `eel-loadshedding-ex` | Elixir | CoDel-inspired load shedding Plug | Platform | `Eel.LoadShedding.Plug`, `Priority`, `Metrics` | All Elixir API services |
| `eel-observability-ex` | Elixir | OTel init, SLO helpers, audit shadow, PII scrubber | Platform | `Eel.Observability`, `Eel.Observability.Slo`, `Eel.Observability.PiiScrubber` | All Elixir services |
| `eel-secrets-ex` | Elixir | OpenBao dynamic Postgres creds, transit encrypt | Platform | `Eel.Secrets.PgDynamicCred`, `Eel.Secrets.Transit` | All Elixir services |
| `service-template-ex` | Elixir | Mix template for Phoenix services + conformance harness | Platform | `ServiceTemplateEx.Conformance` | New Elixir service scaffolding, CI |

## Cross-cutting

| Package | Language | Purpose | Tier | Exposed API surface | Consumers |
|---|---|---|---|---|---|
| `kit-sdk-flutter` | Dart/Flutter | Secure offline-capable kit app foundation: store, queue, verifier FFI, HLC, sync, time-fence, duress, accessibility widgets | Service | `EelStore`, `OfflineQueueManager`, `EelVerifier`, `EelHlcClock`, `SyncEngine`, `TimeFenceGuard`, `DuressDetector`, accessibility widgets | All kit apps (M01/M05/M08/M24/M25/M27) |
| `eel-web-astro` | TypeScript/Astro | Signed-content loader, Swahili/English i18n, SEO helpers | Service | `SignedContentLoader`, `t()`, `EelSeoMeta`, `integration` | Portal and oversight Astro sites |
| `design-system` | CSS/TS/Python | Design tokens, components (Astro/Svelte/Django), WCAG 2.2 AA | Platform | Token JSON, Astro components, Svelte components, Django template tags | All portal, admin, and kit surfaces |
| `api-clients` | Dart/TS/Python/Rust | Auto-generated REST clients from OpenAPI specs | Platform | Language-specific typed client classes per bounded context | Flutter apps, Astro portals, internal tooling, service-to-service |
| `i18n` | JSON | Swahili and English locale files for all surfaces | Platform | JSON locale files; tools for validation and key extraction | `kit-sdk-flutter`, `eel-web-astro`, `eel-web-py` |
| `openapi-contracts` | YAML | OpenAPI 3.1 specs per bounded context; Spectral ruleset | Platform | Spec YAML files; drives `just gen-clients` | `api-clients`, all services (as spec reference) |
| `event-schemas` | JSON Schema | JSON Schema per CloudEvent type per `(ctx, agg, event, version)` | Platform | Schema JSON files registered in Karapace | `eel-events-*` (producer/consumer validation), Karapace |
| `authz-schema` | SpiceDB ZED | SpiceDB authorization schemas per bounded context; golden tests | Platform | `.zed` files deployed to SpiceDB cluster | `eel-auth-*` (check calls), SpiceDB |
| `data-dictionary` | YAML | PII classification for every Postgres column: tier, retention, encryption, owner | Platform | YAML files; drives CI coverage checks and audit shadow generation | `eel-observability-*`, DPO review, CI |
