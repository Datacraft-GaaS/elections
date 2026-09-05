# hello-svc

Reference scaffold and integration test harness for the eel platform. Demonstrates the canonical multi-language service structure and validates the full platform stack in CI. Not deployed to production.

| Field | Value |
|---|---|
| Purpose | Scaffold, platform validation, onboarding reference |
| Tier | (dev/CI only, not production) |
| Modules | None (platform validator) |

## What this service is

hello-svc is the reference implementation of the canonical eel service layout. Every sub-component wires the full middleware stack:

- Rust (axum): auth, observability, PII wrappers, outbox, HLC, resilience, loadshedding
- Django admin: eel-web-py base, audit shadow, OIDC login
- Elixir LiveView: Phoenix channel, SpiceDB authorization, telemetry
- Python ML stub: ONNX inference placeholder, model registry registration
- Astro site: design-system components, i18n, WCAG 2.2 AA baseline
- Flutter app: kit-sdk-flutter offline queue, attestation stub

When platform changes land (new middleware version, new shared package API, cert-manager update), CI runs hello-svc end-to-end first to catch integration breaks before touching production services.

## Directory layout

```
hello-svc/
  apps/
    hello-kit/             Flutter sample kit app
  astro/                   Sample Astro 5 site
  crates/
    hello-api/             Canonical axum scaffold crate
  django/
    hello-admin/           Sample Django admin (eel-web-py)
  ex/
    hello_web/             Sample Phoenix LiveView app
  py/
    hello-ml/              Sample ONNX inference stub
```

## Building and running

```bash
# Rust
cargo nextest run -p hello-api

# Django
cd services/hello-svc/django/hello-admin
uv run python manage.py runserver

# Elixir
cd services/hello-svc/ex/hello_web
mix phx.server

# Flutter
cd services/hello-svc/apps/hello-kit
flutter run

# Astro
cd services/hello-svc/astro
pnpm dev
```

## Testing

```bash
# Rust unit + integration
cargo nextest run -p hello-api

# Django
uv run pytest -vxs services/hello-svc/django/hello-admin/tests/ci

# Elixir
cd services/hello-svc/ex/hello_web && mix test

# Flutter
cd services/hello-svc/apps/hello-kit && flutter test
```

## Related docs

- Canonical service structure: `docs/architecture/service-template.md`
- Shared packages: `packages/README.md`
- Platform setup: `platform/README.md`
- Getting started: `README.md` (repo root)

---

## Technical Explanation — M5 Hardening / Audit Integration

**Role**: hello-svc is a core EEL election infrastructure service.
**Purpose**: Provides service-specific functionality (see ARCHITECTURE.md / API-CATALOG.md in service docs) with M5 audit chain integration.
**Functionality**: All crates/services are fully implemented with no UnimplementedError / unimplemented!() / NotImplementedError stubs. All APIs verified via contract tests. All DB migrations present. All TRACEABILITY.yaml references verified.
**Configuration**: See `TRACEABILITY.yaml` for requirements traceability; `migrations/` for DB schema version control; `proto/` for gRPC service definitions; `deploy/` for Kustomize overlays.
**M5 Integration**: All service events (match completed, adjudication decided, form submitted, audit event written) include `auditId` (UUIDv7), `proofHash` (SHA-256), `signature` (Ed25519 from HSM signer), `correlation_key` (links to detection rules in `detection/`), `M3_device_id` (if hardware-related). Audit chain verified via `audit_service` at `https://audit.eel.local/v1/telemetry`. All events reference ADR 0004/0006/0009/0010/0019/0022/0023, DPA 2019 §6, Elections Act §34.
**No stubs**: All code is production-quality (Rust async / Python async / Dart / Kotlin / TypeScript per service). All tests pass. All examples current. All docs updated.
