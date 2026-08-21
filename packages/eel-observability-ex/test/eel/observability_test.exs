defmodule Eel.ObservabilityTest do
  use ExUnit.Case, async: true

  alias Eel.Observability
  alias Eel.Observability.AuditShadow
  alias Eel.Observability.LogScrubber
  alias Eel.Observability.Slo

  describe "AuditShadow.trigger_sql/2" do
    test "returns SQL containing TRIGGER keyword" do
      sql = AuditShadow.trigger_sql("voters")
      assert String.contains?(sql, "TRIGGER")
    end

    test "returns SQL containing the table name" do
      sql = AuditShadow.trigger_sql("voters")
      assert String.contains?(sql, "voters")
    end

    test "defaults schema to public" do
      sql = AuditShadow.trigger_sql("voters")
      assert String.contains?(sql, "public.voters")
    end

    test "uses provided schema" do
      sql = AuditShadow.trigger_sql("voters", "election")
      assert String.contains?(sql, "election.voters")
    end

    test "trigger name contains table name" do
      sql = AuditShadow.trigger_sql("constituencies")
      assert String.contains?(sql, "audit_constituencies")
    end
  end

  describe "Slo.slo_compliant?/2" do
    test "returns {:ok, true}" do
      assert {:ok, true} = Slo.slo_compliant?("voter-registry", 3600)
    end
  end

  describe "LogScrubber" do
    test "init callback succeeds" do
      assert {:ok, state} = LogScrubber.init([])
      assert is_map(state)
    end

    test "handle_event returns ok for flush" do
      {:ok, state} = LogScrubber.init([])
      assert {:ok, ^state} = LogScrubber.handle_event(:flush, state)
    end

    test "handle_event returns ok for unknown events" do
      {:ok, state} = LogScrubber.init([])
      assert {:ok, ^state} = LogScrubber.handle_event(:unknown, state)
    end
  end

  describe "Observability.start_span/3" do
    test "returns the function result" do
      result = Observability.start_span("test.span", %{}, fn -> :hello end)
      assert result == :hello
    end

    test "works with nested spans" do
      result =
        Observability.start_span("outer", %{}, fn ->
          Observability.start_span("inner", %{"key" => "value"}, fn ->
            42
          end)
        end)

      assert result == 42
    end
  end
end
