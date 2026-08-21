defmodule SpeechAnalystLive.Endpoint do
  use Phoenix.Endpoint, otp_app: :speech_analyst_live

  @session_options [
    store: :cookie,
    key: "_speech_analyst_live_key",
    signing_salt: "eel-speech-salt",
    same_site: "Lax"
  ]

  socket "/live", Phoenix.LiveView.Socket,
    websocket: [connect_info: [session: @session_options]]

  plug Plug.Static,
    at: "/",
    from: :speech_analyst_live,
    gzip: false,
    only: SpeechAnalystLive.static_paths()

  plug Plug.RequestId
  plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]

  plug Plug.Parsers,
    parsers: [:urlencoded, :multipart, :json],
    pass: ["*/*"],
    json_decoder: Phoenix.json_library()

  plug Plug.MethodOverride
  plug Plug.Head
  plug Plug.Session, @session_options
  plug SpeechAnalystLive.Router
end
