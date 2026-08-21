defmodule EccLive.Endpoint do
  use Phoenix.Endpoint, otp_app: :ecc_live

  @session_options [
    store: :cookie,
    key: "_ecc_live_key",
    signing_salt: "eel-ecc-salt",
    same_site: "Lax"
  ]

  socket "/live", Phoenix.LiveView.Socket, websocket: [connect_info: [session: @session_options]]

  plug Plug.Static, at: "/", from: :ecc_live, gzip: false, only: EccLive.static_paths()

  plug Plug.RequestId
  plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]
  plug Plug.Parsers, parsers: [:urlencoded, :multipart, :json], pass: ["*/*"], json_decoder: Phoenix.json_library()
  plug Plug.MethodOverride
  plug Plug.Head
  plug Plug.Session, @session_options
  plug EccLive.Router
end
