defmodule Eel.HlcTest do
  use ExUnit.Case, async: true

  alias Eel.Hlc
  alias Eel.Hlc.Clock

  # --- from_string / to_string interop ---

  test "parses Rust-produced format '1723731000123.5.42' correctly" do
    assert {:ok, hlc} = Hlc.from_string("1723731000123.5.42")
    assert hlc.physical_ms == 1_723_731_000_123
    assert hlc.counter == 5
    assert hlc.node == "42"
  end

  test "round-trips through to_string" do
    original = "1723731000123.5.42"
    {:ok, hlc} = Hlc.from_string(original)
    assert Hlc.to_string(hlc) == original
  end

  test "from_string rejects non-string input" do
    assert {:error, _} = Hlc.from_string(42)
  end

  test "from_string rejects wrong segment count" do
    assert {:error, _} = Hlc.from_string("123.4")
    assert {:error, _} = Hlc.from_string("123.4.5.6")
  end

  test "from_string rejects non-integer segments" do
    assert {:error, _} = Hlc.from_string("abc.4.node1")
    assert {:error, _} = Hlc.from_string("123.xyz.node1")
  end

  test "from_string rejects negative integers" do
    assert {:error, _} = Hlc.from_string("-1.0.node1")
  end

  test "from_string rejects empty node" do
    assert {:error, _} = Hlc.from_string("123.4.")
  end

  # --- new ---

  test "new/3 creates struct" do
    hlc = Hlc.new(1000, 0, "node1")
    assert hlc.physical_ms == 1000
    assert hlc.counter == 0
    assert hlc.node == "node1"
  end

  # --- tick ---

  test "tick advances physical_ms when wall is ahead" do
    hlc = Hlc.new(1000, 5, "n1")
    result = Hlc.tick(hlc, 2000)
    assert result.physical_ms == 2000
    assert result.counter == 0
  end

  test "tick increments counter when wall is not ahead" do
    hlc = Hlc.new(2000, 5, "n1")
    result = Hlc.tick(hlc, 1500)
    assert result.physical_ms == 2000
    assert result.counter == 6
  end

  test "tick increments counter when wall equals physical_ms" do
    hlc = Hlc.new(2000, 3, "n1")
    result = Hlc.tick(hlc, 2000)
    assert result.physical_ms == 2000
    assert result.counter == 4
  end

  # --- merge ---

  test "merge: local physical ahead" do
    local = Hlc.new(3000, 2, "n1")
    recv = Hlc.new(1000, 10, "n2")
    result = Hlc.merge(local, recv)
    assert result.physical_ms == 3000
    assert result.counter == 3
    assert result.node == "n1"
  end

  test "merge: received physical ahead" do
    local = Hlc.new(1000, 2, "n1")
    recv = Hlc.new(5000, 3, "n2")
    result = Hlc.merge(local, recv)
    assert result.physical_ms == 5000
    assert result.counter == 4
    assert result.node == "n1"
  end

  test "merge: same physical, keeps max counter + 1" do
    local = Hlc.new(1000, 2, "n1")
    recv = Hlc.new(1000, 7, "n2")
    result = Hlc.merge(local, recv)
    assert result.physical_ms == 1000
    assert result.counter == 8
    assert result.node == "n1"
  end

  # --- compare ---

  test "compare: less physical is :lt" do
    a = Hlc.new(100, 0, "n1")
    b = Hlc.new(200, 0, "n1")
    assert Hlc.compare(a, b) == :lt
    assert Hlc.compare(b, a) == :gt
  end

  test "compare: same physical, less counter is :lt" do
    a = Hlc.new(100, 1, "n1")
    b = Hlc.new(100, 5, "n1")
    assert Hlc.compare(a, b) == :lt
  end

  test "compare: same physical and counter, node decides" do
    a = Hlc.new(100, 1, "aaa")
    b = Hlc.new(100, 1, "zzz")
    assert Hlc.compare(a, b) == :lt
    assert Hlc.compare(b, a) == :gt
  end

  test "compare: identical values are :eq" do
    a = Hlc.new(100, 1, "n1")
    b = Hlc.new(100, 1, "n1")
    assert Hlc.compare(a, b) == :eq
  end

  # --- Clock GenServer ---

  test "Clock.now/1 returns an HLC struct" do
    {:ok, pid} = start_supervised({Clock, node: "test-node"})
    hlc = Clock.now(pid)
    assert %Hlc{} = hlc
    assert hlc.node == "test-node"
  end

  test "Clock.now/1 is monotonically non-decreasing" do
    {:ok, pid} = start_supervised({Clock, node: "mono-node"})
    a = Clock.now(pid)
    b = Clock.now(pid)
    assert Hlc.compare(a, b) == :lt or Hlc.compare(a, b) == :eq
  end

  test "Clock.observe/2 merges an external HLC" do
    {:ok, pid} = start_supervised({Clock, node: "obs-node"})
    future = Hlc.new(:erlang.system_time(:millisecond) + 10_000, 0, "remote")
    assert :ok = Clock.observe(future, pid)
    next = Clock.now(pid)
    # physical_ms must be at or beyond the observed future time
    assert next.physical_ms >= future.physical_ms
  end
end
