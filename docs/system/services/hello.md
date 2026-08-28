# hello-svc

## 1. Overview

hello-svc is the platform reference scaffold and CI integration harness. It is not
deployed to production; it exists so that changes to shared middleware, packages, and
certificate infrastructure are validated end-to-end against a minimal service before
touching real services (services/hello-svc/README.md).

- **Runtime**: one tiny axum binary plus per-language scaffold stubs: Django project,
  Elixir app, FastAPI Python package, Astro site, Flutter app.
- **Port**: hello-api binds hard-coded `0.0.0.0:8080`
  (services/hello-svc/crates/hello-api/src/main.rs:4–10). Other components use framework
  defaults when run standalone.
- **Entrypoints**: crates/hello-api/src/main.rs; django/hello_project/manage.py;
  ex/hello_app/lib/hello_app/application.ex; py/hello_app/src/hello_app/service.py;
  astro/hello_site; apps/hello-flutter.
- Modules owned: none ("platform validator", README).

## 2. Ownership & Boundaries

- Owns no modules, no data, no events (README table: "Modules: None").
- Upstream/downstream: none; it consumes only platform-level shared packages.
- No TRACEABILITY.yaml.

## 3. Repository Layout

```
services/hello-svc/
  README.md                       # purpose + canonical layout description
  apps/hello-flutter/             # Flutter sample (lib/main.dart, widget_test.dart)
  astro/hello_site/               # Astro site + vitest test (src/test/hello.test.js)
  crates/hello-api/
    src/lib.rs                    # Router with single route
    src/main.rs                   # bind 8080, serve
    tests/health_test.rs          # health integration test
  django/hello_project/           # Django project; urls.py root view; test_root.py
  ex/hello_app/                   # Elixir app skeleton + test
  py/hello_app/src/hello_app/service.py   # FastAPI root
  py/hello_app/tests/ci/test_hello.py
```

Note: README's layout section names `apps/hello-kit`, `django/hello-admin`, `ex/hello_web`,
`py/hello-ml`; actual directories are `apps/hello-flutter`, `django/hello_project`,
`ex/hello_app`, `py/hello_app` — README is stale relative to the tree.

## 4. Capabilities & Functions

| Surface | Endpoint | Implementation | Purpose |
|---|---|---|---|
| Rust | GET / | `health` — services/hello-svc/crates/hello-api/src/lib.rs:7 | `{"ok": true}` JSON |
| Django | GET / | `root` — django/hello_project/hello_project/urls.py:4 | `{"ok": true}` JsonResponse |
| Python | GET / | `root` — py/hello_app/src/hello_app/service.py:8 (`@app.get("/")`) | `{"ok": True}` |
| Elixir | n/a | hello_app.ex is a library skeleton (no web router) | process-lifecycle check |
| Astro | / | astro/hello_site/src/pages/index.astro | static page |
| Flutter | n/a | apps/hello-flutter/lib/main.dart | widget smoke test |

No gRPC, no workers, no LiveViews.

## 5. Data Model

None. No migrations, no tables, no stores, no events produced or consumed.

## 6. Operation

- Build/run: `cargo run -p hello-api` (port 8080); `python manage.py runserver` in
  django/hello_project; FastAPI via service.py app; `mix phx.server` not applicable
  (no Phoenix dep in ex/hello_app/mix.exs beyond bare app); `pnpm dev` in astro/hello_site;
  `flutter run` in apps/hello-flutter.
- Env vars: none read by code.
- Test: `cargo test -p hello-api` (tests/health_test.rs), pytest
  (py/hello_app/tests/ci/test_hello.py), vitest (astro/hello_site), flutter test,
  Django tests/test_root.py.
- Deploy artifact: none (README: "Not deployed to production"); no deploy/ directory.

## 7. Safeguards & Security

- Auth posture: unauthenticated by design; the only endpoints return a constant OK body.
- Idempotency/rate limiting/PII/audit: not applicable; no state is accepted or stored.
- Validation: no request bodies parsed.
- Tests verifying behaviour: health_test.rs, test_root.py, test_hello.py,
  hello.test.js, widget_test.dart all assert the OK contract.
- Known deferrals: none applicable — the entire service is intentionally a stub; note that
  README claims full middleware wiring ("auth, observability, PII wrappers, outbox, HLC...")
  for each sub-component, but the checked-in code is plain frameworks without those layers.

## 8. Observability

- Logging: tracing_subscriber::fmt::init() and a startup line in main.rs:6–8; other
  components use framework defaults.
- Metrics/tracing exporters: none.
- Health: GET / on the Rust, Django, and Python surfaces doubles as the health endpoint;
  no dedicated /health path.
