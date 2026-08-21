# eel-loadshedding-ex

CoDel-inspired adaptive load shedding for Elixir services.

## Features

- **Controller** - GenServer tracking latency and maintaining CoDel shedding state.
- **Plug** - HTTP plug that rejects low-priority requests with 503 during overload.

## Usage

Add to your supervision tree:

```elixir
{Eel.LoadShedding.Controller, [interval_ms: 100, max_latency_ms: 5]}
```

Add to your Plug pipeline:

```elixir
plug Eel.LoadShedding.Plug, priority_header: "X-Priority"
```

## Tests

```
mix test
```
