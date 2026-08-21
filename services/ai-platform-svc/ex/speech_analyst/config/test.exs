import Config

config :speech_analyst_live, SpeechAnalystLive.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4113],
  secret_key_base: "test-secret-key-base-speech-analyst-live-at-least-64-chars-xxxxxxxxxxxxxxxxxx",
  server: false

config :logger, level: :warning
config :phoenix, :plug_init_mode, :runtime
