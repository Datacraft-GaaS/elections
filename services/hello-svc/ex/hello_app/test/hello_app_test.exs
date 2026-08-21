defmodule HelloAppTest do
  use ExUnit.Case, async: true

  test "greeting returns ok: true" do
    assert HelloApp.greeting() == %{ok: true}
  end
end
