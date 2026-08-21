defmodule QueueMapLive.Live.StationMapLiveTest do
  use ExUnit.Case, async: true

  test "application module is defined" do
    assert Code.ensure_loaded?(QueueMapLive.Application)
  end

  test "endpoint module is defined" do
    assert Code.ensure_loaded?(QueueMapLive.Endpoint)
  end

  test "router module is defined" do
    assert Code.ensure_loaded?(QueueMapLive.Router)
  end

  test "station map live module is defined" do
    assert Code.ensure_loaded?(QueueMapLive.Live.StationMapLive)
  end

  test "health controller is defined" do
    assert Code.ensure_loaded?(QueueMapLive.Controllers.HealthController)
  end
end
