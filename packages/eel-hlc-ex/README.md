# eel-hlc-ex

Hybrid Logical Clock (HLC) for Elixir. Part of the IEBC IEMS distributed system toolkit.

An HLC combines a physical wall-clock millisecond timestamp with a monotonic counter and a node identifier. It provides total event ordering across distributed nodes while staying close to real time. The string format is compatible with the Rust (`eel-hlc-rs`) and Python (`eel-hlc-py`) implementations.

## Format

```
"<physical_ms>.<counter>.<node>"
```

Example: `"1723731000123.5.42"`

## Installation

```elixir
# mix.exs
{:eel_hlc, path: "../eel-hlc-ex"}
```

## Usage

```elixir
# Parse a string produced by any IEMS node (Rust, Python, Elixir)
{:ok, hlc} = Eel.Hlc.from_string("1723731000123.5.42")

# Advance for a local event
wall = :erlang.system_time(:millisecond)
next = Eel.Hlc.tick(hlc, wall)

# Merge a received HLC
merged = Eel.Hlc.merge(local_hlc, received_hlc)

# Total ordering
:lt = Eel.Hlc.compare(earlier, later)

# Serialise back
"1723731000123.5.42" = Eel.Hlc.to_string(hlc)
```

### Stateful clock process

```elixir
{:ok, _pid} = Eel.Hlc.Clock.start_link(node: "node-42", name: MyApp.Clock)

# Advance the clock for a local event
hlc = Eel.Hlc.Clock.now(MyApp.Clock)

# Record a clock received from a peer
:ok = Eel.Hlc.Clock.observe(peer_hlc, MyApp.Clock)
```

## Testing

```sh
mix test
```
