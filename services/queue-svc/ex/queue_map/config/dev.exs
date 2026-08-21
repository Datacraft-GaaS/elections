import Config

config :queue_map_live, QueueMapLive.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4111],
  check_origin: false,
  code_reloader: true,
  debug_errors: true,
  secret_key_base: "dev-secret-key-base-queue-map-live-at-least-64-chars-xxxxxxxxxxxx",
  watchers: []

config :logger, :console, format: "[$level] $message\n"
config :phoenix, :stacktrace_depth, 20
config :phoenix, :plug_init_mode, :runtime
