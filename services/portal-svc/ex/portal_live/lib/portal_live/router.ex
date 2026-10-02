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

defmodule PortalLive.Router do
  use Phoenix.Router
  import Phoenix.LiveView.Router

  pipeline :browser do
    plug :accepts, ["html"]
    plug :fetch_session
    plug :put_root_layout, html: {PortalLive.Layouts, :root}
    plug :protect_from_forgery
    plug :put_secure_browser_headers
  end

  pipeline :api do
    plug :accepts, ["json", "text/event-stream"]
  end

  scope "/", PortalLive do
    pipe_through :browser

    live "/live/counts", Live.CountsLive, :index
  end

  scope "/api/v1/portal", PortalLive do
    pipe_through :api

    # SSE endpoint for third-party feeds (FR-M11, ADR-0011 Tier 5).
    # Unauthenticated public stream; strictly aggregate counts, no PII.
    get "/sse/counts", SseController, :counts
  end
end
