defmodule EccLive.Live.S44aLive do
  @moduledoc "Section 44A dual-officer verification flow."
  use EccLive, :live_view

  require Logger

  # Dual-officer states: :idle, :officer1_confirmed, :complete
  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:step, :idle)
     |> assign(:station_code, nil)
     |> assign(:officer1_id, nil)}
  end

  @impl true
  def handle_event("officer1_confirm", %{"officer_id" => oid, "station_code" => sc}, socket) do
    Logger.info("s44a officer1 confirmed", officer_id: oid, station_code: sc)
    # TODO: POST /api/v1/ecc/s44a/officer1 via Req
    {:noreply,
     socket
     |> assign(:step, :officer1_confirmed)
     |> assign(:station_code, sc)
     |> assign(:officer1_id, oid)}
  end

  @impl true
  def handle_event("officer2_confirm", %{"officer_id" => oid}, socket) do
    Logger.info("s44a officer2 confirmed", officer_id: oid, station_code: socket.assigns.station_code)
    # TODO: POST /api/v1/ecc/s44a/officer2 via Req
    {:noreply, assign(socket, :step, :complete)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-s44a">
      <h1>Section 44A Dual-Officer Verification</h1>
      <%= case @step do %>
        <% :idle -> %>
          <form phx-submit="officer1_confirm">
            <input type="text" name="station_code" placeholder="Station Code" required />
            <input type="text" name="officer_id" placeholder="Officer 1 ID" required />
            <button type="submit">Confirm Officer 1</button>
          </form>
        <% :officer1_confirmed -> %>
          <p>Officer 1 confirmed for station <%= @station_code %>. Awaiting Officer 2.</p>
          <form phx-submit="officer2_confirm">
            <input type="text" name="officer_id" placeholder="Officer 2 ID" required />
            <button type="submit">Confirm Officer 2</button>
          </form>
        <% :complete -> %>
          <p class="success">Dual-officer verification complete for station <%= @station_code %>.</p>
      <% end %>
    </div>
    """
  end
end
