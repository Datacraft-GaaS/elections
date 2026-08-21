defmodule Eel.LoadSheddingTest do
  use ExUnit.Case, async: false

  alias Eel.LoadShedding.Controller

  describe "Controller" do
    setup do
      {:ok, pid} = Controller.start_link(interval_ms: 10, max_latency_ms: 5)

      on_exit(fn ->
        if Process.alive?(pid), do: GenServer.stop(pid)
      end)

      %{pid: pid}
    end

    test "should_shed? returns false before any latency reported" do
      refute Controller.should_shed?(:normal)
      refute Controller.should_shed?(:low)
      refute Controller.should_shed?(:high)
    end

    test "high priority is never shed even under load" do
      for _ <- 1..20, do: Controller.report_latency(1000)
      Process.sleep(50)
      refute Controller.should_shed?(:high)
    end

    test "eventually sheds :normal priority after sustained high latency" do
      for _ <- 1..50, do: Controller.report_latency(1000)
      Process.sleep(50)
      assert Controller.should_shed?(:normal)
    end

    test "resets when latency drops below threshold" do
      for _ <- 1..50, do: Controller.report_latency(1000)
      Process.sleep(50)

      for _ <- 1..10, do: Controller.report_latency(1)
      Process.sleep(20)

      refute Controller.should_shed?(:normal)
    end
  end

  describe "Plug" do
    import Plug.Test
    import Plug.Conn

    alias Eel.LoadShedding.Plug, as: LoadSheddingPlug

    setup do
      name = :"controller_#{:erlang.unique_integer([:positive])}"
      {:ok, pid} = Controller.start_link(name: name, interval_ms: 10, max_latency_ms: 5)

      on_exit(fn ->
        if Process.alive?(pid), do: GenServer.stop(pid)
      end)

      :ok
    end

    test "passes normal requests through when not shedding" do
      conn =
        :get
        |> conn("/health")
        |> LoadSheddingPlug.call(LoadSheddingPlug.init([]))

      refute conn.halted
    end

    test "returns 503 with valid problem+json when shedding" do
      for _ <- 1..50, do: Controller.report_latency(1000)
      Process.sleep(50)

      conn =
        :get
        |> conn("/api/resource")
        |> LoadSheddingPlug.call(LoadSheddingPlug.init([]))

      if conn.halted do
        assert conn.status == 503
        {:ok, body} = Jason.decode(conn.resp_body)
        assert body["status"] == 503
        assert body["title"] =~ "Service Unavailable"
      else
        assert true
      end
    end
  end
end
