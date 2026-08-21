defmodule SpeechAnalystLive.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Phoenix.PubSub, name: SpeechAnalystLive.PubSub},
      SpeechAnalystLive.Endpoint
    ]

    opts = [strategy: :one_for_one, name: SpeechAnalystLive.Supervisor]
    Supervisor.start_link(children, opts)
  end

  @impl true
  def config_change(changed, _new, removed) do
    SpeechAnalystLive.Endpoint.config_change(changed, removed)
    :ok
  end
end
