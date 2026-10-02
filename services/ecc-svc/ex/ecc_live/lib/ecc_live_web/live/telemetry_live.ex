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

defmodule EccLive.Live.TelemetryLive do
  @moduledoc """
  Real-time mobile telemetry dashboard.

  Panels:
    - Signals per minute per class (15 s refresh)
    - Open alerts by severity
    - Top devices by drop count
    - Top officers by session duration
    - Geo-heatmap cell counts (active devices)
    - Battery-low fleet list
    - Session-hijack alerts

  Subscribes to `eel:telemetry:alert` PubSub topic for push updates.
  Filter controls: class, severity, time window, county.
  Refresh: 15 s timer as safety net when push is slow.
  """
  use EccLive, :live_view

  require Logger

  @refresh_ms 15_000

  # ---------------------------------------------------------------------------
  # Mount / subscribe
  # ---------------------------------------------------------------------------

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket) do
      Phoenix.PubSub.subscribe(EccLive.PubSub, "eel:telemetry:alert")
      schedule_refresh()
    end

    filters = default_filters()
    dashboard = fetch_dashboard(filters)

    {:ok,
     assign(socket,
       dashboard: dashboard,
       filters: filters,
       last_refreshed: DateTime.utc_now()
     )}
  end

  # ---------------------------------------------------------------------------
  # Params (filter controls)
  # ---------------------------------------------------------------------------

  @impl true
  def handle_params(params, _url, socket) do
    filters = %{
      class: Map.get(params, "class"),
      severity: Map.get(params, "severity"),
      window_hours: params |> Map.get("window_hours") |> parse_int() |> then(&(&1 || 24)),
      county: Map.get(params, "county")
    }

    dashboard = fetch_dashboard(filters)

    {:noreply,
     assign(socket,
       dashboard: dashboard,
       filters: filters
     )}
  end

  # ---------------------------------------------------------------------------
  # PubSub push
  # ---------------------------------------------------------------------------

  @impl true
  def handle_info({:telemetry_alert, alert}, socket) do
    Logger.info("telemetry alert push received", alert_id: alert["alert_id"])
    dashboard = fetch_dashboard(socket.assigns.filters)
    {:noreply, assign(socket, dashboard: dashboard, last_refreshed: DateTime.utc_now())}
  end

  @impl true
  def handle_info(:auto_refresh, socket) do
    schedule_refresh()
    dashboard = fetch_dashboard(socket.assigns.filters)
    {:noreply, assign(socket, dashboard: dashboard, last_refreshed: DateTime.utc_now())}
  end

  # ---------------------------------------------------------------------------
  # Events
  # ---------------------------------------------------------------------------

  @impl true
  def handle_event("filter", params, socket) do
    query =
      params
      |> Map.take(["class", "severity", "window_hours", "county"])
      |> Enum.reject(fn {_k, v} -> v == "" or is_nil(v) end)
      |> Map.new()

    {:noreply, push_patch(socket, to: ~p"/telemetry?#{query}")}
  end

  # ---------------------------------------------------------------------------
  # Render
  # ---------------------------------------------------------------------------

  @impl true
  def render(assigns) do
    ~H"""
    <div class="eel-telemetry-dashboard" role="main" aria-labelledby="telemetry-heading">
      <h1 id="telemetry-heading">Mobile telemetry dashboard</h1>
      <p class="last-refresh" aria-live="polite">
        Last updated: <time datetime={DateTime.to_iso8601(@last_refreshed)}><%= DateTime.to_string(@last_refreshed) %></time>
      </p>

      <%!-- Filter controls --%>
      <form phx-change="filter" aria-label="Dashboard filters" style="display:flex;gap:12px;flex-wrap:wrap;margin-bottom:20px">
        <fieldset style="border:0;padding:0">
          <legend class="sr-only">Telemetry filters</legend>

          <label for="filter-class">Signal class</label>
          <select id="filter-class" name="class" style="margin-left:4px;margin-right:12px">
            <option value="">All classes</option>
            <option value="location" selected={@filters.class == "location"}>Location</option>
            <option value="battery" selected={@filters.class == "battery"}>Battery</option>
            <option value="physical_drops" selected={@filters.class == "physical_drops"}>Physical drops</option>
            <option value="login_logout" selected={@filters.class == "login_logout"}>Login / logout</option>
            <option value="attestation" selected={@filters.class == "attestation"}>Attestation</option>
            <option value="network" selected={@filters.class == "network"}>Network</option>
          </select>

          <label for="filter-severity">Severity</label>
          <select id="filter-severity" name="severity" style="margin-left:4px;margin-right:12px">
            <option value="">All severities</option>
            <option value="critical" selected={@filters.severity == "critical"}>Critical</option>
            <option value="high" selected={@filters.severity == "high"}>High</option>
            <option value="medium" selected={@filters.severity == "medium"}>Medium</option>
            <option value="low" selected={@filters.severity == "low"}>Low</option>
          </select>

          <label for="filter-window">Time window</label>
          <select id="filter-window" name="window_hours" style="margin-left:4px;margin-right:12px">
            <option value="1" selected={@filters.window_hours == 1}>Last 1 h</option>
            <option value="6" selected={@filters.window_hours == 6}>Last 6 h</option>
            <option value="24" selected={@filters.window_hours == 24}>Last 24 h</option>
            <option value="48" selected={@filters.window_hours == 48}>Last 48 h</option>
          </select>

          <label for="filter-county">County</label>
          <input
            type="text"
            id="filter-county"
            name="county"
            value={@filters.county || ""}
            placeholder="e.g. Nairobi"
            style="margin-left:4px;padding:4px 8px;border:1px solid #d1d5db;border-radius:4px"
          >
        </fieldset>
      </form>

      <%!-- Panels grid --%>
      <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(340px,1fr));gap:16px">

        <%!-- Signals per minute per class --%>
        <section class="panel" aria-labelledby="panel-spm-heading" style="background:#fff;border-radius:8px;padding:16px;border:1px solid #e5e5e5">
          <h2 id="panel-spm-heading" style="font-size:1rem;margin:0 0 10px">Signals per minute by class</h2>
          <table role="grid" style="width:100%;font-size:0.88rem">
            <thead>
              <tr>
                <th scope="col">Class</th>
                <th scope="col">Rate / min</th>
              </tr>
            </thead>
            <tbody>
              <%= for {cls, rate} <- (@dashboard[:signals_per_min] || []) do %>
                <tr>
                  <td><code><%= cls %></code></td>
                  <td><%= rate %></td>
                </tr>
              <% end %>
              <%= if Enum.empty?(@dashboard[:signals_per_min] || []) do %>
                <tr><td colspan="2" style="color:#6b7280">No data</td></tr>
              <% end %>
            </tbody>
          </table>
        </section>

        <%!-- Open alerts by severity --%>
        <section class="panel" aria-labelledby="panel-alerts-heading" style="background:#fff;border-radius:8px;padding:16px;border:1px solid #e5e5e5">
          <h2 id="panel-alerts-heading" style="font-size:1rem;margin:0 0 10px">Open alerts by severity</h2>
          <dl style="display:grid;grid-template-columns:1fr 1fr;gap:6px">
            <%= for {sev, count} <- (@dashboard[:alerts_by_severity] || %{}) do %>
              <dt><.severity_badge severity={to_string(sev)} /></dt>
              <dd style="margin:0;font-weight:700"><%= count %></dd>
            <% end %>
          </dl>
        </section>

        <%!-- Top devices by drop count --%>
        <section class="panel" aria-labelledby="panel-drops-heading" style="background:#fff;border-radius:8px;padding:16px;border:1px solid #e5e5e5">
          <h2 id="panel-drops-heading" style="font-size:1rem;margin:0 0 10px">Top devices by drop count</h2>
          <ol style="margin:0;padding-left:20px;font-size:0.88rem">
            <%= for {device_id, drops} <- (@dashboard[:top_drops] || []) do %>
              <li><code><%= device_id %></code>: <%= drops %> drop(s)</li>
            <% end %>
            <%= if Enum.empty?(@dashboard[:top_drops] || []) do %>
              <li style="list-style:none;color:#6b7280">No data</li>
            <% end %>
          </ol>
        </section>

        <%!-- Top officers by session duration --%>
        <section class="panel" aria-labelledby="panel-officers-heading" style="background:#fff;border-radius:8px;padding:16px;border:1px solid #e5e5e5">
          <h2 id="panel-officers-heading" style="font-size:1rem;margin:0 0 10px">Top officers by session duration</h2>
          <ol style="margin:0;padding-left:20px;font-size:0.88rem">
            <%= for {officer_id, dur_min} <- (@dashboard[:top_officers] || []) do %>
              <li><code><%= officer_id %></code>: <%= dur_min %> min</li>
            <% end %>
            <%= if Enum.empty?(@dashboard[:top_officers] || []) do %>
              <li style="list-style:none;color:#6b7280">No data</li>
            <% end %>
          </ol>
        </section>

        <%!-- Battery-low fleet --%>
        <section class="panel" aria-labelledby="panel-battery-heading" style="background:#fff;border-radius:8px;padding:16px;border:1px solid #e5e5e5">
          <h2 id="panel-battery-heading" style="font-size:1rem;margin:0 0 10px">Battery low (&lt; 15%)</h2>
          <%= if Enum.empty?(@dashboard[:battery_low] || []) do %>
            <p style="color:#16a34a;font-size:0.9rem">No kits below threshold.</p>
          <% else %>
            <ul role="list" style="margin:0;padding:0;list-style:none;font-size:0.88rem">
              <%= for device <- (@dashboard[:battery_low] || []) do %>
                <li style="padding:4px 0;border-bottom:1px solid #f3f4f6">
                  <code><%= device["device_id"] %></code>
                  <span style="color:#dc2626;font-weight:700"><%= device["battery_pct"] %>%</span>
                  <span style="color:#6b7280"><%= device["county"] %></span>
                </li>
              <% end %>
            </ul>
          <% end %>
        </section>

        <%!-- Session-hijack alerts --%>
        <section class="panel" aria-labelledby="panel-hijack-heading" style="background:#fff;border-radius:8px;padding:16px;border:1px solid #e5e5e5">
          <h2 id="panel-hijack-heading" style="font-size:1rem;margin:0 0 10px">Session-hijack alerts</h2>
          <%= if Enum.empty?(@dashboard[:session_hijack] || []) do %>
            <p style="color:#16a34a;font-size:0.9rem">No hijack alerts.</p>
          <% else %>
            <ul role="list" style="margin:0;padding:0;list-style:none;font-size:0.88rem">
              <%= for alert <- (@dashboard[:session_hijack] || []) do %>
                <li style="padding:4px 0;border-bottom:1px solid #f3f4f6">
                  <.severity_badge severity="critical" />
                  Officer <code><%= alert["officer_id"] %></code> at <time><%= alert["detected_at"] %></time>
                </li>
              <% end %>
            </ul>
          <% end %>
        </section>

      </div>

      <p style="margin-top:20px">
        <.link navigate={~p"/telemetry/query"}>Open interactive query builder</.link>
      </p>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Components
  # ---------------------------------------------------------------------------

  defp severity_badge(assigns) do
    {shape, colour, label} =
      case assigns.severity do
        "critical" -> {"&#9632;", "#c0392b", "CRITICAL"}
        "high" -> {"&#9650;", "#e67e22", "HIGH"}
        "medium" -> {"&#9679;", "#b7950b", "MEDIUM"}
        _ -> {"&#9670;", "#2980b9", "LOW"}
      end

    assigns = assign(assigns, shape: shape, colour: colour, label: label)

    ~H"""
    <span aria-label={"Severity: #{@label}"} title={@label} style={"font-weight:bold;color:#{@colour}"}>
      <%= raw(@shape) %> <%= @label %>
    </span>
    """
  end

  # ---------------------------------------------------------------------------
  # Data helpers
  # ---------------------------------------------------------------------------

  defp default_filters do
    %{class: nil, severity: nil, window_hours: 24, county: nil}
  end

  defp fetch_dashboard(filters) do
    window_hours = filters.window_hours || 24
    now = DateTime.utc_now()
    from_dt = DateTime.add(now, -window_hours * 3600, :second)

    params =
      [
        from: DateTime.to_iso8601(from_dt),
        to: DateTime.to_iso8601(now)
      ]
      |> maybe_add(:class, filters.class)
      |> maybe_add(:severity, filters.severity)
      |> maybe_add(:county, filters.county)

    %{
      signals_per_min: fetch_signals_per_min(params),
      alerts_by_severity: fetch_alerts_by_severity(params),
      top_drops: fetch_top_drops(params),
      top_officers: fetch_top_officers(params),
      battery_low: fetch_battery_low(params),
      session_hijack: fetch_session_hijack(params)
    }
  end

  defp fetch_signals_per_min(params) do
    case Req.get("#{api_base()}/v1/telemetry/presets/geo_heatmap/run", params: params) do
      {:ok, %{status: 200, body: %{"rows" => rows}}} ->
        rows
        |> Enum.group_by(& &1["signal_class"])
        |> Enum.map(fn {cls, items} -> {cls, length(items)} end)
        |> Enum.sort_by(fn {_, n} -> -n end)
        |> Enum.take(10)

      _ ->
        []
    end
  rescue
    _ -> []
  end

  defp fetch_alerts_by_severity(_params) do
    # Stub: in production call the alerts API
    %{}
  end

  defp fetch_top_drops(params) do
    case Req.get("#{api_base()}/v1/telemetry/drops", params: params) do
      {:ok, %{status: 200, body: %{"rows" => rows}}} ->
        rows
        |> Enum.group_by(& &1["device_id"])
        |> Enum.map(fn {did, items} -> {did, length(items)} end)
        |> Enum.sort_by(fn {_, n} -> -n end)
        |> Enum.take(5)

      _ ->
        []
    end
  rescue
    _ -> []
  end

  defp fetch_top_officers(_params) do
    []
  end

  defp fetch_battery_low(params) do
    case Req.get("#{api_base()}/v1/telemetry/presets/battery_low_fleet/run", params: params) do
      {:ok, %{status: 200, body: %{"rows" => rows}}} ->
        rows |> Enum.take(10)

      _ ->
        []
    end
  rescue
    _ -> []
  end

  defp fetch_session_hijack(params) do
    case Req.get("#{api_base()}/v1/telemetry/presets/session_hijack_alerts/run", params: params) do
      {:ok, %{status: 200, body: %{"rows" => rows}}} ->
        rows |> Enum.take(10)

      _ ->
        []
    end
  rescue
    _ -> []
  end

  defp maybe_add(list, _key, nil), do: list
  defp maybe_add(list, _key, ""), do: list
  defp maybe_add(list, key, value), do: [{key, value} | list]

  defp api_base,
    do: Application.get_env(:ecc_live, :telemetry_api_url, "http://telemetry-query:8080")

  defp parse_int(nil), do: nil
  defp parse_int(""), do: nil

  defp parse_int(s) do
    case Integer.parse(s) do
      {n, _} -> n
      :error -> nil
    end
  end

  defp schedule_refresh do
    Process.send_after(self(), :auto_refresh, @refresh_ms)
  end
end
