defmodule Eel.EventStore.Replay do
  @moduledoc """
  Replays events for a stream to rebuild aggregate state.

  Uses snapshots as a starting point when available, unless `from_version`
  is supplied.
  """

  import Ecto.Query

  alias Eel.EventStore.{Event, Snapshot}

  @doc """
  Replays events for `stream_id` through `aggregate_module`, returning
  the final state and version.

  Options:
  - `from_version` - skip snapshots and start replay from this version.

  Returns `{:ok, state, version}` or `{:error, reason}`.
  """
  @spec replay(module(), String.t(), module(), keyword()) ::
          {:ok, term(), non_neg_integer()} | {:error, term()}
  def replay(repo, stream_id, aggregate_module, opts \\ []) do
    from_version = Keyword.get(opts, :from_version)

    {initial_state, start_version} =
      if from_version do
        {aggregate_module.initial_state(), from_version}
      else
        load_snapshot(repo, stream_id, aggregate_module)
      end

    events_query =
      from(e in Event,
        where: e.stream_id == ^stream_id and e.version > ^start_version,
        order_by: [asc: e.version]
      )

    case repo.all(events_query) do
      events when is_list(events) ->
        {final_state, final_version} =
          Enum.reduce(events, {initial_state, start_version}, fn event, {state, _ver} ->
            new_state = aggregate_module.apply(state, event)
            {new_state, event.version}
          end)

        {:ok, final_state, final_version}

      _ ->
        {:error, :query_failed}
    end
  end

  defp load_snapshot(repo, stream_id, aggregate_module) do
    snapshot_query =
      from(s in Snapshot,
        where: s.stream_id == ^stream_id,
        order_by: [desc: s.version],
        limit: 1
      )

    case repo.one(snapshot_query) do
      nil ->
        {aggregate_module.initial_state(), 0}

      snapshot ->
        {snapshot.state, snapshot.version}
    end
  end
end
