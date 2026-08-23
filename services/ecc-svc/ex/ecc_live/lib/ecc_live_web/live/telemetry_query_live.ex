defmodule EccLive.Live.TelemetryQueryLive do
  @moduledoc """
  Interactive telemetry query builder + NL bridge LiveView.

  Layout:
    Left   - preset library dropdown + custom DSL textarea (JSON)
    Middle - NL text-input + Confirm-proposed-DSL step
    Right  - result table with per-column filter + export buttons

  Every query is logged with the actor SVID via the telemetry query API audit
  endpoint. Export buttons trigger evidence bundle generation.
  """
  use EccLive, :live_view

  # ---------------------------------------------------------------------------
  # Mount
  # ---------------------------------------------------------------------------

  @impl true
  def mount(_params, _session, socket) do
    presets = fetch_presets()

    {:ok,
     assign(socket,
       presets: presets,
       preset_selected: nil,
       dsl_text: default_dsl(),
       nl_text: "",
       proposed_dsl: nil,
       nl_explanation: nil,
       nl_confidence: nil,
       query_result: nil,
       query_error: nil,
       query_id: nil,
       export_result: nil,
       export_error: nil,
       column_filter: %{},
       running: false,
       exporting: false
     )}
  end

  # ---------------------------------------------------------------------------
  # Events
  # ---------------------------------------------------------------------------

  @impl true
  def handle_event("select_preset", %{"name" => name}, socket) do
    # Fetch the preset's default DSL by running it with empty params
    case run_api_post("/v1/telemetry/presets/#{name}/run", %{"params" => %{}}, socket) do
      {:ok, _result} ->
        {:noreply, assign(socket, preset_selected: name, query_error: nil)}

      {:error, msg} ->
        {:noreply, assign(socket, query_error: msg)}
    end
  end

  @impl true
  def handle_event("update_dsl", %{"dsl" => dsl}, socket) do
    {:noreply, assign(socket, dsl_text: dsl)}
  end

  @impl true
  def handle_event("run_dsl", _params, socket) do
    case Jason.decode(socket.assigns.dsl_text) do
      {:ok, parsed} ->
        current_user = socket.assigns[:current_user]

        {:noreply,
         socket
         |> assign(running: true, query_result: nil, query_error: nil)
         |> start_async(:run_query, fn -> execute_query(parsed, current_user) end)}

      {:error, err} ->
        {:noreply, assign(socket, query_error: "Invalid JSON: #{inspect(err)}")}
    end
  end

  @impl true
  def handle_event("run_nl", %{"nl_text" => text}, socket) do
    trimmed = String.trim(text)

    if trimmed == "" do
      {:noreply, assign(socket, query_error: "Query text is required")}
    else
      current_user = socket.assigns[:current_user]

      {:noreply,
       socket
       |> assign(nl_text: trimmed, running: true, proposed_dsl: nil, query_error: nil)
       |> start_async(:nl_translate, fn -> translate_nl(trimmed, current_user) end)}
    end
  end

  @impl true
  def handle_event("confirm_nl_dsl", _params, socket) do
    case socket.assigns.proposed_dsl do
      nil ->
        {:noreply, assign(socket, query_error: "No proposed DSL to confirm")}

      proposed ->
        current_user = socket.assigns[:current_user]

        {:noreply,
         socket
         |> assign(running: true, query_result: nil, query_error: nil)
         |> start_async(:run_query, fn -> execute_query(proposed, current_user) end)}
    end
  end

  @impl true
  def handle_event("update_column_filter", %{"col" => col, "value" => val}, socket) do
    filters =
      if val == "" do
        Map.delete(socket.assigns.column_filter, col)
      else
        Map.put(socket.assigns.column_filter, col, val)
      end

    {:noreply, assign(socket, column_filter: filters)}
  end

  @impl true
  def handle_event("export_bundle", _params, socket) do
    case socket.assigns.query_id do
      nil ->
        {:noreply, assign(socket, export_error: "No query to export")}

      query_id ->
        current_user = socket.assigns[:current_user]

        {:noreply,
         socket
         |> assign(exporting: true, export_result: nil, export_error: nil)
         |> start_async(:export, fn -> trigger_export(query_id, current_user) end)}
    end
  end

  # ---------------------------------------------------------------------------
  # Async task callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def handle_async(:run_query, {:ok, {:ok, result}}, socket) do
    query_id = get_in(result, ["query_id"]) || Map.get(result, :query_id)

    {:noreply,
     assign(socket,
       running: false,
       query_result: result,
       query_id: query_id,
       query_error: nil
     )}
  end

  def handle_async(:run_query, {:ok, {:error, msg}}, socket) do
    {:noreply, assign(socket, running: false, query_error: msg)}
  end

  def handle_async(:run_query, {:exit, reason}, socket) do
    {:noreply, assign(socket, running: false, query_error: "Query crashed: #{inspect(reason)}")}
  end

  def handle_async(:nl_translate, {:ok, {:ok, result}}, socket) do
    proposed = get_in(result, ["proposed", "query"])
    explanation = get_in(result, ["proposed", "explanation"])
    confidence = get_in(result, ["proposed", "confidence"])

    {:noreply,
     assign(socket,
       running: false,
       proposed_dsl: proposed,
       nl_explanation: explanation,
       nl_confidence: confidence,
       dsl_text: Jason.encode!(proposed, pretty: true),
       query_error: nil
     )}
  end

  def handle_async(:nl_translate, {:ok, {:error, msg}}, socket) do
    {:noreply, assign(socket, running: false, query_error: msg)}
  end

  def handle_async(:nl_translate, {:exit, reason}, socket) do
    {:noreply, assign(socket, running: false, query_error: "Translation crashed: #{inspect(reason)}")}
  end

  def handle_async(:export, {:ok, {:ok, result}}, socket) do
    {:noreply, assign(socket, exporting: false, export_result: result, export_error: nil)}
  end

  def handle_async(:export, {:ok, {:error, msg}}, socket) do
    {:noreply, assign(socket, exporting: false, export_error: msg)}
  end

  def handle_async(:export, {:exit, reason}, socket) do
    {:noreply, assign(socket, exporting: false, export_error: "Export crashed: #{inspect(reason)}")}
  end

  # ---------------------------------------------------------------------------
  # Render
  # ---------------------------------------------------------------------------

  @impl true
  def render(assigns) do
    ~H"""
    <div class="eel-telemetry-query" role="main" aria-labelledby="query-heading">
      <h1 id="query-heading">Telemetry query builder</h1>

      <div style="display:grid;grid-template-columns:300px 1fr 1fr;gap:16px;align-items:start">

        <%!-- Left pane: presets + DSL editor --%>
        <aside aria-label="Query source" style="background:#fff;border:1px solid #e5e5e5;border-radius:8px;padding:16px">
          <h2 style="font-size:1rem;margin:0 0 10px">Preset library</h2>
          <select
            phx-change="select_preset"
            name="name"
            aria-label="Select a preset query"
            style="width:100%;padding:6px;border:1px solid #d1d5db;border-radius:4px;margin-bottom:14px"
          >
            <option value="">-- choose a preset --</option>
            <%= for p <- @presets do %>
              <option value={p["name"]} selected={@preset_selected == p["name"]}>
                <%= p["name"] %>
              </option>
            <% end %>
          </select>

          <h2 style="font-size:1rem;margin:0 0 6px">Custom DSL (JSON)</h2>
          <textarea
            phx-change="update_dsl"
            name="dsl"
            rows="14"
            aria-label="Custom DSL editor"
            spellcheck="false"
            style="width:100%;font-family:monospace;font-size:0.82rem;padding:8px;border:1px solid #d1d5db;border-radius:4px;resize:vertical"
          ><%= @dsl_text %></textarea>

          <button
            phx-click="run_dsl"
            class="btn btn-primary"
            disabled={@running}
            aria-busy={@running}
            style="margin-top:10px;width:100%;padding:10px"
          >
            <%= if @running, do: "Running...", else: "Run DSL query" %>
          </button>
        </aside>

        <%!-- Middle pane: NL bridge --%>
        <section aria-labelledby="nl-heading" style="background:#fff;border:1px solid #e5e5e5;border-radius:8px;padding:16px">
          <h2 id="nl-heading" style="font-size:1rem;margin:0 0 10px">Natural-language query</h2>

          <form phx-submit="run_nl">
            <label for="nl-input" class="sr-only">Query in plain English</label>
            <textarea
              id="nl-input"
              name="nl_text"
              rows="5"
              placeholder="Which kits at station 12345 came within 2 metres of a ballot box between 17:00 and 19:00?"
              style="width:100%;padding:8px;border:1px solid #d1d5db;border-radius:4px;font-size:0.9rem;resize:vertical"
            ><%= @nl_text %></textarea>
            <button type="submit" disabled={@running} style="margin-top:8px;padding:8px 16px;background:#6b7280;color:#fff;border:0;border-radius:4px;cursor:pointer;min-height:44px;width:100%;font-weight:600">
              <%= if @running, do: "Translating...", else: "Translate to DSL" %>
            </button>
          </form>

          <%= if @proposed_dsl do %>
            <div style="margin-top:14px;border:1px solid #d97706;border-radius:6px;padding:12px;background:#fefce8">
              <p style="margin:0 0 6px;font-weight:600">Proposed DSL</p>
              <%= if @nl_explanation do %>
                <p style="font-size:0.85rem;color:#6b7280;margin:0 0 6px"><%= @nl_explanation %></p>
              <% end %>
              <%= if @nl_confidence do %>
                <p style="font-size:0.82rem;color:#92400e;margin:0 0 10px">Confidence: <%= @nl_confidence %></p>
              <% end %>
              <button
                phx-click="confirm_nl_dsl"
                disabled={@running}
                style="padding:8px 16px;background:#2563eb;color:#fff;border:0;border-radius:4px;cursor:pointer;min-height:44px;font-weight:600"
                aria-label="Confirm and run the proposed DSL query"
              >
                Confirm and run
              </button>
            </div>
          <% end %>

          <%= if @query_error do %>
            <p role="alert" style="color:#dc2626;margin-top:10px"><%= @query_error %></p>
          <% end %>
        </section>

        <%!-- Right pane: results --%>
        <section aria-labelledby="results-heading" style="background:#fff;border:1px solid #e5e5e5;border-radius:8px;padding:16px">
          <h2 id="results-heading" style="font-size:1rem;margin:0 0 10px">Results</h2>

          <%= if @query_result do %>
            <p role="status" aria-live="polite">
              <strong><%= get_in(@query_result, ["count"]) || 0 %></strong> row(s) returned.
            </p>

            <%!-- Export buttons --%>
            <div style="display:flex;gap:8px;margin-bottom:12px;flex-wrap:wrap">
              <button
                phx-click="export_bundle"
                disabled={@exporting or is_nil(@query_id)}
                style="padding:6px 12px;background:#16a34a;color:#fff;border:0;border-radius:4px;cursor:pointer;font-size:0.88rem;min-height:36px"
                aria-label="Export evidence bundle (PDF, CSV, Parquet)"
              >
                <%= if @exporting, do: "Exporting...", else: "Export evidence bundle" %>
              </button>
            </div>

            <%= if @export_result do %>
              <div style="font-size:0.85rem;background:#f0fdf4;border:1px solid #86efac;border-radius:4px;padding:10px;margin-bottom:12px">
                <p style="margin:0;font-weight:600;color:#15803d">Bundle generated</p>
                <p style="margin:4px 0 0;font-family:monospace;font-size:0.78rem;word-break:break-all">
                  SHA-256: <%= get_in(@export_result, ["manifest", "manifest_sha256"]) %>
                </p>
              </div>
            <% end %>

            <%= if @export_error do %>
              <p role="alert" style="color:#dc2626;font-size:0.85rem;margin-bottom:10px"><%= @export_error %></p>
            <% end %>

            <%!-- Result table with column filter --%>
            <% rows = get_in(@query_result, ["rows"]) || [] %>
            <%= if rows != [] do %>
              <.result_table rows={rows} column_filter={@column_filter} />
            <% else %>
              <p style="color:#6b7280">No rows matched the query.</p>
            <% end %>
          <% else %>
            <p style="color:#6b7280" role="status">Run a query to see results here.</p>
          <% end %>
        </section>

      </div>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Components
  # ---------------------------------------------------------------------------

  defp result_table(assigns) do
    rows = assigns.rows
    filters = assigns.column_filter

    visible_rows =
      Enum.filter(rows, fn row ->
        Enum.all?(filters, fn {col, val} ->
          cell = row[col] || ""
          String.contains?(String.downcase(to_string(cell)), String.downcase(val))
        end)
      end)

    columns =
      rows
      |> List.first(%{})
      |> Map.keys()
      |> Enum.take(10)

    assigns = assign(assigns, visible_rows: visible_rows, columns: columns)

    ~H"""
    <div style="overflow-x:auto">
      <table role="grid" style="width:100%;font-size:0.82rem;border-collapse:collapse">
        <thead>
          <tr>
            <%= for col <- @columns do %>
              <th scope="col" style="padding:6px;border-bottom:2px solid #e5e5e5;text-align:left">
                <%= col %>
                <input
                  type="text"
                  phx-change="update_column_filter"
                  name="value"
                  value={@column_filter[col] || ""}
                  phx-value-col={col}
                  placeholder="filter"
                  aria-label={"Filter by #{col}"}
                  style="display:block;width:100%;margin-top:2px;padding:2px 4px;border:1px solid #d1d5db;border-radius:3px;font-size:0.78rem"
                >
              </th>
            <% end %>
          </tr>
        </thead>
        <tbody>
          <%= for row <- Enum.take(@visible_rows, 200) do %>
            <tr style="border-bottom:1px solid #f3f4f6">
              <%= for col <- @columns do %>
                <td style="padding:4px 6px;max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
                  <%= row[col] %>
                </td>
              <% end %>
            </tr>
          <% end %>
          <%= if length(@visible_rows) > 200 do %>
            <tr>
              <td colspan={length(@columns)} style="padding:6px;color:#6b7280;font-style:italic">
                Showing 200 of <%= length(@visible_rows) %> filtered rows.
              </td>
            </tr>
          <% end %>
        </tbody>
      </table>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp fetch_presets do
    case Req.get("#{api_base()}/v1/telemetry/presets") do
      {:ok, %{status: 200, body: %{"presets" => presets}}} -> presets
      _ -> []
    end
  rescue
    _ -> []
  end

  defp execute_query(dsl, current_user) do
    actor = current_user |> actor_svid()

    case Req.post("#{api_base()}/v1/telemetry/query",
           json: dsl,
           headers: [{"x-actor-svid", actor}]
         ) do
      {:ok, %{status: s, body: body}} when s in 200..299 ->
        {:ok, body}

      {:ok, %{status: s, body: body}} ->
        {:error, "HTTP #{s}: #{inspect(body)}"}

      {:error, reason} ->
        {:error, inspect(reason)}
    end
  rescue
    e -> {:error, inspect(e)}
  end

  defp translate_nl(text, current_user) do
    actor = current_user |> actor_svid()

    case Req.post("#{api_base()}/v1/telemetry/query/nl",
           json: %{"text" => text},
           headers: [{"x-actor-svid", actor}]
         ) do
      {:ok, %{status: s, body: body}} when s in 200..299 ->
        {:ok, body}

      {:ok, %{status: s, body: body}} ->
        {:error, "HTTP #{s}: #{inspect(body)}"}

      {:error, reason} ->
        {:error, inspect(reason)}
    end
  rescue
    e -> {:error, inspect(e)}
  end

  defp trigger_export(query_id, current_user) do
    actor = current_user |> actor_svid()

    case Req.post("#{api_base()}/v1/telemetry/queries/#{query_id}/export",
           json: %{},
           headers: [{"x-actor-svid", actor}]
         ) do
      {:ok, %{status: s, body: body}} when s in 200..299 ->
        {:ok, body}

      {:ok, %{status: s, body: body}} ->
        {:error, "HTTP #{s}: #{inspect(body)}"}

      {:error, reason} ->
        {:error, inspect(reason)}
    end
  rescue
    e -> {:error, inspect(e)}
  end

  defp run_api_post(path, body, _socket) do
    case Req.post("#{api_base()}#{path}", json: body) do
      {:ok, %{status: s, body: resp_body}} when s in 200..299 -> {:ok, resp_body}
      {:ok, %{status: s}} -> {:error, "HTTP #{s}"}
      {:error, reason} -> {:error, inspect(reason)}
    end
  rescue
    e -> {:error, inspect(e)}
  end

  defp actor_svid(nil), do: "anonymous"
  defp actor_svid(user) when is_map(user), do: Map.get(user, :svid, "user://#{Map.get(user, :id, "unknown")}")

  defp api_base,
    do: Application.get_env(:ecc_live, :telemetry_api_url, "http://telemetry-query:8080")

  defp default_dsl do
    now = DateTime.utc_now()
    from = DateTime.add(now, -3600, :second)

    Jason.encode!(
      %{
        time_range: %{
          from: DateTime.to_iso8601(from),
          to: DateTime.to_iso8601(now)
        },
        signal_classes: ["location"],
        limit: 100
      },
      pretty: true
    )
  end
end
