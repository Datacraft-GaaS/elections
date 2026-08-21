import Config

if config_env() == :prod do
  config :speech_analyst_live, SpeechAnalystLive.Endpoint,
    http: [ip: {0, 0, 0, 0}, port: String.to_integer(System.get_env("PORT") || "4112")],
    secret_key_base: System.fetch_env!("SECRET_KEY_BASE"),
    server: true

  config :speech_analyst_live, :speech_integrity_url,
    System.get_env("SPEECH_INTEGRITY_URL") || "http://localhost:8001"
end
