defmodule PortalLive.Live.CountsLive do
  @moduledoc """
  LiveView: real-time aggregate count stream.

  Subscribes to PubSub topic "eel:counts" which is driven by a Kafka consumer
  projecting eel.rts.snapshot_published.v1 events.

  Displays Merkle root cross-verified against the portal-api verify endpoint.
  No PII: only aggregate counts and Merkle metadata are broadcast.
  """
  use Phoenix.LiveView

  @topic "eel:counts"

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket) do
      Phoenix.PubSub.subscribe(PortalLive.PubSub, @topic)
    end

    {:ok,
     assign(socket,
       counts: %{},
       merkle_root: nil,
       published_at: nil,
       snapshot_id: nil,
       stale: false,
       page_title: "Live Results"
     )}
  end

  @impl true
  def handle_info({:snapshot_updated, snapshot}, socket) do
    {:noreply,
     assign(socket,
       counts: snapshot.counts,
       merkle_root: snapshot.merkle_root,
       published_at: snapshot.published_at,
       snapshot_id: snapshot.snapshot_id,
       stale: false
     )}
  end

  @impl true
  def handle_info(:mark_stale, socket) do
    {:noreply, assign(socket, stale: true)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <main id="main" tabindex="-1" aria-label="Live election results">
      <h1>Live Results</h1>

      <%= if @stale do %>
        <div role="alert" aria-live="assertive" class="stale-warning">
          Data may be stale. Last update: <%= @published_at %>
        </div>
      <% end %>

      <section aria-labelledby="h-merkle">
        <h2 id="h-merkle">Merkle Root</h2>
        <dl>
          <dt>Root</dt>
          <dd><code aria-label="merkle root"><%= @merkle_root || "Pending" %></code></dd>
          <dt>Snapshot</dt>
          <dd><code><%= @snapshot_id || "Pending" %></code></dd>
          <dt>Published</dt>
          <dd><%= @published_at || "Pending" %></dd>
        </dl>
      </section>

      <section aria-labelledby="h-counts">
        <h2 id="h-counts">Aggregate Counts</h2>
        <%= if map_size(@counts) == 0 do %>
          <p>Waiting for results...</p>
        <% else %>
          <table aria-label="Live vote counts">
            <thead>
              <tr><th scope="col">Candidate</th><th scope="col">Votes</th></tr>
            </thead>
            <tbody>
              <%= for {candidate_id, votes} <- @counts do %>
                <tr>
                  <td><%= candidate_id %></td>
                  <td><%= votes %></td>
                </tr>
              <% end %>
            </tbody>
          </table>
        <% end %>
      </section>
    </main>
    """
  end
end
