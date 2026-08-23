defmodule BvvWeb.Objections.Objection do
  @moduledoc """
  Schema for the objections table (bvi-svc migration V6__objections.sql).

  `id` is the table primary key; `obj_id` is a second database-generated
  unique identifier surfaced to citizens on filing confirmations.
  """

  use Ecto.Schema

  import Ecto.Changeset

  @objection_types ~w(rectification transfer_request removal_request)

  @primary_key {:id, :binary_id, autogenerate: true}
  @foreign_key_type :binary_id
  schema "objections" do
    field :obj_id, :binary_id, read_after_writes: true
    field :objector_session_hash, :string
    field :target_voter_id, :string
    field :objection_type, :string
    field :justification, :string
    field :evidence_ref, :string
    field :status, :string, default: "pending"
    field :resolution, :string
    field :resolved_at, :utc_datetime_usec
    field :resolved_by, :string

    timestamps(inserted_at: :filed_at, type: :utc_datetime_usec, updated_at: false)
  end

  def changeset(objection, attrs) do
    objection
    |> cast(attrs, [
      :objector_session_hash,
      :target_voter_id,
      :objection_type,
      :justification,
      :evidence_ref
    ])
    |> validate_required([:objector_session_hash, :target_voter_id, :objection_type, :justification])
    |> validate_inclusion(:objection_type, @objection_types)
    |> validate_length(:target_voter_id, max: 64)
    |> validate_length(:justification, min: 10, max: 4000)
    |> validate_length(:evidence_ref, max: 256)
  end

  def objection_types, do: @objection_types
end
