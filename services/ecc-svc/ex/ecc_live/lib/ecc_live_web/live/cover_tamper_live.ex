defmodule EccLive.Live.CoverTamperLive do
  @moduledoc """
  Real-time cover tamper alert panel.

  Subscribes to the `eel:inventory:cover_tamper` PubSub topic and renders
  pending alerts sorted by severity then detection time.  Auto-refreshes
  every 15 seconds so stale push is never the only safety net.

  Severity encoding uses colour + shape + text (triple encoding, WCAG AA/AAA).
  """
  use EccLive, :live_view

  require Logger

  @refresh_ms 15_000

  # Severity ordering: critical=0, high=1, medium=2, low=3
  @severity_rank %{"critical" => 0, "high" => 1, "medium" => 2, "low" => 3}

  # ---------------------------------------------------------------------------
  # Mount / subscribe
  # ---------------------------------------------------------------------------

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket) do
      Phoenix.PubSub.subscribe(EccLive.PubSub, "eel:inventory:cover_tamper")
      schedule_refresh()
    end

    alerts = fetch_alerts(%{severity: nil, dispatched: nil, window_hours: nil})

    {:ok,
     assign(socket,
       alerts: alerts,
       filter_severity: nil,
       filter_dispatched: nil,
       filter_window_hours: nil,
       last_refreshed: DateTime.utc_now()
     )}
  end

  # ---------------------------------------------------------------------------
  # Params (filter controls)
  # ---------------------------------------------------------------------------

  @impl true
  def handle_params(params, _url, socket) do
    filters = %{
      severity: Map.get(params, "severity"),
      dispatched: Map.get(params, "dispatched"),
      window_hours: params |> Map.get("window_hours") |> parse_int()
    }

    alerts = fetch_alerts(filters)

    {:noreply,
     assign(socket,
       alerts: alerts,
       filter_severity: filters.severity,
       filter_dispatched: filters.dispatched,
       filter_window_hours: filters.window_hours
     )}
  end

  # ---------------------------------------------------------------------------
  # PubSub push
  # ---------------------------------------------------------------------------

  @impl true
  def handle_info({:cover_tamper_alert, alert}, socket) do
    Logger.info("cover tamper push received", alert_id: alert["alert_id"])
    filters = current_filters(socket)
    alerts = fetch_alerts(filters)
    {:noreply, assign(socket, alerts: alerts, last_refreshed: DateTime.utc_now())}
  end

  @impl true
  def handle_info(:auto_refresh, socket) do
    schedule_refresh()
    filters = current_filters(socket)
    alerts = fetch_alerts(filters)
    {:noreply, assign(socket, alerts: alerts, last_refreshed: DateTime.utc_now())}
  end

  # ---------------------------------------------------------------------------
  # Events
  # ---------------------------------------------------------------------------

  @impl true
  def handle_event("resolve", %{"alert_id" => alert_id} = params, socket) do
    second_officer = Map.get(params, "second_officer_id", "") |> String.trim()

    if second_officer == "" do
      {:noreply, put_flash(socket, :error, "Dual control: a second officer must countersign the resolution.")}
    else
      Logger.info("resolving cover tamper alert",
        alert_id: alert_id,
        countersigned_by: second_officer
      )

      # POST to inventory-svc API: PATCH /v1/rfid/tamper-alerts/:id/resolve
      case resolve_alert(alert_id, second_officer) do
      :ok ->
        filters = current_filters(socket)
        alerts = fetch_alerts(filters)
        {:noreply,
         socket
         |> put_flash(:info, "Alert #{alert_id} resolved.")
         |> assign(alerts: alerts)}

      {:error, reason} ->
        Logger.error("failed to resolve alert", alert_id: alert_id, reason: reason)
        {:noreply, put_flash(socket, :error, "Could not resolve alert: #{reason}")}
      end
    end
  end

  @impl true
  def handle_event("filter", params, socket) do
    query =
      params
      |> Map.take(["severity", "dispatched", "window_hours"])
      |> Enum.reject(fn {_k, v} -> v == "" or is_nil(v) end)
      |> Map.new()

    {:noreply, push_patch(socket, to: ~p"/cover-tamper?#{query}")}
  end

  # ---------------------------------------------------------------------------
  # Render
  # ---------------------------------------------------------------------------

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-cover-tamper" role="main" aria-labelledby="cover-tamper-heading">
      <h1 id="cover-tamper-heading">Cover tamper alerts</h1>
      <p class="last-refresh" aria-live="polite">
        Last updated: <time datetime={DateTime.to_iso8601(@last_refreshed)}><%= DateTime.to_string(@last_refreshed) %></time>
      </p>

      <%!-- Filter controls --%>
      <form phx-change="filter" aria-label="Filter alerts">
        <fieldset style="display:flex;gap:12px;flex-wrap:wrap;border:0;padding:0">
          <legend class="sr-only">Alert filters</legend>

          <label for="filter-severity">Severity</label>
          <select id="filter-severity" name="severity">
            <option value="">All</option>
            <option value="critical" selected={@filter_severity == "critical"}>Critical</option>
            <option value="high" selected={@filter_severity == "high"}>High</option>
            <option value="medium" selected={@filter_severity == "medium"}>Medium</option>
            <option value="low" selected={@filter_severity == "low"}>Low</option>
          </select>

          <label for="filter-dispatched">ECC dispatched</label>
          <select id="filter-dispatched" name="dispatched">
            <option value="">All</option>
            <option value="true" selected={@filter_dispatched == "true"}>Yes</option>
            <option value="false" selected={@filter_dispatched == "false"}>No</option>
          </select>

          <label for="filter-window">Time window</label>
          <select id="filter-window" name="window_hours">
            <option value="">All time</option>
            <option value="1" selected={@filter_window_hours == 1}>Last 1 h</option>
            <option value="6" selected={@filter_window_hours == 6}>Last 6 h</option>
            <option value="24" selected={@filter_window_hours == 24}>Last 24 h</option>
          </select>
        </fieldset>
      </form>

      <p role="status" aria-live="polite">
        <%= length(@alerts) %> pending alert<%= if length(@alerts) != 1, do: "s" %>.
      </p>

      <%!-- Alert cards for critical severity --%>
      <%= for alert <- @alerts, alert["severity"] == "critical" do %>
        <.alert_card alert={alert} />
      <% end %>

      <%!-- Remaining alerts in table --%>
      <%= if Enum.any?(@alerts, &(&1["severity"] != "critical")) do %>
        <table role="grid" aria-label="Non-critical cover tamper alerts" style="width:100%;border-collapse:collapse;margin-top:16px">
          <thead>
            <tr>
              <th scope="col">Severity</th>
              <th scope="col">Tag EPC</th>
              <th scope="col">Box</th>
              <th scope="col">Expected</th>
              <th scope="col">Observed</th>
              <th scope="col">Detected</th>
              <th scope="col">ECC dispatched</th>
              <th scope="col">Action</th>
            </tr>
          </thead>
          <tbody>
            <%= for alert <- @alerts, alert["severity"] != "critical" do %>
              <tr>
                <td><.severity_badge severity={alert["severity"]} /></td>
                <td><code><%= alert["epc"] %></code></td>
                <td><%= alert["paired_with_box"] %></td>
                <td><%= alert["expected_state"] %></td>
                <td><%= alert["observed_state"] %></td>
                <td><time datetime={alert["detected_at"]}><%= alert["detected_at"] %></time></td>
                <td><%= if alert["ecc_dispatched"], do: "Yes", else: "No" %></td>
                <td>
                  <button phx-click="resolve" phx-value-alert_id={alert["alert_id"]}
                    style="padding:4px 10px;cursor:pointer" aria-label={"Resolve alert #{alert["alert_id"]}"}>
                    Resolve
                  </button>
                </td>
              </tr>
            <% end %>
          </tbody>
        </table>
      <% end %>

      <%= if Enum.empty?(@alerts) do %>
        <p role="status">No pending cover tamper alerts.</p>
      <% end %>

      <p>
        <.link navigate={~p"/cover-tamper/history"}>View full state-transition history</.link>
      </p>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Components
  # ---------------------------------------------------------------------------

  defp alert_card(assigns) do
    ~H"""
    <article
      class={"alert-card severity-#{@alert["severity"]}"}
      aria-labelledby={"alert-#{@alert["alert_id"]}-heading"}
      style="border:2px solid #c0392b;border-radius:6px;padding:16px;margin-bottom:12px"
    >
      <h2 id={"alert-#{@alert["alert_id"]}-heading"} style="font-size:1.1em;margin:0 0 8px">
        <.severity_badge severity={@alert["severity"]} />
        Tag <code><%= @alert["epc"] %></code>
      </h2>
      <dl style="display:grid;grid-template-columns:max-content 1fr;gap:2px 12px;font-size:0.9em">
        <dt>Box</dt><dd><%= @alert["paired_with_box"] || "(unbound)" %></dd>
        <dt>Officer</dt><dd><%= @alert["dispatch_officer_id"] || "Not dispatched" %></dd>
        <dt>Expected state</dt><dd><%= @alert["expected_state"] %></dd>
        <dt>Observed state</dt><dd><%= @alert["observed_state"] %></dd>
        <dt>Detected</dt><dd><time datetime={@alert["detected_at"]}><%= @alert["detected_at"] %></time></dd>
        <%= if @alert["geo_lat"] do %>
          <dt>Geo</dt><dd><%= @alert["geo_lat"] %>, <%= @alert["geo_lon"] %></dd>
        <% end %>
        <dt>Description</dt><dd><%= @alert["description"] %></dd>
      </dl>
      <button
        phx-click="resolve"
        phx-value-alert_id={@alert["alert_id"]}
        style="margin-top:12px;padding:8px 16px;background:#27ae60;color:#fff;border:0;border-radius:4px;cursor:pointer;min-height:48px"
        aria-label={"Resolve alert for tag #{@alert["epc"]}"}
      >
        Resolve alert
      </button>
    </article>
    """
  end

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
    <span
      aria-label={"Severity: #{@label}"}
      title={@label}
      style={"font-weight:bold;color:#{@colour}"}
    >
      <%= raw(@shape) %> <%= @label %>
    </span>
    """
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp fetch_alerts(filters) do
    # In production this calls inventory-svc GET /v1/rfid/tamper-alerts via Req.
    # The response is filtered server-side; here we return an empty list as the
    # safe default so the view renders correctly without a live API connection.
    params =
      [
        severity: filters.severity,
        dispatched: filters.dispatched,
        window_hours: filters.window_hours
      ]
      |> Enum.reject(fn {_k, v} -> is_nil(v) end)
      |> Map.new()

    case Req.get("#{base_url()}/v1/rfid/tamper-alerts", params: params) do
      {:ok, %{status: 200, body: %{"alerts" => alerts}}} ->
        alerts
        |> Enum.sort_by(&{Map.get(@severity_rank, &1["severity"], 99), &1["detected_at"]})

      _ ->
        []
    end
  rescue
    _ -> []
  end

  defp resolve_alert(alert_id, second_officer) do
    case Req.patch("#{base_url()}/v1/rfid/tamper-alerts/#{alert_id}/resolve",
           json: %{second_officer: second_officer}
         ) do
      {:ok, %{status: s}} when s in [200, 204] -> :ok
      {:ok, %{status: s}} -> {:error, "HTTP #{s}"}
      {:error, reason} -> {:error, inspect(reason)}
    end
  rescue
    e -> {:error, inspect(e)}
  end

  defp base_url, do: Application.get_env(:ecc_live, :inventory_api_url, "http://inventory-svc:8080")

  defp current_filters(socket) do
    %{
      severity: socket.assigns[:filter_severity],
      dispatched: socket.assigns[:filter_dispatched],
      window_hours: socket.assigns[:filter_window_hours]
    }
  end

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
