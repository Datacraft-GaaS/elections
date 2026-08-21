import Config

config :ecc_live, EccLive.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4010],
  check_origin: false,
  code_reloader: true,
  debug_errors: true,
  secret_key_base: "dev-secret-key-base-ecc-live-at-least-64-chars-xxxxxxxxxxxxxxxxxxxxxxxxxx"

config :ecc_live, EccLive.Repo, show_sensitive_data_on_connection_error: true
