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
    live "/ecc/workforce", Live.WorkforceLive, :index
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
