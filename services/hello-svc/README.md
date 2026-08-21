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
