defmodule EccLive.Live.DashboardLiveTest do
  @moduledoc "ExUnit unit tests for DashboardLive."
  use ExUnit.Case, async: true

  alias EccLive.Live.DashboardLive

  # ---------------------------------------------------------------------------
  # Module-level sanity
  # ---------------------------------------------------------------------------

  test "module is loaded" do
    assert Code.ensure_loaded?(DashboardLive)
  end

  test "module exposes live view surface" do
    assert function_exported?(DashboardLive, :mount, 3)
    assert function_exported?(DashboardLive, :render, 1)
    assert function_exported?(DashboardLive, :handle_info, 2)
  end

  # ---------------------------------------------------------------------------
  # Mount / data loading
  # ---------------------------------------------------------------------------

  test "mount assigns a fully normalised summary even when the api is unreachable" do
    {:ok, socket} = DashboardLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    assert socket.assigns.stations_total == 0
    assert socket.assigns.stations_reporting == 0
    assert socket.assigns.alerts_active == 0
    assert socket.assigns.turnout_pct == 0.0
  end

  # ---------------------------------------------------------------------------
  # Push payload handling
  # ---------------------------------------------------------------------------

  test "dashboard_update push maps string keyed payload onto assigns" do
    {:ok, socket} = DashboardLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    payload = %{
      "total_stations" => 46_000,
      "reporting_stations" => 45_100,
      "active_alerts" => 7,
      "turnout_pct" => 55.4
    }

    {:noreply, socket} = DashboardLive.handle_info({:dashboard_update, payload}, socket)

    assert socket.assigns.stations_total == 46_000
    assert socket.assigns.stations_reporting == 45_100
    assert socket.assigns.alerts_active == 7
    assert socket.assigns.turnout_pct == 55.4
  end

  test "partial push payload fills missing fields with safe defaults" do
    {:ok, socket} = DashboardLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    {:noreply, socket} =
      DashboardLive.handle_info({:dashboard_update, %{"active_alerts" => 3}}, socket)

    assert socket.assigns.alerts_active == 3
    assert socket.assigns.stations_total == 0
    assert socket.assigns.turnout_pct == 0.0
  end

  test "non map push payloads degrade to zeroed summary" do
    {:ok, socket} = DashboardLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    {:noreply, socket} = DashboardLive.handle_info({:dashboard_update, :garbage}, socket)

    assert socket.assigns.stations_total == 0
  end
end
