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

defmodule Eel.EventStore.Aggregate do
  @moduledoc """
  Behaviour for event-sourced aggregates.

  Implementing modules define how events are applied to state
  and how snapshots are produced.
  """

  @doc """
  Applies an event to the current aggregate state, returning new state.
  """
  @callback apply(state :: term(), event :: map()) :: term()

  @doc """
  Produces a snapshot of the current state, or `:skip` to skip snapshotting.
  """
  @callback snapshot(state :: term()) :: {:ok, snapshot_data :: map()} | :skip

  @doc """
  Returns the initial (empty) state for this aggregate.
  """
  @callback initial_state() :: term()
end
