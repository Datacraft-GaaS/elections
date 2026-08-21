defmodule SpeechAnalystLive.Live.FlagsLive do
  @moduledoc "Lists content flags with filter by type and status."
  use SpeechAnalystLive, :live_view

  @flag_types ~w(misinformation hate_speech incitement deepfake cib all)
  @statuses ~w(pending confirmed rejected all)

  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:flags, [])
     |> assign(:filter_type, "all")
     |> assign(:filter_status, "all")
     |> assign(:flag_types, @flag_types)
     |> assign(:statuses, @statuses)}
  end

  @impl true
  def handle_event("filter_type", %{"type" => type}, socket) do
    {:noreply, assign(socket, :filter_type, type)}
  end

  @impl true
  def handle_event("filter_status", %{"status" => status}, socket) do
    {:noreply, assign(socket, :filter_status, status)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="flags-list">
      <h1>Content Flags</h1>
      <div class="filters">
        <label>Type:
          <select phx-change="filter_type" name="type">
            <%= for t <- @flag_types do %>
              <option value={t} selected={t == @filter_type}><%= t %></option>
            <% end %>
          </select>
        </label>
        <label>Status:
          <select phx-change="filter_status" name="status">
            <%= for s <- @statuses do %>
              <option value={s} selected={s == @filter_status}><%= s %></option>
            <% end %>
          </select>
        </label>
      </div>
      <table>
        <thead>
          <tr>
            <th>Flag ID</th><th>Item ID</th><th>Type</th><th>Confidence</th><th>Status</th>
          </tr>
        </thead>
        <tbody>
          <%= for flag <- @flags do %>
            <tr>
              <td><%= flag["flag_id"] %></td>
              <td><%= flag["item_id"] %></td>
              <td><%= flag["flag_type"] %></td>
              <td><%= flag["confidence"] %></td>
              <td><%= flag["analyst_decision"] %></td>
            </tr>
          <% end %>
        </tbody>
      </table>
    </div>
    """
  end
end
