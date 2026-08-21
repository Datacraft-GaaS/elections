defmodule ObserverWorkspaceWeb.MissionWorkspaceLive do
  use ObserverWorkspaceWeb, :live_view

  alias ObserverWorkspace.Missions

  @impl true
  def mount(%{"id" => mission_id}, _session, socket) do
    # RBAC: in production, user mission_id and role come from session JWT claims.
    # Stub: allow access for demo; real impl calls Missions.authorize/3.
    missions = Missions.list_missions()
    mission = Enum.find(missions, fn m -> m.id == mission_id end)

    case mission do
      nil ->
        {:ok, push_navigate(socket, to: "/")}
      _ ->
        briefings = Missions.list_briefings(mission_id)
        reports = Missions.list_reports(mission_id)
        {:ok,
         socket
         |> assign(:mission, mission)
         |> assign(:briefings, briefings)
         |> assign(:reports, reports)}
    end
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="mx-auto max-w-4xl p-6">
      <h1 class="text-2xl font-bold mb-2"><%= @mission.organisation %></h1>
      <p class="text-sm text-gray-500 mb-6">Workspace: <%= @mission.id %></p>

      <div class="grid grid-cols-2 gap-6">
        <div>
          <h2 class="text-lg font-semibold mb-2">Upcoming Briefings</h2>
          <%= if @briefings == [] do %>
            <p class="text-gray-400 text-sm">No briefings scheduled.</p>
          <% else %>
            <ul>
              <%= for b <- @briefings do %>
                <li><%= b.topic %> - <%= b.scheduled_at %></li>
              <% end %>
            </ul>
          <% end %>
        </div>

        <div>
          <h2 class="text-lg font-semibold mb-2">Observation Reports</h2>
          <%= if @reports == [] do %>
            <p class="text-gray-400 text-sm">No reports submitted.</p>
          <% else %>
            <ul>
              <%= for r <- @reports do %>
                <li><%= r.report_type %> - <%= r.submitted_at %></li>
              <% end %>
            </ul>
          <% end %>
        </div>
      </div>

      <div class="mt-6 flex gap-4">
        <.link navigate={"/missions/#{@mission.id}/briefings"} class="btn">Briefing Calendar</.link>
        <.link navigate={"/missions/#{@mission.id}/reports"} class="btn">Submit Report</.link>
        <.link navigate={"/joint-statements"} class="btn">Joint Statements</.link>
      </div>
    </div>
    """
  end
end
