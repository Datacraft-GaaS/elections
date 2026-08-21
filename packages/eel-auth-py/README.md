# eel-auth-py

JWT/JWKS verification, SpiceDB ReBAC authorization, and audit emission for eel Python services. Mirror of `eel-auth-rs`.

## Purpose in context

FastAPI and Django services validate bearer tokens and check SpiceDB permissions through this package. It provides a Starlette middleware that authenticates every request, a `Subject` extraction type, and a `SpiceDBClient` with fail-closed semantics. `require_permission` is a FastAPI dependency factory for inline permission checks.

## Public API

```python
from eel_auth import (
    JwksCache, verify_token, AuthError,
    Subject, SubjectExtractor,
    SpiceDBClient, CheckResult,
    AuditSink, NoopAuditSink, LogAuditSink,
    AuthMiddleware, require_permission,
)
```

| Symbol | Description |
|---|---|
| `JwksCache` | Async JWKS cache with TTL and kid-miss refetch |
| `verify_token(token, cache)` | Returns `Subject`; raises `AuthError` on failure |
| `Subject` | `sub`, `roles`, `scopes`, `is_service_account` |
| `SubjectExtractor` | Extracts `Subject` from a Starlette `Request` |
| `SpiceDBClient` | Async gRPC client; `check() -> CheckResult`; fail-closed |
| `CheckResult` | `Allow | Deny` |
| `AuthMiddleware` | Starlette middleware; sets `request.state.subject` |
| `require_permission` | FastAPI dependency factory: `Depends(require_permission("rts", "form-34a/x", "submit"))` |

## Layout

```
eel-auth-py/
  src/eel_auth/
    __init__.py
    audit.py
    jwks.py
    middleware.py
    spicedb.py
    subject.py
  tests/
  pyproject.toml
```

## Usage

```python
from fastapi import FastAPI, Depends
from eel_auth import AuthMiddleware, JwksCache, require_permission

app = FastAPI()
cache = JwksCache("https://auth.example.local/realms/eel/protocol/openid-connect/certs")
app.add_middleware(AuthMiddleware, jwks_cache=cache)

@app.post("/tally")
async def submit_tally(
    subject=Depends(require_permission("rts", "tally/2027-general", "submit")),
):
    return {"status": "ok"}
```

## Config / env

| Variable | Description |
|---|---|
| `KEYCLOAK_JWKS_URL` | JWKS endpoint |
| `SPICEDB_ENDPOINT` | SpiceDB gRPC address |
| `SPICEDB_PRESHARED_KEY` | SpiceDB preshared key |

## Dependencies

`httpx>=0.27`, `python-jose[cryptography]>=3.3`, `starlette>=0.40`, `pydantic>=2.0`.

## Install

```bash
uv add eel-auth-py
```

## Test

```bash
uv run pytest packages/eel-auth-py/tests -v
uv run pyright packages/eel-auth-py/src
```

## Related docs

- `packages/authz-schema/README.md` - SpiceDB schema per bounded context
- `packages/eel-auth-rs/README.md` - Rust mirror
- `docs/adr/0004-iam.md` - Keycloak realm, scope taxonomy

## Limitations

`SpiceDBClient` uses gRPC over HTTP/2 via `httpx` async. SpiceDB must be reachable; any connection error causes `CheckResult.Deny`. Plan SpiceDB availability in the service's resilience budget.
