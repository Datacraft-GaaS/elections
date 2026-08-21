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
