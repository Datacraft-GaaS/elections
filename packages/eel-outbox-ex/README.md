# eel-outbox-ex

Transactional outbox pattern for the IEBC IEMS project, implemented in Elixir.

## Features

- `Eel.Outbox.emit/2` - append a CloudEvent to the `outbox` table as part of an `Ecto.Multi` transaction
- `Eel.Outbox.Entry` - Ecto schema for the outbox table
- `Eel.Outbox.Blob` - optional blob offload to MinIO for large payloads
- `Eel.Outbox.Checks.NoRawKafka` - Credo check that flags direct use of `:brod`, `:kafka_ex`, `KafkaEx`, or `Kafkaex`

## How it works

1. Business logic builds an `Ecto.Multi`.
2. `Eel.Outbox.emit/2` inserts a row into the `outbox` table within the same transaction.
3. A relay process (not included here) polls the table and publishes un-sent rows to Kafka, then marks them as published.

This eliminates the dual-write hazard: if the transaction rolls back, the outbox row is also rolled back.

## Installation

```elixir
{:eel_outbox, path: "../eel-outbox-ex"}
```

## Migration

```elixir
create table(:outbox, primary_key: false) do
  add :id, :binary_id, primary_key: true, default: fragment("gen_random_uuid()")
  add :event_id, :string, null: false
  add :event_type, :string, null: false
  add :topic, :string, null: false
  add :payload, :binary, null: false
  add :published_at, :utc_datetime
  add :inserted_at, :utc_datetime, null: false, default: fragment("now()")
end

create index(:outbox, [:published_at], where: "published_at IS NULL", name: :outbox_unpublished_idx)
```

## Quick start

```elixir
Ecto.Multi.new()
|> MyApp.Orders.insert_order(order_params)
|> Eel.Outbox.emit(envelope)
|> MyApp.Repo.transaction()
```

## Configuration

```elixir
config :eel_outbox, :blob_threshold, 524_288  # 512 KB
```

## Running tests

```
mix deps.get
mix test
```
