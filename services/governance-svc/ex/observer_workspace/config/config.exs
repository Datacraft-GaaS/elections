import Config

config :observer_workspace,
  ecto_repos: []

config :observer_workspace, ObserverWorkspaceWeb.Endpoint,
  url: [host: "localhost"],
  adapter: Bandit.PhoenixAdapter,
  render_errors: [
    formats: [html: ObserverWorkspaceWeb.ErrorHTML, json: ObserverWorkspaceWeb.ErrorJSON],
    layout: false
  ],
  pubsub_server: ObserverWorkspace.PubSub,
  live_view: [signing_salt: "eel-obs-lv-salt"]

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

# Swoosh: use Finch adapter (no hackney needed)
config :swoosh, :api_client, Swoosh.ApiClient.Finch
config :swoosh, :finch_name, ObserverWorkspace.Finch

import_config "#{config_env()}.exs"
