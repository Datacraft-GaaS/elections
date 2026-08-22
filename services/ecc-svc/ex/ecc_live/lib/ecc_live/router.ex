defmodule EccLive.Router do
  use EccLive, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {EccLive.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  # Roles: :chair, :commissioner, :director, :analyst, :observer
  scope "/", EccLive do
    pipe_through :browser

    live "/dashboard", Live.DashboardLive, :index
    live "/alerts", Live.AlertsLive, :index
    live "/s44a", Live.S44aLive, :index
    live "/kits", Live.KitsLive, :index
    live "/cover-tamper", Live.CoverTamperLive, :index
    live "/cover-tamper/history", Live.CoverHistoryLive, :index
    live "/telemetry", Live.TelemetryLive, :index
    live "/telemetry/query", Live.TelemetryQueryLive, :index
  end

  scope "/health", EccLive do
    pipe_through :api
    get "/", Controllers.HealthController, :index
  end
end
