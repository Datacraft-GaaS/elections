defmodule EccLive.Live.CoverHistoryLive do
  @moduledoc """
  Sub-page: full state-transition timeline for a specific RFID cover tag.

  Reached via /cover-tamper/history?tag_id=<uuid> or /cover-tamper/history?epc=<hex>.
  Verifies the prev_hash chain in-memory and surfaces any broken links.
  """
  use EccLive, :live_view

  require Logger

  @impl true
  def mount(_params, _session, socket) do
    {:ok, assign(socket, tag: nil, transitions: [], chain_errors: [], tag_query: nil)}
  end

  @impl true
  def handle_params(%{"tag_id" => tag_id} = _params, _url, socket) do
    {tag, transitions} = load_tag_history(tag_id: tag_id)
    errors = verify_chain(transitions)
    {:noreply, assign(socket, tag: tag, transitions: transitions, chain_errors: errors, tag_query: tag_id)}
  end

  def handle_params(%{"epc" => epc} = _params, _url, socket) do
    {tag, transitions} = load_tag_history(epc: epc)
    errors = verify_chain(transitions)
    {:noreply, assign(socket, tag: tag, transitions: transitions, chain_errors: errors, tag_query: epc)}
  end

  def handle_params(_params, _url, socket) do
    {:noreply, socket}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-cover-history" role="main" aria-labelledby="cover-history-heading">
      <h1 id="cover-history-heading">Cover tag state-transition history</h1>

      <p><.link navigate={~p"/cover-tamper"}>Back to tamper alerts</.link></p>

      <%= if @tag do %>
        <section aria-labelledby="tag-meta-heading">
          <h2 id="tag-meta-heading">Tag <code><%= @tag["epc"] %></code></h2>
          <dl style="display:grid;grid-template-columns:max-content 1fr;gap:2px 12px">
            <dt>Tag ID</dt><dd><code><%= @tag["tag_id"] %></code></dd>
            <dt>Box</dt><dd><%= @tag["paired_with_box"] || "(unbound)" %></dd>
            <dt>Cover state</dt><dd><%= @tag["cover_state"] %></dd>
            <dt>Tamper state</dt><dd><%= @tag["tamper_state"] %></dd>
          </dl>
        </section>

        <%= if @chain_errors != [] do %>
          <div role="alert" aria-live="assertive"
               style="background:#fde8e8;border:2px solid #c0392b;padding:12px;margin:12px 0;border-radius:4px">
            <strong>Chain integrity errors detected:</strong>
            <ul>
              <%= for {idx, msg} <- @chain_errors do %>
                <li>Transition <%= idx %>: <%= msg %></li>
              <% end %>
            </ul>
          </div>
        <% end %>

        <section aria-labelledby="transitions-heading">
          <h2 id="transitions-heading">Transitions (<%= length(@transitions) %>)</h2>
          <%= if @transitions == [] do %>
            <p>No transitions recorded.</p>
          <% else %>
            <table role="grid" aria-label="State transition timeline" style="width:100%;border-collapse:collapse">
              <thead>
                <tr>
                  <th scope="col">#</th>
                  <th scope="col">From</th>
                  <th scope="col">To</th>
                  <th scope="col">Officer</th>
                  <th scope="col">When</th>
                  <th scope="col">Hash chain</th>
                  <th scope="col">Merkle anchor</th>
                </tr>
              </thead>
              <tbody>
                <%= for {t, idx} <- Enum.with_index(@transitions, 1) do %>
                  <tr>
                    <td><%= idx %></td>
                    <td><%= t["from_state"] %></td>
                    <td><%= t["to_state"] %></td>
                    <td><%= t["officer_id"] %></td>
                    <td><time datetime={t["transitioned_at"]}><%= t["transitioned_at"] %></time></td>
                    <td>
                      <%= if t["chain_valid"] do %>
                        <span style="color:#27ae60" aria-label="Chain valid">&#10003; valid</span>
                      <% else %>
                        <span style="color:#c0392b;font-weight:bold" aria-label="Chain BROKEN">&#9888; BROKEN</span>
                      <% end %>
                    </td>
                    <td>
                      <%= if t["merkle_anchor_id"] do %>
                        <code title={t["merkle_anchor_id"]}><%= String.slice(t["merkle_anchor_id"], 0, 20) %>...</code>
                      <% else %>
                        <span style="color:#888">pending</span>
                      <% end %>
                    </td>
                  </tr>
                <% end %>
              </tbody>
            </table>
          <% end %>
        </section>
      <% else %>
        <%= if @tag_query do %>
          <p role="alert">No tag found for query "<%= @tag_query %>".</p>
        <% else %>
          <p>Provide a <code>tag_id</code> or <code>epc</code> query parameter.</p>
        <% end %>
      <% end %>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp load_tag_history(query) do
    param = Enum.into(query, %{}, fn {k, v} -> {to_string(k), v} end)

    case Req.get("#{base_url()}/v1/rfid/tags/history", params: param) do
      {:ok, %{status: 200, body: body}} ->
        tag = Map.get(body, "tag")
        transitions = Map.get(body, "transitions", [])
        {tag, transitions}

      _ ->
        {nil, []}
    end
  rescue
    _ -> {nil, []}
  end

  defp verify_chain(transitions) do
    transitions
    |> Enum.with_index(1)
    |> Enum.reduce({[], nil}, fn {t, idx}, {errors, prev} ->
      error =
        cond do
          is_nil(prev) ->
            nil

          t["chain_valid"] == false ->
            {idx, "prev_hash does not match this_hash of transition #{idx - 1}"}

          true ->
            nil
        end

      {if(error, do: [error | errors], else: errors), t}
    end)
    |> elem(0)
    |> Enum.reverse()
  end

  defp base_url, do: Application.get_env(:ecc_live, :inventory_api_url, "http://inventory-svc:8080")
end
