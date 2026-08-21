defmodule EccLive.Live.DashboardLive do
  @moduledoc "National summary: kits, alerts, results."
  use EccLive, :live_view

  require Logger

  @impl true
  def mount(_params, _session, socket) do
    # TODO: subscribe to ecc-api SSE /api/v1/ecc/dashboard via Req stream
    {:ok,
     socket
     |> assign(:kits_total, 0)
     |> assign(:kits_online, 0)
     |> assign(:alerts_active, 0)
     |> assign(:results_submitted, 0)}
  end

  @impl true
  def handle_info({:dashboard_update, payload}, socket) do
    Logger.info("dashboard update", event: "dashboard_update")
    {:noreply, assign(socket, payload)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-dashboard">
      <h1>ECC National Dashboard</h1>
      <div class="stats">
        <div class="stat">
          <span class="label">Kits Online</span>
          <span class="value"><%= @kits_online %> / <%= @kits_total %></span>
        </div>
        <div class="stat">
          <span class="label">Active Alerts</span>
          <span class="value"><%= @alerts_active %></span>
        </div>
        <div class="stat">
          <span class="label">Results Submitted</span>
          <span class="value"><%= @results_submitted %></span>
        </div>
      </div>
    </div>
    """
  end
end
