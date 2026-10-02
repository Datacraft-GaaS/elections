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

defmodule BvvWeb.Objections do
  @moduledoc """
  Context for BVV objection filing and tracking.

  Backed by the objections table from bvi-svc migrations (V6__objections.sql).
  Each accepted filing is broadcast on the `eel:bvv:objections` PubSub topic
  so open consoles pick it up without polling.
  """

  import Ecto.Query

  alias BvvWeb.Objections.Objection
  alias BvvWeb.Repo

  @pubsub BvvWeb.PubSub
  @topic "eel:bvv:objections"

  def subscribe, do: Phoenix.PubSub.subscribe(@pubsub, @topic)

  def list_objections(limit \\ 50) do
    Objection
    |> order_by(desc: :filed_at)
    |> limit(^limit)
    |> Repo.all()
  end

  def get_objection!(id), do: Repo.get!(Objection, id)

  def create_objection(attrs) do
    %Objection{}
    |> Objection.changeset(attrs)
    |> Repo.insert()
    |> case do
      {:ok, objection} ->
        Phoenix.PubSub.broadcast(@pubsub, @topic, {:objection_filed, objection})
        {:ok, objection}

      error ->
        error
    end
  end

  def change_objection(objection \\ %Objection{}, attrs \\ %{}) do
    Objection.changeset(objection, attrs)
  end
end
