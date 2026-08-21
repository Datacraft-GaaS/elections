defmodule QueueMapLive.Router do
  use QueueMapLive, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {QueueMapLive.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  pipeline :sse do
    plug :accepts, ["text/event-stream", "*/*"]
  end

  scope "/", QueueMapLive do
    pipe_through :browser

    live "/", Live.StationMapLive, :index
  end

  scope "/sse", QueueMapLive do
    pipe_through :sse
    get "/queue-updates", SseController, :queue_updates
  end

  scope "/health", QueueMapLive do
    pipe_through :api
    get "/", Controllers.HealthController, :index
  end
end
