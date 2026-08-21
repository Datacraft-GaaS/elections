# eel-loadshedding-py

CoDel-inspired priority load shedding Starlette middleware for eel Python services. Mirror of `eel-loadshedding-rs`.

## Purpose in context

Python API services on election night must shed Anonymous and AccreditedMachine traffic before dropping AuthenticatedUser or Admin traffic. This middleware tracks per-request latency and sheds the lowest-priority class first when the target budget is breached. Shed responses return HTTP 503 with `Retry-After` and a `problem+json` body conforming to ADR-0011.

## Public API

```python
from eel_loadshedding import (
    LoadSheddingMiddleware,
    Priority,
    default_priority_extractor,
    shed_response,
)
```

| Symbol | Description |
|---|---|
| `Priority` | Enum: `Admin > AuthenticatedUser > AccreditedMachine > Anonymous` |
| `LoadSheddingMiddleware` | Starlette middleware; configure via `CoDelConfig` |
| `default_priority_extractor` | Reads `X-Eel-Priority` header; replace with JWT-claims extractor in production |
| `shed_response` | Returns a pre-built 503 `Response` with RFC 9457 body |

## Layout

```
eel-loadshedding-py/
  src/eel_loadshedding/
    __init__.py
    middleware.py
  tests/
  pyproject.toml
```

## Usage

```python
from fastapi import FastAPI
from eel_loadshedding import LoadSheddingMiddleware, Priority
from eel_auth import AuthMiddleware  # adds request.state.subject

app = FastAPI()
app.add_middleware(
    LoadSheddingMiddleware,
    target_latency_ms=100,
    interval_ms=500,
    priority_extractor=lambda req: (
        Priority.AuthenticatedUser
        if hasattr(req.state, "subject")
        else Priority.Anonymous
    ),
)
```

## Dependencies

`starlette>=0.40`.

## Install

```bash
uv add eel-loadshedding-py
```

## Test

```bash
uv run pytest packages/eel-loadshedding-py/tests -v
uv run pyright packages/eel-loadshedding-py/src
```

## Related docs

- `packages/eel-loadshedding-rs/README.md` - Rust canonical implementation
- `docs/adr/0009-resilience.md` - load shedding strategy
- `docs/adr/0011-api-conventions.md` - 503 body format and `Retry-After` semantics

## Limitations

Priority extraction is caller-supplied. Wire the JWT subject extractor from `eel-auth-py` to map roles to priority classes correctly.
