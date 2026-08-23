defmodule EccLive.Live.AlertsLive do
  @moduledoc """
  Alert feed and dispatch console.

  Alerts map to ecc-api escalations: the feed loads open escalations
  (GET /ecc/escalations?status=open) and dispatching an alert acknowledges
  the underlying escalation (PATCH /ecc/escalations/{escalation_id}).
  """
  use EccLive, :live_view

  require Logger

  @impl true
  def mount(_params, _session, socket) do
    {:ok, assign(socket, :alerts, fetch_alerts())}
  end

  @impl true
  def handle_event("dispatch", %{"alert_id" => alert_id}, socket) do
    Logger.info("dispatching alert", alert_id: alert_id)

    case acknowledge_alert(alert_id) do
      :ok ->
        alerts =
          Enum.map(socket.assigns.alerts, fn alert ->
            if alert.id == alert_id, do: %{alert | status: "acknowledged"}, else: alert
          end)

        {:noreply,
         socket
         |> assign(:alerts, alerts)
         |> put_flash(:info, "Alert #{alert_id} dispatched")}

      {:error, reason} ->
        Logger.warning("alert dispatch failed", alert_id: alert_id, reason: reason)
        {:noreply, put_flash(socket, :error, "Failed to dispatch alert #{alert_id}: #{reason}")}
    end
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-alerts">
      <h1>Alert Feed</h1>
      <%= if msg = @flash["info"] do %>
        <p class="flash flash-info"><%= msg %></p>
      <% end %>
      <%= if msg = @flash["error"] do %>
        <p class="flash flash-error"><%= msg %></p>
      <% end %>
      <ul>
        <%= for alert <- @alerts do %>
          <li class={"alert alert-#{alert.severity}"}>
            <span class="severity"><%= alert.severity %></span>
            <span class="station"><%= alert.station_code %></span>
            <%= alert.summary %>
            <%= if alert.status == "open" do %>
              <button phx-click="dispatch" phx-value-alert_id={alert.id}>Dispatch</button>
            <% else %>
              <span class="status"><%= alert.status %></span>
            <% end %>
          </li>
        <% end %>
      </ul>
      <%= if Enum.empty?(@alerts) do %>
        <p>No active alerts.</p>
      <% end %>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp fetch_alerts do
    case Req.get("#{api_base()}/ecc/escalations", params: [status: "open"]) do
      {:ok, %{status: 200, body: %{"escalations" => escalations}}} ->
        Enum.map(escalations, &to_alert/1)

      other ->
        Logger.warning("escalation feed fetch failed", response: inspect(other))
        []
    end
  rescue
    e ->
      Logger.warning("escalation feed fetch raised", error: Exception.message(e))
      []
  end

  defp to_alert(escalation) do
    %{
      id: escalation["escalation_id"],
      station_code: escalation["station_code"],
      category: escalation["category"],
      severity: escalation["severity"],
      status: escalation["status"],
      summary: escalation["description"]
    }
  end

  defp acknowledge_alert(alert_id) do
    case Req.patch("#{api_base()}/ecc/escalations/#{alert_id}", json: %{status: "acknowledged"}) do
      {:ok, %{status: 200}} ->
        :ok

      {:ok, %{status: status}} ->
        {:error, "HTTP #{status}"}

      {:error, reason} ->
        {:error, inspect(reason)}
    end
  rescue
    e -> {:error, Exception.message(e)}
  end

  defp api_base, do: Application.get_env(:ecc_live, :ecc_api_url, "http://ecc-api:8080")
end
