import Config

config :speech_analyst_live, SpeechAnalystLive.Endpoint,
  url: [host: "localhost"],
  render_errors: [
    formats: [html: SpeechAnalystLive.ErrorHTML, json: SpeechAnalystLive.ErrorJSON],
    layout: false
  ],
  pubsub_server: SpeechAnalystLive.PubSub,
  live_view: [signing_salt: "eel-speech-lv-salt"]

config :speech_analyst_live, :speech_integrity_url, "http://localhost:8001"

config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

config :phoenix, :json_library, Jason

import_config "#{config_env()}.exs"
