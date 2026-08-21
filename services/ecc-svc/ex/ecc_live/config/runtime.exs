import Config

if config_env() == :prod do
  config :ecc_live, EccLive.Endpoint,
    http: [port: String.to_integer(System.get_env("PORT") || "4010")],
    secret_key_base: System.fetch_env!("SECRET_KEY_BASE")

  config :ecc_live, EccLive.Repo,
    url: System.fetch_env!("DB_URL"),
    pool_size: String.to_integer(System.get_env("POOL_SIZE") || "10")
end
