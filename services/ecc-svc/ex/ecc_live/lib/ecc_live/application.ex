defmodule EccLive.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      EccLive.Repo,
      {Phoenix.PubSub, name: EccLive.PubSub},
      EccLive.Endpoint
    ]

    opts = [strategy: :one_for_one, name: EccLive.Supervisor]
    Supervisor.start_link(children, opts)
  end

  @impl true
  def config_change(changed, _new, removed) do
    EccLive.Endpoint.config_change(changed, removed)
    :ok
  end
end
