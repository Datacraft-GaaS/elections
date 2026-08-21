# eel-events-py

CloudEvents envelope, Ed25519 signing, Karapace schema client, and consumer middleware for eel Python services. Wire-compatible with `eel-events-rs`.

## Purpose in context

Python FastAPI workers and Django admin services that produce or consume eel events use this package. It mirrors the Rust crate's `Envelope`, `EventBuilder`, and `EventConsumer` with an async Python implementation. Signing uses PyNaCl (Ed25519).

## Public API

```python
from eel_events import (
    Envelope, DataClass,
    EventBuilder, SpiffeKeyProvider,
    KarapaceClient,
    EventConsumer, DedupStore, ConsumerMiddleware,
    canonicalize,
)
```

| Symbol | Description |
|---|---|
| `Envelope` | Pydantic v2 model of the signed CloudEvent envelope |
| `DataClass` | Enum A/B/C/D/E matching ADR-0016 tiers |
| `EventBuilder` | Fluent builder; `.build(provider)` returns a signed `Envelope` |
| `SpiffeKeyProvider` | Loads Ed25519 signing key from a SPIFFE SVID path |
| `KarapaceClient` | Async HTTP client for Karapace schema registry |
| `EventConsumer` | Async consumer with dedup and gap detection |
| `DedupStore` | Protocol for idempotency store (Postgres-backed in production) |
| `ConsumerMiddleware` | ASGI middleware for consumer-side envelope validation |
| `canonicalize` | RFC 8785 JSON canonicalization |

## Layout

```
eel-events-py/
  src/eel_events/
    __init__.py
    builder.py
    consumer.py
    envelope.py
    karapace.py
    rfc8785.py
    signing.py
  tests/
  pyproject.toml
```

## Usage

```python
import asyncio
from eel_events import EventBuilder, DataClass, SpiffeKeyProvider

provider = SpiffeKeyProvider("/var/run/spiffe/svid.key")

envelope = await EventBuilder() \
    .event_type("eel.rts.form34a_captured.v1") \
    .source("spiffe://eel.local/service/rts-backend") \
    .stream_id("01950f8c-0000-7000-0000-000000000001") \
    .seq(0) \
    .data({"station_id": "47-001"}) \
    .data_class(DataClass.B) \
    .build(provider)

print(envelope.model_dump_json())
```

## Config / env

| Variable | Description |
|---|---|
| `KARAPACE_URL` | Base URL of the Karapace schema registry |
| `KARAPACE_USERNAME` | Basic auth username (optional) |
| `KARAPACE_PASSWORD` | Basic auth password (optional) |

## Dependencies

`pydantic>=2.0`, `httpx>=0.27`, `pynacl>=1.5`, `uuid6>=2024.1.12`.

## Install

```bash
uv add eel-events-py
```

## Test

```bash
uv run pytest packages/eel-events-py/tests -v
uv run pyright packages/eel-events-py/src
```

## Related docs

- `packages/event-schemas/README.md` - JSON Schema per event type
- `packages/eel-outbox-py/README.md` - sole publication path for Python services
- `docs/EVENT-CONVENTIONS.md` - envelope field semantics

## Limitations

`SpiffeKeyProvider` reads the key file at construction time; rotate the key by restarting the process or calling `provider.reload()`. The consumer does not manage Kafka offsets; use an `aiokafka` consumer loop and pass each message to `EventConsumer.process()`.
