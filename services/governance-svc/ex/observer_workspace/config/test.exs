import Config

config :observer_workspace, ObserverWorkspaceWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4022],
  secret_key_base: "eel-obs-test-secret-key-base-at-least-64-chars-long-placeholder-value",
  server: false

config :logger, level: :warning
config :phoenix, :plug_init_mode, :runtime
config :phoenix_live_view, enable_targets: true
config :swoosh, :api_client, false
