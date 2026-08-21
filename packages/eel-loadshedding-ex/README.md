# eel-loadshedding-ex

CoDel-inspired priority load shedding Plug for eel Elixir services. Mirror of `eel-loadshedding-rs`.

## Purpose in context

Phoenix services on election night must shed Anonymous traffic before dropping authenticated traffic. This library provides a Plug that tracks per-request latency and returns 503 with `Retry-After` when the target budget is breached, shedding lowest-priority traffic first.

## Public modules

| Module | Description |
|---|---|
| `Eel.LoadShedding.Plug` | Plug middleware; accepts `CoDelConfig` |
| `Eel.LoadShedding.Priority` | `admin | authenticated_user | accredited_machine | anonymous` |
| `Eel.LoadShedding.Metrics` | `shed_count/1`, `breach_count/0`, `queue_depth/0` |

## Layout

```
eel-loadshedding-ex/
  lib/
    eel_load_shedding/
      plug.ex
      priority.ex
      metrics.ex
  test/
  mix.exs
```

## Usage

```elixir
# endpoint.ex
plug Eel.LoadShedding.Plug,
  target_latency_ms: 100,
  interval_ms: 500,
  priority_extractor: &extract_priority/1

defp extract_priority(conn) do
  if conn.assigns[:eel_subject], do: :authenticated_user, else: :anonymous
end
```

## Dependencies

`plug`, `jason`.

## Install

```elixir
{:eel_load_shedding, path: "../../packages/eel-loadshedding-ex"}
```

```bash
mix deps.get
```

## Test

```bash
mix test
```

## Related docs

- `packages/eel-loadshedding-rs/README.md` - Rust canonical implementation
- `docs/adr/0009-resilience.md` - load shedding strategy
- `docs/adr/0011-api-conventions.md` - 503 body format and `Retry-After` semantics

## Limitations

Priority extraction is caller-supplied. Wire `Eel.Auth.Plug` before this plug so that `conn.assigns.eel_subject` is populated before the priority extractor runs.
