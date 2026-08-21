defmodule BvvWeb.Router do
  use BvvWeb, :router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :fetch_live_flash
    plug :put_root_layout, html: {BvvWeb.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/", BvvWeb do
    pipe_through :browser

    live "/", LookupLive, :index
    live "/lookup", LookupLive, :lookup
    live "/objections", ObjectionsLive, :index
    live "/objections/new", ObjectionsLive, :new
    live "/objections/:id", ObjectionsLive, :show
  end

  scope "/health", BvvWeb do
    pipe_through :api
    get "/", HealthController, :index
  end
end
