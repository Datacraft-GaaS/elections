import Config

config :bvv_web, BvvWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4000],
  check_origin: false,
  code_reloader: true,
  debug_errors: true,
  secret_key_base: "dev-secret-key-base-at-least-64-chars-long-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

config :bvv_web, BvvWeb.Repo, show_sensitive_data_on_connection_error: true
