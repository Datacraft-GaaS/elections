defmodule SpeechAnalystLive.Router do
  use SpeechAnalystLive, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {SpeechAnalystLive.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/", SpeechAnalystLive do
    pipe_through :browser

    live "/dashboard", Live.DashboardLive, :index
    live "/flags", Live.FlagsLive, :index
    live "/adjudication", Live.AdjudicationLive, :index
  end

  scope "/health", SpeechAnalystLive do
    pipe_through :api
    get "/", Controllers.HealthController, :index
  end
end
