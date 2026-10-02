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

defmodule SpeechAnalystLive.Live.DashboardLive do
  @moduledoc "Overview of content flags and analyst decisions."
  use SpeechAnalystLive, :live_view

  require Logger

  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:flags_total, 0)
     |> assign(:flags_pending, 0)
     |> assign(:flags_confirmed, 0)
     |> assign(:flags_rejected, 0)
     |> assign(:by_type, %{})}
  end

  @impl true
  def handle_info({:flags_update, payload}, socket) do
    Logger.info("dashboard flags update", event: "flags_update")
    {:noreply, assign(socket, payload)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="speech-dashboard">
      <h1>Speech Integrity Dashboard</h1>
      <div class="stats">
        <div class="stat">
          <span class="label">Total Flags</span>
          <span class="value"><%= @flags_total %></span>
        </div>
        <div class="stat">
          <span class="label">Pending</span>
          <span class="value"><%= @flags_pending %></span>
        </div>
        <div class="stat">
          <span class="label">Confirmed</span>
          <span class="value"><%= @flags_confirmed %></span>
        </div>
        <div class="stat">
          <span class="label">Rejected</span>
          <span class="value"><%= @flags_rejected %></span>
        </div>
      </div>
    </div>
    """
  end
end
