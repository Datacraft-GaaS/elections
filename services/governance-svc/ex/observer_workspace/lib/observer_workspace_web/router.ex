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
