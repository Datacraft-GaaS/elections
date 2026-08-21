defmodule Eel.EventStore do
  @moduledoc """
  Public API for the IEBC IEMS event store.

  Provides event appending with optimistic concurrency and event replay
  for rebuilding aggregate state.
  """

  alias Eel.EventStore.{Stream, Replay}

  @doc """
  Appends events to the stream identified by `stream_id`.

  `expected_version` is the version the caller believes the stream is at.
  Pass `0` when creating a new stream. Returns `{:ok, new_version}` on
  success or `{:error, :version_conflict}` when another writer has
  already appended events.
  """
  defdelegate append_events(repo, stream_id, events, expected_version), to: Stream

  @doc """
  Rebuilds aggregate state for `stream_id` by replaying events through
  `aggregate_module`.

  Loads the latest snapshot if available. Accepts `[from_version: integer]`
  to skip snapshots and start replay from a specific version.

  Returns `{:ok, state, version}` or `{:error, reason}`.
  """
  defdelegate replay(repo, stream_id, aggregate_module, opts \\ []), to: Replay
end
