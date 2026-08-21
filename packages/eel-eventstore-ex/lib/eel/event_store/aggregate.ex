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
