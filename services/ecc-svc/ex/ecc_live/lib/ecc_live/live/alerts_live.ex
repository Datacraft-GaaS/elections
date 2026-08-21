defmodule EccLive.Live.AlertsLive do
  @moduledoc "Alert feed and dispatch console."
  use EccLive, :live_view

  require Logger

  @impl true
  def mount(_params, _session, socket) do
    # TODO: call ecc-api GET /api/v1/ecc/alerts via Req
    {:ok, assign(socket, :alerts, [])}
  end

  @impl true
  def handle_event("dispatch", %{"alert_id" => alert_id}, socket) do
    Logger.info("dispatching alert", alert_id: alert_id)
    # TODO: POST /api/v1/ecc/alerts/:id/dispatch via Req
    {:noreply, put_flash(socket, :info, "Alert #{alert_id} dispatched")}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-alerts">
      <h1>Alert Feed</h1>
      <ul>
        <%= for alert <- @alerts do %>
          <li>
            <%= alert.summary %>
            <button phx-click="dispatch" phx-value-alert_id={alert.id}>Dispatch</button>
          </li>
        <% end %>
      </ul>
      <%= if Enum.empty?(@alerts) do %>
        <p>No active alerts.</p>
      <% end %>
    </div>
    """
  end
end
