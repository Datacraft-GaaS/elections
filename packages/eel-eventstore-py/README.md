# eel-eventstore-py

Append-only event store with optimistic concurrency for eel Python services. Wire-compatible with `eel-eventstore-rs`.

## Purpose in context

Python domain services that use event sourcing depend on this package to append and replay event streams from Postgres. The schema is shared with the Rust implementation; both languages read the same `event_streams` and `snapshots` tables.

## Public API

```python
from eel_eventstore import (
    StoredEvent, AggregateSnapshot,
    Aggregate,
    EventStore, OptimisticConcurrencyError, EventStoreSchema,
)
```

| Symbol | Kind | Description |
|---|---|---|
| `StoredEvent` | Pydantic model | Stored event row: `stream_id`, `seq`, `hlc`, `event_type`, `payload` |
| `AggregateSnapshot` | Pydantic model | Serialized aggregate at a given `seq` |
| `Aggregate` | Protocol | `apply(event: StoredEvent) -> None` |
| `EventStore` | class | `append()`, `load()`, `snapshot()`, `replay()` (async) |
| `OptimisticConcurrencyError` | exception | Raised on seq conflict |
| `EventStoreSchema` | class | DDL helpers: `create_tables_sql` |

## Layout

```
eel-eventstore-py/
  src/eel_eventstore/
    __init__.py
    store.py
    models.py
    protocols.py
  tests/
  pyproject.toml
```

## Usage

```python
import asyncio
from sqlalchemy.ext.asyncio import create_async_engine
from eel_eventstore import EventStore, StoredEvent

class BallotBox:
    count: int = 0
    def apply(self, ev: StoredEvent) -> None:
        if ev.event_type == "eel.rts.vote_cast.v1":
            self.count += 1

async def main():
    engine = create_async_engine("postgresql+asyncpg://...")
    store = EventStore(engine)
    await store.create_tables()

    await store.append("stream-1", seq=0, event_type="eel.rts.vote_cast.v1", payload={})
    box = BallotBox()
    await store.replay("stream-1", box)
    print(box.count)  # 1
```

## Dependencies

`pydantic>=2.0`, `sqlalchemy>=2.0`.

## Install

```bash
uv add eel-eventstore-py
```

## Test

```bash
uv run pytest packages/eel-eventstore-py/tests -v
uv run pyright packages/eel-eventstore-py/src
```

## Related docs

- `packages/eel-eventstore-rs/README.md` - canonical implementation and schema DDL
- `packages/eel-outbox-py/README.md` - publishes stored events to Kafka

## Limitations

Requires `asyncpg` driver for SQLAlchemy async. Snapshot compaction (deleting old events) is not implemented. Large replays (> 100k events) should use the `replay_from_snapshot()` path to avoid memory pressure.
