defmodule Eel.EventStore.Event do
  @moduledoc """
  Ecto schema for the `events` table.
  """

  use Ecto.Schema

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id

  schema "events" do
    field :stream_id, :string
    field :version, :integer
    field :type, :string
    field :data, :map
    field :metadata, :map
    field :prev_hash, :string

    timestamps(type: :utc_datetime_usec, updated_at: false)
  end
end
