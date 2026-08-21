# eel-outbox-ex

Transactional outbox for eel Elixir services: the sole path for publishing events to Kafka. Mirror of `eel-outbox-rs`.

## Purpose in context

Elixir Phoenix services insert outbox rows inside their Ecto `Multi` transactions. Debezium CDC picks them up and publishes to Kafka with exactly-once semantics. This library provides the Ecto schema, the `emit/3` function for use inside `Ecto.Multi`, and migration helpers.

## Public modules

| Module | Description |
|---|---|
| `Eel.Outbox` | `emit/3` - inserts an outbox row via `Ecto.Multi` |
| `Eel.Outbox.Envelope` | Ecto schema matching the `outbox` table |
| `Eel.Outbox.Schema` | `create_table_sql/0` DDL string |

## Layout

```
eel-outbox-ex/
  lib/
    eel_outbox.ex
    eel_outbox/
      envelope.ex
      schema.ex
  priv/repo/migrations/
  test/
  mix.exs
```

## Usage

```elixir
alias Eel.Outbox
alias Eel.Events.Envelope

multi =
  Ecto.Multi.new()
  |> Ecto.Multi.insert(:voter, voter_changeset)
  |> Outbox.emit(:voter_registered, %Envelope{
    id: uuid7str(),
    event_type: "eel.bvr.voter_registered.v1",
    source: "spiffe://eel.local/service/bvr-backend",
    data: %{voter_id: voter.id},
    eel_election_id: "2027-general"
  })

{:ok, _} = Repo.transaction(multi)
```

## Dependencies

`ecto`, `jason`. `finch` optional for large-payload MinIO upload.

## Install

```elixir
{:eel_outbox, path: "../../packages/eel-outbox-ex"}
```

```bash
mix deps.get
mix ecto.migrate
```

## Config

```elixir
config :eel_outbox,
  minio_endpoint: System.get_env("MINIO_ENDPOINT"),
  minio_bucket: System.get_env("MINIO_BUCKET"),
  minio_access_key: System.get_env("MINIO_ACCESS_KEY"),
  minio_secret_key: System.get_env("MINIO_SECRET_KEY")
```

## Test

```bash
mix test
```

## Related docs

- `packages/eel-outbox-rs/README.md` - Rust canonical implementation and Debezium setup
- `packages/eel-events-ex/README.md` - `Envelope` type passed to `emit/3`
- `platform/kafka/topics/` - topic routing configuration

## Limitations

`emit/3` must be called inside an `Ecto.Multi` that is executed with `Repo.transaction/1`. Calling `emit/3` outside a transaction will fail at runtime. Large payloads (> `max_inline_bytes`) require the `finch` dependency and a reachable MinIO endpoint.
