defmodule BvvWeb.Endpoint do
  use Phoenix.Endpoint, otp_app: :bvv_web

  @session_options [
    store: :cookie,
    key: "_bvv_web_key",
    signing_salt: "eel-bvv-salt",
    same_site: "Lax"
  ]

  socket "/live", Phoenix.LiveView.Socket, websocket: [connect_info: [session: @session_options]]

  plug Plug.Static, at: "/", from: :bvv_web, gzip: false, only: BvvWeb.static_paths()

  plug Plug.RequestId
  plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]
  plug Plug.Parsers, parsers: [:urlencoded, :multipart, :json], pass: ["*/*"], json_decoder: Phoenix.json_library()
  plug Plug.MethodOverride
  plug Plug.Head
  plug Plug.Session, @session_options
  plug BvvWeb.Router
end
