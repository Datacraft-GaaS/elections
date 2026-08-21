# eel-eventstore-py

Append-only event store with optimistic concurrency, snapshots, and aggregate replay.

## Install

```
uv add eel-eventstore-py
```

## Usage

```python
from eel_eventstore import EventStore, StoredEvent, OptimisticConcurrencyError

store = EventStore(session_factory)
await store.append_events("stream-1", expected_version=0, events=[event])
```
