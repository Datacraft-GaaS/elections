# eel-eventstore-ex

Elixir event store library for the IEBC IEMS project. Provides optimistic-concurrency event appending and aggregate replay via Ecto.

## Features

- `Eel.EventStore.Aggregate` behaviour for domain aggregates
- `Eel.EventStore.Stream.append_events/4` with `SELECT ... FOR UPDATE` optimistic concurrency
- `Eel.EventStore.Replay.replay/4` with snapshot support
- Ecto schemas for `events` and `event_snapshots` tables

## Usage

```elixir
# Define an aggregate
defmodule MyAggregate do
  @behaviour Eel.EventStore.Aggregate

  def initial_state, do: %{count: 0}
  def apply(state, %{type: "inc"}), do: %{state | count: state.count + 1}
  def apply(state, _), do: state
  def snapshot(state), do: {:ok, state}
end

# Append events
Eel.EventStore.append_events(MyRepo, "stream-1", [%{type: "inc", data: %{}}], 0)

# Replay
{:ok, state, version} = Eel.EventStore.replay(MyRepo, "stream-1", MyAggregate)
```

## Dependencies

- `ecto ~> 3.12`
- `ecto_sql ~> 3.12`
- `jason ~> 1.4`
