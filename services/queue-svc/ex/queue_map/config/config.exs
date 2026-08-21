import Config

config :queue_map_live, QueueMapLive.Endpoint,
  url: [host: "localhost"],
  render_errors: [
    formats: [html: QueueMapLive.ErrorHTML, json: QueueMapLive.ErrorJSON],
    layout: false
  ],
  pubsub_server: QueueMapLive.PubSub,
  live_view: [signing_salt: "eel-queue-lv-salt"]

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

import_config "#{config_env()}.exs"
