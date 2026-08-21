import Config

config :ecc_live, EccLive.Endpoint,
  url: [host: "localhost"],
  render_errors: [
    formats: [html: EccLive.ErrorHTML, json: EccLive.ErrorJSON],
    layout: false
  ],
  pubsub_server: EccLive.PubSub,
  live_view: [signing_salt: "eel-ecc-lv-salt"]

config :ecc_live, EccLive.Repo,
  database: "ecc_live_dev",
  username: "postgres",
  password: "postgres",
  hostname: "localhost",
  pool_size: 10

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

import_config "#{config_env()}.exs"
