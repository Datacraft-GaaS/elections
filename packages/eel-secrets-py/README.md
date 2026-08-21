# eel-secrets-py

OpenBao (Vault-compatible) secrets client for eel Python services: dynamic Postgres credentials, transit encryption, and PKI leaf certs. Mirror of `eel-secrets-rs`.

## Purpose in context

Python FastAPI workers never hold long-lived database passwords. On startup they call `PgDynamicCred` to lease short-lived credentials from OpenBao and renew them automatically. Transit encryption wraps sensitive fields before they are stored in Postgres.

## Public API

```python
from eel_secrets import OpenBaoClient, Transit, PgDynamicCred, PgCredential
```

| Symbol | Description |
|---|---|
| `OpenBaoClient` | Base async HTTP client for OpenBao API |
| `PgDynamicCred` | Leases and auto-renews Postgres credentials |
| `PgCredential` | Pydantic model: `username`, `password`, `lease_duration` |
| `Transit` | `encrypt(plaintext) -> str`, `decrypt(ciphertext) -> bytes` |

## Layout

```
eel-secrets-py/
  src/eel_secrets/
    __init__.py
    client.py
    dynamic_creds.py
    transit.py
  tests/
  pyproject.toml
```

## Usage

```python
import asyncio
from eel_secrets import PgDynamicCred, Transit
from sqlalchemy.ext.asyncio import create_async_engine

async def startup():
    cred = await PgDynamicCred.create(
        addr="http://openbao:8200",
        token="s.service-token",
        role_path="database/creds/rts-backend",
    )
    engine = create_async_engine(cred.dsn())
    # cred.start_renewal_task() to auto-renew in background

    transit = Transit("http://openbao:8200", "s.service-token", "eel-data")
    cipher = await transit.encrypt(b"voter-id-plaintext")
    plain = await transit.decrypt(cipher)
```

## Config / env

| Variable | Description |
|---|---|
| `OPENBAO_ADDR` | OpenBao server address |
| `OPENBAO_TOKEN` | Service token |
| `OPENBAO_TRANSIT_KEY` | Transit key name |

## Dependencies

`httpx>=0.27`, `pydantic>=2.0`.

## Install

```bash
uv add eel-secrets-py
```

## Test

```bash
uv run pytest packages/eel-secrets-py/tests -v
uv run pyright packages/eel-secrets-py/src
```

## Related docs

- `packages/eel-secrets-rs/README.md` - Rust canonical implementation
- `docs/adr/0007-secrets.md` - OpenBao role design and lease policies

## Limitations

Credential renewal is performed by an `asyncio` background task; if the event loop is blocked, renewal may be delayed. Ensure the service's health check fails before the lease expires to trigger pod replacement.
