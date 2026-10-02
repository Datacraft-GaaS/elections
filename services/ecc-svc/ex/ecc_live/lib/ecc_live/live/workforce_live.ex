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

defmodule EccLive.Live.WorkforceLive do
  @moduledoc """
  Workforce readiness view.

  Rows are per-station rollups from the workforce-api readiness endpoint
  (GET /v1/workforce/readiness): active postings, open kit claims and task
  state counts. The table refreshes every 60 seconds. When the workforce
  API is unreachable the view renders a degraded banner with zeroed totals.
  """
  use EccLive, :live_view

  require Logger

  @refresh_ms 60_000

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket), do: schedule_refresh()

    {:ok, assign_readiness(socket, fetch_readiness())}
  end

  @impl true
  def handle_info(:auto_refresh, socket) do
    schedule_refresh()
    {:noreply, assign_readiness(socket, fetch_readiness())}
  end

  defp assign_readiness(socket, readiness) do
    socket
    |> assign(:stations, Map.get(readiness, "stations", []))
    |> assign(:totals, Map.get(readiness, "totals", %{}))
    |> assign(:degraded, Map.get(readiness, "degraded", false))
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-workforce">
      <h1>Workforce Readiness</h1>
      <%= if @degraded do %>
        <p class="workforce-degraded">Workforce API unreachable</p>
      <% end %>
      <div class="totals">
        <span class="total-cell">Active postings: {@totals["active_postings"] || 0}</span>
        <span class="total-cell">Open kit claims: {@totals["open_kit_claims"] || 0}</span>
        <span class="total-cell">Assignments: {@totals["assignments"] || 0}</span>
      </div>
      <table class="readiness-table">
        <thead>
          <tr>
            <th>Station</th>
            <th>Postings</th>
            <th>Kit claims</th>
            <th>Done/Total</th>
            <th>Blocked</th>
            <th>Overdue</th>
          </tr>
        </thead>
        <tbody>
          <%= for station <- sorted_stations(@stations) do %>
            <tr class="station-row">
              <td>{station["station_code"]}</td>
              <td>{station["active_postings"]}</td>
              <td>{station["open_kit_claims"]}</td>
              <td>{station["tasks_done"]}/{station["tasks_total"]}</td>
              <td class={if station["tasks_blocked"] > 0, do: "cell-blocked", else: ""}>
                {station["tasks_blocked"]}
              </td>
              <td class={if station["tasks_overdue"] > 0, do: "cell-overdue", else: ""}>
                {station["tasks_overdue"]}
              </td>
            </tr>
          <% end %>
          <%= if Enum.empty?(@stations) do %>
            <tr><td colspan="6">No workforce data available.</td></tr>
          <% end %>
        </tbody>
      </table>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Public data helpers
  # ---------------------------------------------------------------------------

  @doc """
  Fetches station readiness rollups from the workforce API.

  On failure returns an empty, degraded payload so the UI can render a
  banner instead of crashing.
  """
  def fetch_readiness do
    case Req.get("#{workforce_base()}/v1/workforce/readiness") do
      {:ok, %{status: 200, body: %{"stations" => stations, "totals" => totals}}} ->
        %{"stations" => stations, "totals" => totals, "degraded" => false}

      other ->
        Logger.warning("workforce readiness fetch failed", response: inspect(other))
        empty_readiness()
    end
  rescue
    e ->
      Logger.warning("workforce readiness fetch raised", error: Exception.message(e))
      empty_readiness()
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp empty_readiness do
    %{
      "stations" => [],
      "totals" => %{"active_postings" => 0, "open_kit_claims" => 0, "assignments" => 0},
      "degraded" => true
    }
  end

  defp sorted_stations(stations) do
    Enum.sort_by(stations, fn s -> {-Map.get(s, "tasks_overdue", 0), s["station_code"]} end)
  end

  defp workforce_base,
    do: Application.get_env(:ecc_live, :workforce_api_url, "http://workforce-api:8100")

  defp schedule_refresh do
    Process.send_after(self(), :auto_refresh, @refresh_ms)
  end
end
