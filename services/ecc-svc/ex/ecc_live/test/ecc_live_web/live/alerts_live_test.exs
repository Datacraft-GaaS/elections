defmodule EccLive.Live.AlertsLiveTest do
  @moduledoc "ExUnit unit tests for AlertsLive."
  use ExUnit.Case, async: true

  alias EccLive.Live.AlertsLive

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp alert(overrides \\ %{}) do
    Map.merge(
      %{
        id: "aaaaaaaa-0000-0000-0000-000000000001",
        station_code: "029-004",
        category: "security",
        severity: "critical",
        status: "open",
        summary: "Presiding officer reported a security incident"
      },
      Map.new(overrides)
    )
  end

  defp with_feed(alerts) do
    {:ok, socket} = AlertsLive.mount(nil, nil, %Phoenix.LiveView.Socket{})
    # :flash is reserved, so seed it via a direct struct update instead of
    # assign/2.
    socket = %{socket | assigns: Map.put(socket.assigns, :flash, %{})}

    Phoenix.Component.assign(socket, :alerts, alerts)
  end

  defp render_alerts(socket) do
    assigns =
      Map.merge(
        %{__changed__: %{}, flash: socket.assigns[:flash] || %{}},
        socket.assigns
      )

    AlertsLive.render(assigns)
    |> Phoenix.LiveViewTest.rendered_to_string()
  end
  # ---------------------------------------------------------------------------
  # Module-level sanity
  # ---------------------------------------------------------------------------

  test "module is loaded" do
    assert Code.ensure_loaded?(AlertsLive)
  end

  test "module exposes live view surface" do
    assert function_exported?(AlertsLive, :mount, 3)
    assert function_exported?(AlertsLive, :render, 1)
    assert function_exported?(AlertsLive, :handle_event, 3)
  end

  # ---------------------------------------------------------------------------
  # Mount / feed loading
  # ---------------------------------------------------------------------------

  test "mount assigns an empty feed when the api is unreachable" do
    {:ok, socket} = AlertsLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    assert socket.assigns.alerts == []
  end

  # ---------------------------------------------------------------------------
  # Dispatch behaviour (ecc-api unreachable in unit tests: failure path)
  # ---------------------------------------------------------------------------

  test "failed dispatch keeps the escalation open and raises an error flash" do
    socket = with_feed([alert(), alert(id: "b", severity: "high")])

    {:noreply, socket} =
      AlertsLive.handle_event(
        "dispatch",
        %{"alert_id" => "aaaaaaaa-0000-0000-0000-000000000001"},
        socket
      )

    assert Enum.map(socket.assigns.alerts, & &1.status) == ~w(open open)
    assert match?(%{"error" => <<"Failed to dispatch alert ", _::binary>>}, socket.assigns[:flash])
  end

  test "acknowledged alerts are rendered without a dispatch button" do
    socket = with_feed([alert(%{status: "acknowledged"})])
    html = render_alerts(socket)

    refute html =~ "Dispatch</button>"
    assert html =~ "acknowledged"
  end

  test "open alerts are rendered with a dispatch button" do
    socket = with_feed([alert()])
    html = render_alerts(socket)

    assert html =~ ~s(phx-click="dispatch")
    assert html =~ alert().summary
  end
end
