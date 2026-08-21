defmodule ObserverWorkspaceWeb.Endpoint do
  use Phoenix.Endpoint, otp_app: :observer_workspace

  @session_options [
    store: :cookie,
    key: "_observer_workspace_key",
    signing_salt: "eel-obs-salt",
    same_site: "Lax"
  ]

  socket "/live", Phoenix.LiveView.Socket,
    websocket: [connect_info: [session: @session_options]],
    longpoll: [connect_info: [session: @session_options]]

  plug Plug.Static,
    at: "/",
    from: :observer_workspace,
    gzip: false,
    only: ObserverWorkspaceWeb.static_paths()

  plug Plug.RequestId
  plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]

  plug Plug.Parsers,
    parsers: [:urlencoded, :multipart, :json],
    pass: ["*/*"],
    json_decoder: Phoenix.json_library()

  plug Plug.MethodOverride
  plug Plug.Head
  plug Plug.Session, @session_options
  plug ObserverWorkspaceWeb.Router
end
