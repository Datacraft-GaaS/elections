defmodule PortalLive.Application do
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Phoenix.PubSub, name: PortalLive.PubSub},
    ]

    opts = [strategy: :one_for_one, name: PortalLive.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
