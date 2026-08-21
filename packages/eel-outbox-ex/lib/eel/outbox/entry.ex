defmodule Eel.Outbox.Entry do
  @moduledoc """
  Ecto schema for the `outbox` table.

  Each row represents one CloudEvent that has been committed to the database
  as part of a business transaction but not yet published to Kafka. The relay
  process reads unpublished rows (`published_at IS NULL`) and forwards them.
  """

  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:id, :binary_id, autogenerate: true}
  @timestamps_opts [type: :utc_datetime]

  schema "outbox" do
    field :event_id, :string
    field :event_type, :string
    field :topic, :string
    field :payload, :binary
    field :published_at, :utc_datetime

    timestamps(updated_at: false)
  end

  @required_fields [:event_id, :event_type, :topic, :payload]

  @doc """
  Build a changeset for inserting a new outbox entry.
  """
  @spec changeset(t :: %__MODULE__{}, attrs :: map()) :: Ecto.Changeset.t()
  def changeset(%__MODULE__{} = entry, attrs) do
    entry
    |> cast(attrs, @required_fields ++ [:published_at])
    |> validate_required(@required_fields)
  end
end
