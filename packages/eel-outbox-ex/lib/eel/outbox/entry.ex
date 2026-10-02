# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
