defmodule EccLive.Live.KitsLive do
  @moduledoc "Kit telemetry heatmap view."
  use EccLive, :live_view

  require Logger

  @impl true
  def mount(_params, _session, socket) do
    # TODO: call ecc-api GET /api/v1/ecc/kits via Req
    {:ok, assign(socket, :kits, [])}
  end

  @impl true
  def handle_info({:kit_update, kit}, socket) do
    Logger.info("kit telemetry update", kit_id: kit.id, status: kit.status)
    kits =
      socket.assigns.kits
      |> Enum.map(fn k -> if k.id == kit.id, do: kit, else: k end)
    {:noreply, assign(socket, :kits, kits)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-kits">
      <h1>Kit Telemetry Heatmap</h1>
      <div class="heatmap">
        <%= for kit <- @kits do %>
          <div class={"kit-cell kit-#{kit.status}"} title={"#{kit.code}: #{kit.status}"}>
            <%= kit.code %>
          </div>
        <% end %>
        <%= if Enum.empty?(@kits) do %>
          <p>No kit data available.</p>
        <% end %>
      </div>
    </div>
    """
  end
end
