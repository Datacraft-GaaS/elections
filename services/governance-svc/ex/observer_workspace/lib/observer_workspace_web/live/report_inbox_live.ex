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

defmodule ObserverWorkspaceWeb.ReportInboxLive do
  use ObserverWorkspaceWeb, :live_view

  alias ObserverWorkspace.Missions

  @impl true
  def mount(%{"id" => mission_id}, _session, socket) do
    reports = Missions.list_reports(mission_id)
    {:ok, assign(socket, mission_id: mission_id, reports: reports)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="mx-auto max-w-2xl p-6">
      <h1 class="text-xl font-bold mb-4">Observation Reports - <%= @mission_id %></h1>
      <%= if @reports == [] do %>
        <p class="text-gray-400">No reports submitted yet.</p>
      <% else %>
        <ul class="space-y-2">
          <%= for r <- @reports do %>
            <li class="border rounded p-3"><%= r.report_type %> - <%= r.submitted_at %></li>
          <% end %>
        </ul>
      <% end %>
    </div>
    """
  end
end
