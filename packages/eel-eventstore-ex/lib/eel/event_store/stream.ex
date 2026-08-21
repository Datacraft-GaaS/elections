defmodule Eel.EventStore.Stream do
  @moduledoc """
  Appends events to a named stream with optimistic concurrency control.
  """

  alias Eel.EventStore.Event

  @select_for_update "SELECT version FROM event_streams WHERE stream_id = $1 FOR UPDATE"

  @doc """
  Appends `events` to the stream identified by `stream_id`.

  `expected_version` must match the stream's current version, or be `0` when
  creating a new stream. Returns `{:ok, new_version}` on success,
  `{:error, :version_conflict}` on mismatch, or `{:error, reason}` for
  other failures.
  """
  @spec append_events(module(), String.t(), [map()], non_neg_integer()) ::
          {:ok, non_neg_integer()} | {:error, term()}
  def append_events(repo, stream_id, events, expected_version) do
    multi =
      Ecto.Multi.new()
      |> Ecto.Multi.run(:lock_stream, fn repo_arg, _changes ->
        case Ecto.Adapters.SQL.query(repo_arg, @select_for_update, [stream_id]) do
          {:ok, %{rows: []}} ->
            if expected_version == 0 do
              {:ok, :new}
            else
              {:error, :version_conflict}
            end

          {:ok, %{rows: [[current_version]]}} ->
            if current_version == expected_version do
              {:ok, {:existing, current_version}}
            else
              {:error, :version_conflict}
            end

          {:error, reason} ->
            {:error, reason}
        end
      end)
      |> Ecto.Multi.run(:upsert_stream, fn repo_arg, %{lock_stream: lock_result} ->
        new_version = expected_version + length(events)

        case lock_result do
          :new ->
            case Ecto.Adapters.SQL.query(
                   repo_arg,
                   "INSERT INTO event_streams (stream_id, version) VALUES ($1, $2)",
                   [stream_id, new_version]
                 ) do
              {:ok, _} -> {:ok, new_version}
              {:error, reason} -> {:error, reason}
            end

          {:existing, _current_version} ->
            case Ecto.Adapters.SQL.query(
                   repo_arg,
                   "UPDATE event_streams SET version = $1 WHERE stream_id = $2",
                   [new_version, stream_id]
                 ) do
              {:ok, _} -> {:ok, new_version}
              {:error, reason} -> {:error, reason}
            end
        end
      end)
      |> Ecto.Multi.run(:insert_events, fn repo_arg, %{upsert_stream: new_version} ->
        start_version = expected_version + 1
        total = length(events)

        event_records =
          events
          |> Enum.with_index(start_version)
          |> Enum.map(fn {event, version} ->
            %{
              stream_id: stream_id,
              version: version,
              type: Map.get(event, :type, Map.get(event, "type")),
              data: Map.get(event, :data, Map.get(event, "data", %{})),
              metadata: Map.get(event, :metadata, Map.get(event, "metadata", %{})),
              prev_hash: Map.get(event, :prev_hash, Map.get(event, "prev_hash")),
              inserted_at: DateTime.utc_now()
            }
          end)

        {count, _} = repo_arg.insert_all(Event, event_records)

        if count == total do
          {:ok, new_version}
        else
          {:error, :partial_insert}
        end
      end)

    case repo.transaction(multi) do
      {:ok, %{insert_events: new_version}} -> {:ok, new_version}
      {:error, _step, :version_conflict, _changes} -> {:error, :version_conflict}
      {:error, _step, reason, _changes} -> {:error, reason}
    end
  end
end
