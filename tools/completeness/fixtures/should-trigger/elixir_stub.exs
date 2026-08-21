# Fixture: should trigger exunit_tag_skip and todo_comment

defmodule StubTest do
  use ExUnit.Case

  # TODO: remove once implemented
  @tag :skip
  test "stub test" do
    assert 1 == 1
  end
end
