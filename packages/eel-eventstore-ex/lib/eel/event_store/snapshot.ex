defmodule Eel.EventStore.Snapshot do
  @moduledoc """
  Ecto schema for the `event_snapshots` table.
  """

  use Ecto.Schema

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id

  schema "event_snapshots" do
    field :stream_id, :string
    field :version, :integer
    field :state, :map

    timestamps(type: :utc_datetime_usec, updated_at: false)
  end
end
