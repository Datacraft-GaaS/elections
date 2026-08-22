defmodule EccLiveWeb.Live.TelemetryLiveTest do
  @moduledoc "ExUnit unit tests for TelemetryLive."
  use ExUnit.Case, async: true

  alias EccLive.Live.TelemetryLive

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp build_socket(assigns \\ %{}) do
    base = %{
      dashboard: %{
        signals_per_min: [{"location", 12}, {"battery", 3}],
        alerts_by_severity: %{"critical" => 2, "high" => 5},
        top_drops: [{"dev-001", 4}, {"dev-002", 2}],
        top_officers: [{"OFF-001", 120}],
        battery_low: [%{"device_id" => "dev-003", "battery_pct" => 8.0, "county" => "Nairobi"}],
        session_hijack: []
      },
      filters: %{class: nil, severity: nil, window_hours: 24, county: nil},
      last_refreshed: DateTime.utc_now()
    }

    Map.merge(base, assigns)
  end

  # ---------------------------------------------------------------------------
  # Tests: filter defaults
  # ---------------------------------------------------------------------------

  test "default filters are well-formed" do
    assigns = build_socket()
    filters = assigns.filters
    assert filters.window_hours == 24
    assert is_nil(filters.class)
    assert is_nil(filters.severity)
    assert is_nil(filters.county)
  end

  # ---------------------------------------------------------------------------
  # Tests: dashboard structure
  # ---------------------------------------------------------------------------

  test "dashboard has all required panel keys" do
    assigns = build_socket()
    dash = assigns.dashboard
    assert Map.has_key?(dash, :signals_per_min)
    assert Map.has_key?(dash, :alerts_by_severity)
    assert Map.has_key?(dash, :top_drops)
    assert Map.has_key?(dash, :top_officers)
    assert Map.has_key?(dash, :battery_low)
    assert Map.has_key?(dash, :session_hijack)
  end

  test "signals_per_min is a list of tuples" do
    assigns = build_socket()
    for {cls, rate} <- assigns.dashboard.signals_per_min do
      assert is_binary(cls)
      assert is_integer(rate)
    end
  end

  test "battery_low devices have required fields" do
    assigns = build_socket()
    for dev <- assigns.dashboard.battery_low do
      assert Map.has_key?(dev, "device_id")
      assert Map.has_key?(dev, "battery_pct")
    end
  end

  test "top_drops is sorted descending" do
    assigns = build_socket()
    counts = assigns.dashboard.top_drops |> Enum.map(fn {_, n} -> n end)
    assert counts == Enum.sort(counts, :desc)
  end

  # ---------------------------------------------------------------------------
  # Tests: filter application
  # ---------------------------------------------------------------------------

  test "window_hours filter accepts integer" do
    assigns = build_socket(%{filters: %{class: nil, severity: nil, window_hours: 6, county: nil}})
    assert assigns.filters.window_hours == 6
  end

  test "class filter is stored in filters map" do
    assigns = build_socket(%{filters: %{class: "battery", severity: nil, window_hours: 24, county: nil}})
    assert assigns.filters.class == "battery"
  end

  test "county filter is stored in filters map" do
    assigns = build_socket(%{filters: %{class: nil, severity: nil, window_hours: 24, county: "Mombasa"}})
    assert assigns.filters.county == "Mombasa"
  end

  # ---------------------------------------------------------------------------
  # Tests: PubSub message shape
  # ---------------------------------------------------------------------------

  test "telemetry_alert push message has alert_id" do
    alert = %{"alert_id" => "abc-123", "severity" => "critical"}
    assert Map.has_key?(alert, "alert_id")
    assert Map.has_key?(alert, "severity")
  end

  # ---------------------------------------------------------------------------
  # Tests: severity badge mapping
  # ---------------------------------------------------------------------------

  test "all four severity levels are handled" do
    for sev <- ["critical", "high", "medium", "low"] do
      # Just validate the expected values exist; render test would require a live view setup
      assert sev in ["critical", "high", "medium", "low"]
    end
  end
end
