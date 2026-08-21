# ADR 0002: Language selection

Status: accepted
Date: 2026-08-21

## Context

The system spans ~69 modules (growing toward ~100) across backend, kits, mobile, portals, and desktop. Three-year contract, 15-year audit retention. The selection metric is not time-to-first-prototype (Python wins) but velocity at month 18 with several parallel teams shipping across ~100 services (Rust wins), combined with the hard requirement that election-critical services eliminate memory-safety CVE classes by construction. See `docs/architecture/decision-log.md` §3 Phase 4 for the evolution.

## Decision

Tiered assignment, no per-service improvisation.

- **Rust (default)** for all backend services except the exceptions below. Stack: axum + tower + sqlx (compile-time SQL) + tokio + utoipa (OpenAPI from types) + tonic (gRPC) + serde + garde + tracing + opentelemetry + rdkafka + fred + opensearch-rs + cryptoki + oqs + sigstore-rs + cargo-nextest + testcontainers-rs + cargo-fuzz.
- **Python 3.12 async** for ML pipelines (M17, M26, M28, M21 SHANGRLA), vendor-SDK-bound integrations (IPRS, KRA, telco USSD/SMS, CBK, KEBS, ODPC), and ops glue. Stack: FastAPI + Pydantic v2 + SQLAlchemy 2 async + uv + ruff + pyright.
- **Elixir + Phoenix LiveView** for real-time and public-exposed surfaces where BEAM's per-connection process model earns the fourth language: M11 dynamic, M13 real-time, M15, M19, M23, M26 triage. Stack: Phoenix + LiveView + Ecto + Broadway + Oban + Rustler for Rust interop.
- **Dart / Flutter** for KIEMS kit firmware and mobile apps.
- **Django 5 + HTMX + Alpine** for internal admin CRUD (see ADR-0008).
- **Astro 5 + Svelte islands** for static-first public read surfaces (see ADR-0008).
- **Tauri 2** (Rust + web UI) for operator desktop (see ADR-0008).

Every language has a `<scaffold> new` template that pre-wires the shared crates/libs (auth, secrets, events, outbox, telemetry, resilience, PII). One-command scaffolding is what turns Rust-default into net-positive velocity.

## Consequences

Positive: memory-safety CVE class eliminated in security-boundary services; compile-time cross-module type safety at scale; single static binaries match ADR-0006 immutable-build ethos; LiveView removes browser-JWT attack surface for exposed surfaces.

Negative: four backend languages is real cost, mitigated by strict per-category allocation and shared cross-language libraries. Rust and Elixir hiring pools are thinner in Kenya; mitigated by senior anchors + 6-8 week immersion + remote hires. Rust compile times are real; mitigated by sccache, cargo-hakari, workspace hygiene.

## Alternatives rejected

- Pure Python: no memory-safety guarantee for security-boundary services; refactor cost at 100-service scale unacceptable.
- Pure Rust: would force rewriting Python-native ML reference implementations (SHANGRLA) and vendor SDKs; audit-visible risk.
- Node/TypeScript: no memory safety; ecosystem CVE record too noisy for consensus-critical services.
- Go: considered; Rust's compile-time SQL, richer type system, and safety guarantees at unsafe boundaries won.

## Related

ADR-0006 (build), ADR-0007 (identity libs), ADR-0008 (portal), ADR-0015 (event bus libs), ADR-0018 (write path).
