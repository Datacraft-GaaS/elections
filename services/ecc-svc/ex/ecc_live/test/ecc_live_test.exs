defmodule EccLiveTest do
  use ExUnit.Case, async: true

  test "application module is defined" do
    assert Code.ensure_loaded?(EccLive.Application)
  end

  test "endpoint module is defined" do
    assert Code.ensure_loaded?(EccLive.Endpoint)
  end

  test "router module is defined" do
    assert Code.ensure_loaded?(EccLive.Router)
  end
end
