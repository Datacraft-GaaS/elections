defmodule BvvWeb.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      BvvWeb.Repo,
      {Phoenix.PubSub, name: BvvWeb.PubSub},
      BvvWeb.Endpoint
    ]

    opts = [strategy: :one_for_one, name: BvvWeb.Supervisor]
    Supervisor.start_link(children, opts)
  end

  @impl true
  def config_change(changed, _new, removed) do
    BvvWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
