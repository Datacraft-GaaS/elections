import Config

config :bvv_web, BvvWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4002],
  secret_key_base: "test-secret-key-base-at-least-64-chars-long-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  server: false

config :bvv_web, BvvWeb.Repo,
  database: "bvv_web_test#{System.get_env("MIX_TEST_PARTITION")}",
  pool: Ecto.Adapters.SQL.Sandbox

config :logger, level: :warning
config :phoenix, :plug_init_mode, :runtime
