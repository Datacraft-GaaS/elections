import Config

if config_env() == :prod do
  secret_key_base = System.fetch_env!("SECRET_KEY_BASE")
  config :bvv_web, BvvWeb.Endpoint,
    http: [ip: {0, 0, 0, 0}, port: String.to_integer(System.get_env("PORT") || "4000")],
    secret_key_base: secret_key_base

  config :bvv_web, BvvWeb.Repo,
    url: System.fetch_env!("DATABASE_URL"),
    pool_size: String.to_integer(System.get_env("POOL_SIZE") || "10")
end
