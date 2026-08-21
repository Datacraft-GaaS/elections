import Config

config :speech_analyst_live, SpeechAnalystLive.Endpoint,
  http: [ip: {0, 0, 0, 0}, port: 4112],
  secret_key_base: "dev-secret-key-base-speech-analyst-live-at-least-64-chars-xxxxxxxxxxxxxxxx",
  check_origin: false,
  code_reloader: false,
  debug_errors: true,
  server: true

config :logger, level: :debug
