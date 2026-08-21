# eel-outbox-py

Transactional outbox: the sole path for publishing events to Kafka from any eel Python service. Wire-compatible with `eel-outbox-rs`.

## Purpose in context

Python FastAPI and Django services must never produce to Kafka directly. This package writes an outbox row inside the caller's SQLAlchemy transaction; Debezium picks it up and delivers it to Kafka with exactly-once semantics.

## Public API

```python
from eel_outbox import emit, MAX_INLINE_BYTES, OutboxSchema
```

| Symbol | Description |
|---|---|
| `emit(conn, envelope)` | Async; inserts the envelope into `outbox` table within `conn`'s transaction |
| `MAX_INLINE_BYTES` | Payload size threshold; larger payloads upload to MinIO first |
| `OutboxSchema` | `create_table_sql` DDL constant |

## Layout

```
eel-outbox-py/
  src/eel_outbox/
    __init__.py
    outbox.py
  tests/
  pyproject.toml
```

## Usage

```python
from sqlalchemy.ext.asyncio import AsyncConnection
from eel_events import Envelope
from eel_outbox import emit

async def handle_form34a(conn: AsyncConnection, envelope: Envelope) -> None:
    # ... business logic inside the same transaction ...
    await emit(conn, envelope)
    # conn.commit() called by the caller
```

## Features (optional)

| Extra | Description |
|---|---|
| `minio` | Uploads large payloads to MinIO before inserting the URL into the outbox |

```bash
uv add "eel-outbox-py[minio]"
```

## Config / env

| Variable | Description |
|---|---|
| `MINIO_ENDPOINT` | MinIO endpoint (required when payload exceeds `MAX_INLINE_BYTES`) |
| `MINIO_BUCKET` | Destination bucket |
| `MINIO_ACCESS_KEY` | MinIO access key |
| `MINIO_SECRET_KEY` | MinIO secret key |

## Dependencies

`pydantic>=2.0`, `sqlalchemy>=2.0`, `eel-events-py`. `minio>=7.0` optional.

## Install

```bash
uv add eel-outbox-py
```

## Test

```bash
uv run pytest packages/eel-outbox-py/tests -v
```

## Related docs

- `packages/eel-outbox-rs/README.md` - Rust canonical implementation and Debezium setup
- `packages/eel-events-py/README.md` - `Envelope` type passed to `emit()`
- `platform/kafka/topics/` - topic routing configuration

## Limitations

`emit()` must be called before `conn.commit()`. If called after commit, no event is published and no error is raised. Large payload upload requires network access to MinIO at the time of the call; plan for this in the service's resilience budget.
