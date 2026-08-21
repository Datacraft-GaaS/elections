# eel-events-ex

CloudEvents 1.0 envelope library for the IEBC IEMS project, implemented in Elixir.

## Features

- `Eel.Events.Envelope` - struct covering all CloudEvents core attributes plus IEBC-specific `eel_*` extensions (stream ID, HLC, data class, signatures, etc.)
- `Eel.Events.Builder` - fluent construction API; auto-generates UUIDv4 IDs and timestamps
- `Eel.Events.CanonicalJson` - RFC 8785 canonical JSON (key-sorted, no whitespace, shortest-round-trip numbers)
- `Eel.Events.Signing` - Ed25519 sign and verify via OTP `:crypto` (no NIFs required)
- `Eel.Events.Karapace` - async Karapace schema registry client backed by Finch
- `Eel.Events.Consumer` - behaviour for event handler modules

## Installation

```elixir
{:eel_events, path: "../eel-events-ex"}
```

## Quick start

```elixir
{:ok, envelope} =
  Eel.Events.Builder.new("eel.rts.form34a_captured.v1", "spiffe://eel.local/rts", form_data)
  |> Eel.Events.Builder.with_election("2027-general")
  |> Eel.Events.Builder.with_data_class("B")
  |> Eel.Events.Builder.build()

{pub_key, priv_key} = :crypto.generate_key(:eddsa, :ed25519)
{:ok, signed} = Eel.Events.Signing.sign(envelope, priv_key)
:ok = Eel.Events.Signing.verify(signed, pub_key)
```

## Configuration

```elixir
config :eel_events, :karapace_url, "https://karapace.example.com"
```

## Running tests

```
mix deps.get
mix test
```
