# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

defmodule Eel.EventStoreTest do
  use ExUnit.Case, async: true

  # Test aggregate that counts increment events.
  defmodule TestCounter do
    @behaviour Eel.EventStore.Aggregate

    @impl true
    def initial_state, do: %{count: 0}

    @impl true
    def apply(%{count: n}, %{type: "increment"}), do: %{count: n + 1}
    def apply(%{count: n}, %{"type" => "increment"}), do: %{count: n + 1}
    def apply(state, _event), do: state

    @impl true
    def snapshot(%{count: n}) when n > 0, do: {:ok, %{count: n}}
    def snapshot(_state), do: :skip
  end

  describe "Eel.EventStore.Aggregate behaviour" do
    test "initial_state returns empty state" do
      assert TestCounter.initial_state() == %{count: 0}
    end

    test "apply increments count" do
      state = TestCounter.initial_state()
      state = TestCounter.apply(state, %{type: "increment"})
      state = TestCounter.apply(state, %{type: "increment"})
      assert state.count == 2
    end

    test "apply ignores unknown events" do
      state = %{count: 5}
      assert TestCounter.apply(state, %{type: "unknown"}) == %{count: 5}
    end

    test "snapshot returns :skip for zero count" do
      assert TestCounter.snapshot(%{count: 0}) == :skip
    end

    test "snapshot returns {:ok, data} for non-zero count" do
      assert TestCounter.snapshot(%{count: 3}) == {:ok, %{count: 3}}
    end
  end

  describe "Eel.EventStore.Event schema" do
    test "schema fields exist" do
      fields = Eel.EventStore.Event.__schema__(:fields)
      assert :id in fields
      assert :stream_id in fields
      assert :version in fields
      assert :type in fields
      assert :data in fields
      assert :metadata in fields
      assert :prev_hash in fields
      assert :inserted_at in fields
    end

    test "schema source is events" do
      assert Eel.EventStore.Event.__schema__(:source) == "events"
    end
  end

  describe "Eel.EventStore.Snapshot schema" do
    test "schema fields exist" do
      fields = Eel.EventStore.Snapshot.__schema__(:fields)
      assert :id in fields
      assert :stream_id in fields
      assert :version in fields
      assert :state in fields
      assert :inserted_at in fields
    end

    test "schema source is event_snapshots" do
      assert Eel.EventStore.Snapshot.__schema__(:source) == "event_snapshots"
    end
  end

  describe "event fold logic" do
    test "reduces events over initial state correctly" do
      events = [
        %{type: "increment"},
        %{type: "increment"},
        %{type: "increment"}
      ]

      {final_state, final_version} =
        Enum.reduce(events, {TestCounter.initial_state(), 0}, fn event, {state, _ver} ->
          new_state = TestCounter.apply(state, event)
          {new_state, :erlang.unique_integer([:positive])}
        end)

      assert final_state.count == 3
      assert is_integer(final_version)
    end
  end
end
