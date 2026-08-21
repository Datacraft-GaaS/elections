defmodule SpeechAnalystLive.FlagsLiveTest do
  use ExUnit.Case, async: true

  test "FlagsLive module exists" do
    assert Code.ensure_loaded?(SpeechAnalystLive.Live.FlagsLive)
  end

  test "DashboardLive module exists" do
    assert Code.ensure_loaded?(SpeechAnalystLive.Live.DashboardLive)
  end

  test "AdjudicationLive module exists" do
    assert Code.ensure_loaded?(SpeechAnalystLive.Live.AdjudicationLive)
  end

  test "HealthController module exists" do
    assert Code.ensure_loaded?(SpeechAnalystLive.Controllers.HealthController)
  end
end
