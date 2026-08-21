defmodule ObserverWorkspaceWeb.Router do
  use ObserverWorkspaceWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {ObserverWorkspaceWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/", ObserverWorkspaceWeb do
    pipe_through :browser

    get "/health", HealthController, :index
    live "/", MissionIndexLive, :index
    live "/missions/:id", MissionWorkspaceLive, :show
    live "/missions/:id/briefings", BriefingCalendarLive, :index
    live "/missions/:id/reports", ReportInboxLive, :index
    live "/joint-statements", JointStatementLive, :index
  end
end
