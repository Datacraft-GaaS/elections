import Config

config :queue_map_live, QueueMapLive.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4112],
  secret_key_base: "test-secret-key-base-queue-map-live-at-least-64-chars-xxxxxxxxxxxx",
  server: false

config :logger, level: :warning
config :phoenix, :plug_init_mode, :runtime
