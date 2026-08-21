defmodule Eel.Outbox do
  @moduledoc """
  Transactional outbox pattern for IEBC IEMS event publishing.

  `emit/2` appends a CloudEvent to the `outbox` database table as part of an
  existing `Ecto.Multi`, ensuring atomicity with the surrounding business
  transaction. A separate relay process reads un-published rows and forwards
  them to Kafka.

  ## Usage

      {:ok, _result} =
        Ecto.Multi.new()
        |> MyApp.Repo.multi_insert_form(form_params)
        |> Eel.Outbox.emit(envelope)
        |> MyApp.Repo.transaction()

  ## Size limit

  Payloads exceeding 1 MB are rejected with `{:error, :payload_too_large}`.
  Use `Eel.Outbox.Blob.maybe_offload/2` to store large blobs in MinIO and
  emit a compact reference envelope instead.
  """

  alias Eel.Outbox.Entry

  @max_payload_bytes 1_048_576

  @doc """
  Append a CloudEvent envelope to the outbox as part of `multi`.

  `envelope` is any map (or struct convertible to a map). The `topic` for
  the Kafka destination is derived from the `type` field of the envelope.

  Returns the updated `Ecto.Multi` on success, or
  `{:error, :payload_too_large}` when the serialised payload exceeds 1 MB.
  """
  @spec emit(Ecto.Multi.t(), map()) :: Ecto.Multi.t() | {:error, :payload_too_large}
  def emit(%Ecto.Multi{} = multi, envelope) when is_map(envelope) do
    json = encode_envelope(envelope)

    if byte_size(json) > @max_payload_bytes do
      {:error, :payload_too_large}
    else
      event_id = fetch_field(envelope, "id", :id, "unknown")
      event_type = fetch_field(envelope, "type", :type, "unknown")
      topic = event_type

      attrs = %{
        event_id: event_id,
        event_type: event_type,
        topic: topic,
        payload: json
      }

      changeset = Entry.changeset(%Entry{}, attrs)
      Ecto.Multi.insert(multi, {:outbox_entry, event_id}, changeset)
    end
  end

  defp encode_envelope(envelope) when is_struct(envelope) do
    envelope
    |> Map.from_struct()
    |> Enum.reject(fn {_k, v} -> is_nil(v) end)
    |> Map.new()
    |> Jason.encode!()
  end

  defp encode_envelope(envelope), do: Jason.encode!(envelope)

  defp fetch_field(map, str_key, atom_key, default) do
    case Map.fetch(map, str_key) do
      {:ok, v} when not is_nil(v) -> v
      _ ->
        case Map.fetch(map, atom_key) do
          {:ok, v} when not is_nil(v) -> v
          _ -> default
        end
    end
  end
end
