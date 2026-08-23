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
