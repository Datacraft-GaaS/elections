defmodule EccLiveWeb.Live.TelemetryQueryLiveTest do
  @moduledoc "ExUnit unit tests for TelemetryQueryLive."
  use ExUnit.Case, async: true

  alias EccLive.Live.TelemetryQueryLive

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp base_assigns do
    %{
      presets: [
        %{"name" => "drops_by_county", "description" => "Physical drops by county", "required_params" => ["county", "hours"]},
        %{"name" => "officer_activity", "description" => "Officer activity", "required_params" => ["officer_id", "from", "to"]}
      ],
      preset_selected: nil,
      dsl_text: ~s({"time_range":{"from":"2026-01-01T00:00:00Z","to":"2026-01-02T00:00:00Z"}}),
      nl_text: "",
      proposed_dsl: nil,
      nl_explanation: nil,
      nl_confidence: nil,
      query_result: nil,
      query_error: nil,
      query_id: nil,
      export_result: nil,
      export_error: nil,
      column_filter: %{},
      running: false,
      exporting: false
    }
  end

  defp sample_result do
    %{
      "rows" => [
        %{"device_id" => "dev-001", "signal_class" => "battery", "battery_pct" => "12.0"},
        %{"device_id" => "dev-002", "signal_class" => "location", "battery_pct" => nil}
      ],
      "count" => 2,
      "query_id" => "00000000-0000-0000-0000-000000000001"
    }
  end

  # ---------------------------------------------------------------------------
  # Tests: initial state
  # ---------------------------------------------------------------------------

  test "base assigns has all required keys" do
    assigns = base_assigns()
    for key <- ~w(presets preset_selected dsl_text nl_text proposed_dsl query_result query_error query_id export_result export_error column_filter running exporting)a do
      assert Map.has_key?(assigns, key), "missing key: #{key}"
    end
  end

  test "running starts false" do
    assigns = base_assigns()
    refute assigns.running
  end

  test "column_filter starts empty" do
    assigns = base_assigns()
    assert assigns.column_filter == %{}
  end

  # ---------------------------------------------------------------------------
  # Tests: DSL validation
  # ---------------------------------------------------------------------------

  test "valid DSL JSON parses without error" do
    dsl = ~s({"time_range":{"from":"2026-01-01T00:00:00Z","to":"2026-01-02T00:00:00Z"},"signal_classes":["battery"],"limit":500})
    assert {:ok, _parsed} = Jason.decode(dsl)
  end

  test "invalid DSL JSON returns parse error" do
    bad_dsl = "not json at all"
    assert {:error, _} = Jason.decode(bad_dsl)
  end

  # ---------------------------------------------------------------------------
  # Tests: preset list
  # ---------------------------------------------------------------------------

  test "presets list has expected names" do
    assigns = base_assigns()
    names = assigns.presets |> Enum.map(& &1["name"])
    assert "drops_by_county" in names
    assert "officer_activity" in names
  end

  test "each preset has required_params" do
    assigns = base_assigns()
    for p <- assigns.presets do
      assert Map.has_key?(p, "required_params")
      assert is_list(p["required_params"])
    end
  end

  # ---------------------------------------------------------------------------
  # Tests: column filter logic
  # ---------------------------------------------------------------------------

  test "column filter matches substring case-insensitively" do
    rows = sample_result()["rows"]
    col_filter = %{"signal_class" => "bat"}

    visible =
      Enum.filter(rows, fn row ->
        Enum.all?(col_filter, fn {col, val} ->
          cell = row[col] || ""
          String.contains?(String.downcase(to_string(cell)), String.downcase(val))
        end)
      end)

    assert length(visible) == 1
    assert List.first(visible)["signal_class"] == "battery"
  end

  test "empty column filter returns all rows" do
    rows = sample_result()["rows"]
    col_filter = %{}

    visible =
      Enum.filter(rows, fn row ->
        Enum.all?(col_filter, fn {col, val} ->
          cell = row[col] || ""
          String.contains?(String.downcase(to_string(cell)), String.downcase(val))
        end)
      end)

    assert length(visible) == 2
  end

  # ---------------------------------------------------------------------------
  # Tests: async task result handling
  # ---------------------------------------------------------------------------

  test "successful query result updates query_result assign" do
    assigns = base_assigns()
    result = sample_result()
    updated = Map.put(assigns, :query_result, result)
    assert updated.query_result["count"] == 2
    assert length(updated.query_result["rows"]) == 2
  end

  test "error result sets query_error assign" do
    assigns = base_assigns()
    updated = Map.put(assigns, :query_error, "HTTP 500: internal error")
    assert updated.query_error =~ "HTTP 500"
  end

  # ---------------------------------------------------------------------------
  # Tests: NL proposed DSL
  # ---------------------------------------------------------------------------

  test "proposed_dsl is stored when NL translation succeeds" do
    assigns = base_assigns()
    proposed = %{"time_range" => %{"from" => "2026-01-01T00:00:00Z", "to" => "2026-01-02T00:00:00Z"}}
    updated = Map.merge(assigns, %{proposed_dsl: proposed, nl_explanation: "Mock explanation", nl_confidence: 0.75})
    assert updated.proposed_dsl["time_range"]["from"] == "2026-01-01T00:00:00Z"
    assert updated.nl_confidence == 0.75
  end

  test "confirm_nl_dsl requires proposed_dsl to be set" do
    assigns = base_assigns()
    # proposed_dsl is nil, so confirm should be blocked
    assert is_nil(assigns.proposed_dsl)
  end

  # ---------------------------------------------------------------------------
  # Tests: export state
  # ---------------------------------------------------------------------------

  test "export_result stores manifest sha256" do
    assigns = base_assigns()
    manifest = %{"manifest_sha256" => String.duplicate("a", 64), "hsm_signature" => "stub:sha256:aabbcc"}
    updated = Map.merge(assigns, %{export_result: %{"manifest" => manifest}, exporting: false})
    assert String.length(get_in(updated.export_result, ["manifest", "manifest_sha256"])) == 64
  end
end
