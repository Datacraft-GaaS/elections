defmodule ObserverWorkspaceWeb.BriefingCalendarLive do
  use ObserverWorkspaceWeb, :live_view

  alias ObserverWorkspace.Missions

  @impl true
  def mount(%{"id" => mission_id}, _session, socket) do
    briefings = Missions.list_briefings(mission_id)
    {:ok, assign(socket, mission_id: mission_id, briefings: briefings)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="mx-auto max-w-2xl p-6">
      <h1 class="text-xl font-bold mb-4">Briefing Calendar - <%= @mission_id %></h1>
      <%= if @briefings == [] do %>
        <p class="text-gray-400">No briefings scheduled for this mission.</p>
      <% else %>
        <ul class="space-y-2">
          <%= for b <- @briefings do %>
            <li class="border rounded p-3"><%= b.topic %> at <%= b.scheduled_at %></li>
          <% end %>
        </ul>
      <% end %>
    </div>
    """
  end
end
