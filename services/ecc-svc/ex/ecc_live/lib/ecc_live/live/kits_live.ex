defmodule EccLive.Live.KitsLive do
  @moduledoc """
  Kit telemetry heatmap view.

  Cells are polling stations from the ecc-api station status list
  (GET /ecc/stations), keyed by station code and coloured by operational
  status. The grid refreshes every 30 seconds; live updates arrive as
  {:kit_update, kit} messages carrying a kit map with the same shape.
  """
  use EccLive, :live_view

  require Logger

  @refresh_ms 30_000

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket), do: schedule_refresh()

    {:ok, assign(socket, :kits, fetch_kits())}
  end

  @impl true
  def handle_info({:kit_update, kit}, socket) do
    Logger.info("kit telemetry update", kit_id: kit.id, status: kit.status)

    kits =
      socket.assigns.kits
      |> Enum.map(fn k -> if k.id == kit.id, do: kit, else: k end)
      |> Kernel.then(fn updated ->
        if Enum.any?(updated, &(&1.id == kit.id)), do: updated, else: [kit | updated]
      end)

    {:noreply, assign(socket, :kits, kits)}
  end

  @impl true
  def handle_info(:auto_refresh, socket) do
    schedule_refresh()
    {:noreply, assign(socket, :kits, fetch_kits())}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-kits">
      <h1>Kit Telemetry Heatmap</h1>
      <div class="heatmap">
        <%= for kit <- @kits do %>
          <div class={"kit-cell kit-#{kit.status}"} title={"#{kit.code} #{kit.name}: #{kit.status}"}>
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

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp fetch_kits do
    case Req.get("#{api_base()}/ecc/stations", params: [page_size: 500]) do
      {:ok, %{status: 200, body: %{"stations" => stations}}} ->
        Enum.map(stations, &to_kit/1)

      other ->
        Logger.warning("station list fetch failed", response: inspect(other))
        []
    end
  rescue
    e ->
      Logger.warning("station list fetch raised", error: Exception.message(e))
      []
  end

  defp to_kit(station) do
    %{
      id: station["station_code"],
      code: station["station_code"],
      name: station["name"] || station["station_code"],
      constituency: station["constituency_code"],
      county: station["county_code"],
      status: station["status"] || "unknown",
      last_updated: station["last_updated"]
    }
  end

  defp api_base, do: Application.get_env(:ecc_live, :ecc_api_url, "http://ecc-api:8080")

  defp schedule_refresh do
    Process.send_after(self(), :auto_refresh, @refresh_ms)
  end
end
