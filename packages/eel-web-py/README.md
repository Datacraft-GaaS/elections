# eel-web-py

Django reusable app for the eel platform: shared auth middleware, SpiceDB integration, base templates, HTMX partials, and Postgres session store.

## Purpose in context

Admin and oversight surfaces for the system use Django. This package is the shared Django app installed in every `services/<svc>/django/<app>/` project. It provides authentication middleware wired to Keycloak via `eel-auth-py`, a SpiceDB permission mixin for class-based views, reusable base templates with the `design-system` tokens, HTMX-compatible partial response helpers, and a Postgres-backed session store.

## Key components

| Component | Location | Description |
|---|---|---|
| `AuthMiddleware` | `middleware.py` | Validates bearer token; sets `request.eel_subject` |
| `SpiceDBMixin` | `views.py` | CBV mixin; calls `require_permission()` in `dispatch()` |
| `EelBaseTemplate` | `templates/eel_web/base.html` | Base HTML with design-system CSS, HTMX, and CSP headers |
| `partial_response` | `htmx.py` | Returns only the `<main>` fragment on `HX-Request` headers |
| `PgSessionStore` | `session.py` | Django session backend backed by the service Postgres |
| `EelWebConfig` | `apps.py` | App config; `default_app_config` |

## Layout

```
eel-web-py/
  src/eel_web/
    __init__.py
    apps.py
    middleware.py
    views.py
    htmx.py
    session.py
    templatetags/
    templates/eel_web/
    migrations/
    management/commands/
  tests/
  pyproject.toml
```

## Usage

```python
# settings.py
INSTALLED_APPS = [
    "eel_web",
    # ... other apps
]
MIDDLEWARE = [
    "eel_web.middleware.AuthMiddleware",
    # ...
]
SESSION_ENGINE = "eel_web.session"
```

## Config / env

| Variable | Description |
|---|---|
| `KEYCLOAK_JWKS_URL` | JWKS endpoint for token validation |
| `SPICEDB_ENDPOINT` | SpiceDB gRPC address |
| `SPICEDB_PRESHARED_KEY` | SpiceDB preshared key |
| `DATABASE_URL` | Postgres DSN for session store |

## Dependencies

`django>=4.2`, `pydantic>=2.0`, `eel-auth-py` (as sibling package).

## Install

```bash
uv add eel-web-py
# Run migrations after install:
python manage.py migrate eel_web
```

## Test

```bash
uv run pytest packages/eel-web-py/tests -v
uv run pyright packages/eel-web-py/src
```

## Related docs

- `packages/eel-auth-py/README.md` - auth primitives used by this package
- `packages/design-system/README.md` - tokens and components referenced in templates
- `docs/adr/0013-admin-surfaces.md` - Django admin conventions

## Limitations

The Postgres session store shares the service database; high session volume should use a dedicated schema or switch to Redis. HTMX partial rendering requires the `HX-Request` header; full-page navigation degrades gracefully to standard Django responses.
