import Config

config :ecc_live, EccLive.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4013],
  secret_key_base: "test-secret-key-base-ecc-live-at-least-64-chars-xxxxxxxxxxxxxxxxxxxxxxxxxx",
  server: false

config :ecc_live, EccLive.Repo,
  database: "ecc_live_test#{System.get_env("MIX_TEST_PARTITION")}",
  pool: Ecto.Adapters.SQL.Sandbox

config :logger, level: :warning
config :phoenix, :plug_init_mode, :runtime
