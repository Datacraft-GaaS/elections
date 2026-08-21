import Config

config :bvv_web, BvvWeb.Endpoint,
  url: [host: "localhost"],
  render_errors: [
    formats: [html: BvvWeb.ErrorHTML, json: BvvWeb.ErrorJSON],
    layout: false
  ],
  pubsub_server: BvvWeb.PubSub,
  live_view: [signing_salt: "eel-bvv-lv-salt"]

config :bvv_web, BvvWeb.Repo,
  database: "bvv_web_dev",
  username: "postgres",
  password: "postgres",
  hostname: "localhost",
  pool_size: 10

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

import_config "#{config_env()}.exs"
