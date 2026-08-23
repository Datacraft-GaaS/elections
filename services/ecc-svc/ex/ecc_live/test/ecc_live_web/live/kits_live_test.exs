defmodule EccLive.Live.KitsLiveTest do
  @moduledoc "ExUnit unit tests for KitsLive."
  use ExUnit.Case, async: true

  alias EccLive.Live.KitsLive

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp kit(overrides \\ %{}) do
    Map.merge(
      %{
        id: "029-004",
        code: "029-004",
        name: "Kilifi Primary School",
        constituency: "029",
        county: "003",
        status: "online",
        last_updated: "2026-08-09T06:30:00Z"
      },
      Map.new(overrides)
    )
  end

  # ---------------------------------------------------------------------------
  # Module-level sanity
  # ---------------------------------------------------------------------------

  test "module is loaded" do
    assert Code.ensure_loaded?(KitsLive)
  end

  test "module exposes live view surface" do
    assert function_exported?(KitsLive, :mount, 3)
    assert function_exported?(KitsLive, :render, 1)
    assert function_exported?(KitsLive, :handle_info, 2)
  end

  # ---------------------------------------------------------------------------
  # Mount / grid loading
  # ---------------------------------------------------------------------------

  test "mount assigns an empty grid when the api is unreachable" do
    {:ok, socket} = KitsLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    assert socket.assigns.kits == []
  end

  # ---------------------------------------------------------------------------
  # Push handling
  # ---------------------------------------------------------------------------

  test "kit_update inserts unknown kits into the grid" do
    {:ok, socket} = KitsLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    {:noreply, socket} = KitsLive.handle_info({:kit_update, kit()}, socket)

    assert length(socket.assigns.kits) == 1
    assert hd(socket.assigns.kits).code == "029-004"
  end

  test "kit_update replaces existing kits matched on id" do
    {:ok, socket} = KitsLive.mount(nil, nil, %Phoenix.LiveView.Socket{})
    {:noreply, socket} = KitsLive.handle_info({:kit_update, kit()}, socket)

    {:noreply, socket} =
      KitsLive.handle_info({:kit_update, kit(status: "offline")}, socket)

    assert length(socket.assigns.kits) == 1
    assert hd(socket.assigns.kits).status == "offline"
  end

  test "kit_update keeps unrelated kits" do
    {:ok, socket} = KitsLive.mount(nil, nil, %Phoenix.LiveView.Socket{})
    {:noreply, socket} = KitsLive.handle_info({:kit_update, kit(id: "001-001", code: "001-001")}, socket)

    {:noreply, socket} = KitsLive.handle_info({:kit_update, kit(status: "degraded")}, socket)

    assert length(socket.assigns.kits) == 2
    assert Enum.any?(socket.assigns.kits, &(&1.code == "001-001"))
    assert Enum.any?(socket.assigns.kits, &(&1.status == "degraded"))
  end
end
