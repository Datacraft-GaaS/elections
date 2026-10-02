# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
