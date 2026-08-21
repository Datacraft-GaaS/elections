import Config

config :observer_workspace, ObserverWorkspaceWeb.Endpoint,
  http: [ip: {0, 0, 0, 0}, port: 4020],
  check_origin: false,
  code_reloader: true,
  debug_errors: true,
  secret_key_base: "eel-obs-dev-secret-key-base-at-least-64-chars-long-placeholder-value",
  watchers: []

config :logger, level: :debug
