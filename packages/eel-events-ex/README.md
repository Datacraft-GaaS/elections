# eel-events-ex

CloudEvents envelope with eel platform extensions, Ed25519 signing, and Broadway-based consumer for Elixir services. Mirror of `eel-events-rs`.

## Purpose in context

Phoenix LiveView services that produce or consume eel events use this library. It provides the `Envelope` struct with all eel extensions, an `EventBuilder` pipeline, a Broadway pipeline module for Kafka consumption, and an Ecto type for persisting `DataClass` values.

## Public modules

| Module | Description |
|---|---|
| `Eel.Events.Envelope` | Struct with all CloudEvent + eel extension fields |
| `Eel.Events.Builder` | Fluent pipeline: `new/0`, `event_type/2`, `source/2`, `data/2`, `build/2` |
| `Eel.Events.DataClass` | Enum module A/B/C/D/E with Ecto type |
| `Eel.Events.Consumer` | Broadway pipeline module for Kafka topic consumption |
| `Eel.Events.Karapace` | HTTP client for Karapace schema registry |

## Layout

```
eel-events-ex/
  lib/
    eel_events.ex
    eel_events/
      envelope.ex
      builder.ex
      data_class.ex
      consumer.ex
      karapace.ex
  test/
  mix.exs
```

## Usage

```elixir
alias Eel.Events.{Builder, DataClass}

{:ok, envelope} =
  Builder.new()
  |> Builder.event_type("eel.rts.form34a_captured.v1")
  |> Builder.source("spiffe://eel.local/service/rts-backend")
  |> Builder.stream_id("01950f8c-0000-7000-0000-000000000001")
  |> Builder.seq(0)
  |> Builder.data(%{station_id: "47-001"})
  |> Builder.data_class(DataClass.b())
  |> Builder.build(signing_key)

IO.inspect(envelope.specversion)  # "1.0"
```

## Config

```elixir
# config/config.exs
config :eel_events,
  karapace_url: System.get_env("KARAPACE_URL"),
  karapace_username: System.get_env("KARAPACE_USERNAME"),
  karapace_password: System.get_env("KARAPACE_PASSWORD")
```

## Dependencies

`finch`, `jason`, `broadway`. `ecto` optional.

## Install

```elixir
{:eel_events, path: "../../packages/eel-events-ex"}
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

- `packages/event-schemas/README.md` - JSON Schema per event type
- `packages/eel-outbox-ex/README.md` - sole publication path for Elixir services
- `docs/EVENT-CONVENTIONS.md` - envelope field semantics

## Limitations

Signing uses the `:crypto` OTP application (Ed25519 via `:crypto.sign/5`). The Rustler NIF from `eel-crypto-rs` is available for post-quantum signing; wire it in when PQ features are enabled in the service.
