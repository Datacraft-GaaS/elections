import Config

if config_env() == :prod do
  config :observer_workspace, ObserverWorkspaceWeb.Endpoint,
    url: [host: System.get_env("PHX_HOST") || "localhost", port: 443, scheme: "https"],
    http: [ip: {0, 0, 0, 0}, port: String.to_integer(System.get_env("PORT") || "4020")],
    secret_key_base: System.fetch_env!("SECRET_KEY_BASE")
end
