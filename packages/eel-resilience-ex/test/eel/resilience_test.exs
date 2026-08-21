defmodule Eel.ResilienceTest do
  use ExUnit.Case, async: false

  alias Eel.Resilience.CircuitBreaker
  alias Eel.Resilience.DegradationFlag
  alias Eel.Resilience.Retry
  alias Eel.Resilience.Timeout

  describe "CircuitBreaker" do
    test "starts in :closed state" do
      {:ok, cb} = CircuitBreaker.start_link(threshold: 3)
      assert %{state: :closed, failure_count: 0} = CircuitBreaker.state(cb)
    end

    test "opens after threshold failures" do
      {:ok, cb} = CircuitBreaker.start_link(threshold: 3)
      failing = fn -> {:error, :boom} end

      CircuitBreaker.call(cb, failing)
      CircuitBreaker.call(cb, failing)
      CircuitBreaker.call(cb, failing)

      assert %{state: :open} = CircuitBreaker.state(cb)
    end

    test "returns :circuit_open when open" do
      {:ok, cb} = CircuitBreaker.start_link(threshold: 1)
      CircuitBreaker.call(cb, fn -> {:error, :boom} end)
      assert {:error, :circuit_open} = CircuitBreaker.call(cb, fn -> :ok end)
    end

    test "returns success result when closed" do
      {:ok, cb} = CircuitBreaker.start_link(threshold: 5)
      assert :hello = CircuitBreaker.call(cb, fn -> :hello end)
    end
  end

  describe "Retry" do
    test "returns error when function always fails" do
      result = Retry.with_retry(fn -> {:error, :boom} end, max_attempts: 2, base_delay_ms: 1, jitter: false)
      assert {:error, :max_retries_exceeded} = result
    end

    test "returns result on success" do
      counter = :counters.new(1, [])
      result = Retry.with_retry(
        fn ->
          :counters.add(counter, 1, 1)
          n = :counters.get(counter, 1)
          if n < 2, do: {:error, :not_yet}, else: :done
        end,
        max_attempts: 3,
        base_delay_ms: 1,
        jitter: false
      )
      assert :done = result
    end
  end

  describe "Timeout" do
    test "returns result when function completes in time" do
      assert :ok = Timeout.with_timeout(fn -> :ok end, 1000)
    end

    test "returns {:error, :timeout} when function exceeds timeout" do
      result = Timeout.with_timeout(fn -> Process.sleep(500) end, 10)
      assert {:error, :timeout} = result
    end
  end

  describe "DegradationFlag" do
    setup do
      DegradationFlag.set(:normal)
      :ok
    end

    test "defaults to :normal" do
      assert :normal = DegradationFlag.get()
    end

    test "degraded? returns false for :normal" do
      refute DegradationFlag.degraded?()
    end

    test "degraded? returns true after set(:degraded)" do
      DegradationFlag.set(:degraded)
      assert DegradationFlag.degraded?()
    end

    test "read_only? returns true after set(:read_only)" do
      DegradationFlag.set(:read_only)
      assert DegradationFlag.read_only?()
    end

    test "read_only? returns false for :degraded" do
      DegradationFlag.set(:degraded)
      refute DegradationFlag.read_only?()
    end
  end

  describe "IdempotencyPlug" do
    import Plug.Test
    import Plug.Conn

    alias Eel.Resilience.IdempotencyPlug

    defp call_plug(conn) do
      IdempotencyPlug.call(conn, IdempotencyPlug.init([]))
    end

    test "passes through requests without idempotency key" do
      conn = conn(:post, "/test", "body")
      result = call_plug(conn)
      refute result.halted
    end

    test "passes through GET requests even with idempotency key" do
      conn =
        :get
        |> conn("/test", "")
        |> put_req_header("idempotency-key", "test-key-get")

      result = call_plug(conn)
      refute result.halted
    end

    test "caches and replays response for same key and body" do
      key = "replay-key-#{:erlang.unique_integer([:positive])}"

      first =
        :post
        |> conn("/test", ~s({"data":"value"}))
        |> put_req_header("idempotency-key", key)
        |> put_req_header("content-type", "application/json")
        |> call_plug()

      refute first.halted

      first = send_resp(first, 200, ~s({"result":"ok"}))
      Enum.each(first.before_send, fn f -> f.(first) end)

      second =
        :post
        |> conn("/test", ~s({"data":"value"}))
        |> put_req_header("idempotency-key", key)
        |> put_req_header("content-type", "application/json")
        |> call_plug()

      assert second.halted
      assert second.status == 200
    end
  end
end
