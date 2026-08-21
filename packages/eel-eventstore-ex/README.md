# eel-eventstore-ex

Append-only event store with optimistic concurrency for eel Elixir services. Shares the Postgres schema with `eel-eventstore-rs`.

## Purpose in context

Elixir/Phoenix services that use event sourcing store and replay event streams through this library. It wraps Ecto queries against the same `event_streams` and `snapshots` tables used by Rust services, providing bidirectional interoperability.

## Public modules

| Module | Description |
|---|---|
| `Eel.EventStore` | `append/4`, `load/2`, `replay/3`, `snapshot/3`, `load_since/3` |
| `Eel.EventStore.StoredEvent` | Ecto schema for event stream rows |
| `Eel.EventStore.Snapshot` | Ecto schema for snapshot rows |
| `Eel.EventStore.OptimisticConcurrencyError` | Exception on seq conflict |

## Layout

```
eel-eventstore-ex/
  lib/
    eel_event_store.ex
    eel_event_store/
      stored_event.ex
      snapshot.ex
      error.ex
  priv/repo/migrations/
  test/
  mix.exs
```

## Usage

```elixir
alias Eel.EventStore

# Append an event at seq 0 (raises on conflict):
{:ok, _} = EventStore.append(repo, "stream-uuid", 0, %{
  event_type: "eel.rts.tally_submitted.v1",
  payload: %{station_id: "47-001"}
})

# Replay into an aggregate:
events = EventStore.load(repo, "stream-uuid")
state = Enum.reduce(events, %{tallied: false}, &apply_event/2)
```

## Dependencies

`ecto`, `ecto_sql`, `jason`.

## Install

```elixir
{:eel_event_store, path: "../../packages/eel-eventstore-ex"}
```

```bash
mix deps.get
mix ecto.migrate  # runs the shared schema migration
```

## Test

```bash
mix test
```

## Related docs

- `packages/eel-eventstore-rs/README.md` - canonical schema DDL
- `packages/eel-outbox-ex/README.md` - publishes stored events to Kafka

## Limitations

Snapshot compaction is not implemented. Use the `load_since/3` function to replay only events after a snapshot rather than the full stream. Elixir services should not share a Postgres connection pool with Rust services; use separate pool configurations pointing to the same database.
