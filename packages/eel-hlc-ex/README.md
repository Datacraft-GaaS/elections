# eel-hlc-ex

Hybrid Logical Clock for eel Elixir services: wire-compatible with `eel-hlc-rs` and `eel-hlc-py`.

## Purpose in context

Phoenix/LiveView services that produce or consume eel events need HLC timestamps that interoperate with Rust and Python nodes. This library provides the `Hlc` struct and `HlcClock` GenServer in pure Elixir with no native dependencies.

## Public modules

| Module | Description |
|---|---|
| `Eel.Hlc` | `from_string/1`, `to_string/1`, `compare/2` |
| `Eel.Hlc.Clock` | GenServer; `now/1`, `receive/2`, `start_link/1` |

## Layout

```
eel-hlc-ex/
  lib/
    eel_hlc.ex
    eel_hlc/clock.ex
  test/
  mix.exs
```

## Usage

```elixir
# Start the clock (add to your supervision tree):
{Eel.Hlc.Clock, node_id: 3}

# Issue a timestamp:
hlc = Eel.Hlc.Clock.now(:eel_hlc_clock)
IO.puts(Eel.Hlc.to_string(hlc))  # "1723731000123.0.3"

# Advance on message receipt:
remote = Eel.Hlc.from_string!("1723731000200.1.7")
updated = Eel.Hlc.Clock.receive(:eel_hlc_clock, remote)
```

## Dependencies

No runtime dependencies. Elixir >= 1.17.

## Install

```elixir
# mix.exs
{:eel_hlc, path: "../../packages/eel-hlc-ex"}
```

```bash
mix deps.get
```

## Test

```bash
mix test
mix dialyzer
```

## Related docs

- `packages/eel-hlc-rs/README.md` - canonical wire format
- `packages/eel-events-ex/README.md` - HLC embedded in every envelope

## Limitations

`Eel.Hlc.Clock` is a GenServer; register it in your application supervision tree with a unique name per node. Multiple clocks on the same node are supported but each must have a distinct node ID.
