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

defmodule EccLive.Live.DashboardLive do
  @moduledoc """
  National summary: station reporting state, active alerts, aggregate turnout.

  Data is loaded from the ecc-api situational awareness snapshot
  (GET /ecc/situational-awareness) and refreshed every 30 seconds to match
  the upstream aggregation cadence. Push updates arrive as
  {:dashboard_update, payload} messages carrying the same JSON shape.
  """
  use EccLive, :live_view

  require Logger

  @refresh_ms 30_000

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket), do: schedule_refresh()

    {:ok, assign_summary(socket, fetch_situational_awareness())}
  end

  @impl true
  def handle_info({:dashboard_update, payload}, socket) do
    Logger.info("dashboard update", event: "dashboard_update")
    {:noreply, assign_summary(socket, normalise(payload))}
  end

  @impl true
  def handle_info(:auto_refresh, socket) do
    schedule_refresh()
    {:noreply, assign_summary(socket, fetch_situational_awareness())}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-dashboard">
      <h1>ECC National Dashboard</h1>
      <div class="stats">
        <div class="stat">
          <span class="label">Stations Reporting</span>
          <span class="value"><%= @stations_reporting %> / <%= @stations_total %></span>
        </div>
        <div class="stat">
          <span class="label">Active Alerts</span>
          <span class="value"><%= @alerts_active %></span>
        </div>
        <div class="stat">
          <span class="label">Turnout</span>
          <span class="value"><%= :erlang.float_to_binary(@turnout_pct, decimals: 1) %>%</span>
        </div>
      </div>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp assign_summary(socket, summary) do
    assign(socket,
      stations_total: summary.total_stations,
      stations_reporting: summary.reporting_stations,
      alerts_active: summary.active_alerts,
      turnout_pct: summary.turnout_pct
    )
  end

  # Accepts the string-keyed JSON body (HTTP response or push payload) and
  # fills safe defaults so a partial push cannot crash the view.
  defp normalise(body) when is_map(body) do
    %{
      total_stations: int(body["total_stations"]),
      reporting_stations: int(body["reporting_stations"]),
      active_alerts: int(body["active_alerts"]),
      turnout_pct: float(body["turnout_pct"])
    }
  end

  defp normalise(_body), do: normalise(%{})

  defp int(v) when is_integer(v), do: v
  defp int(v) when is_float(v), do: trunc(v)
  defp int(_), do: 0

  defp float(v) when is_integer(v), do: v * 1.0
  defp float(v) when is_float(v), do: v
  defp float(_), do: 0.0

  defp fetch_situational_awareness do
    case Req.get("#{api_base()}/ecc/situational-awareness") do
      {:ok, %{status: 200, body: body}} ->
        normalise(body)

      {:ok, %{status: status}} ->
        Logger.warning("situational awareness fetch failed", status: status)
        normalise(%{})

      {:error, reason} ->
        Logger.warning("situational awareness unreachable", reason: inspect(reason))
        normalise(%{})
    end
  rescue
    e ->
      Logger.warning("situational awareness fetch raised", error: Exception.message(e))
      normalise(%{})
  end

  defp api_base, do: Application.get_env(:ecc_live, :ecc_api_url, "http://ecc-api:8080")

  defp schedule_refresh do
    Process.send_after(self(), :auto_refresh, @refresh_ms)
  end
end
