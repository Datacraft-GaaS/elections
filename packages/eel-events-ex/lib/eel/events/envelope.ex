defmodule Eel.Events.Envelope do
  @moduledoc """
  CloudEvents 1.0 envelope with the Commission IEMS-specific extension attributes.

  Covers the required CloudEvents core attributes (`specversion`, `id`,
  `type`, `source`) and the optional ones (`subject`, `time`,
  `datacontenttype`, `dataschema`, `data`) plus all `eel_*` extensions
  defined in EVENT-CONVENTIONS.md.
  """

  @type t :: %__MODULE__{
          specversion: String.t() | nil,
          id: String.t() | nil,
          type: String.t() | nil,
          source: String.t() | nil,
          subject: String.t() | nil,
          time: String.t() | nil,
          datacontenttype: String.t() | nil,
          dataschema: String.t() | nil,
          data: term(),
          eel_stream_id: String.t() | nil,
          eel_seq: integer() | nil,
          eel_hlc: String.t() | nil,
          eel_prev_hash: String.t() | nil,
          eel_correlation_id: String.t() | nil,
          eel_causation_id: String.t() | nil,
          eel_producer_key_id: String.t() | nil,
          eel_signature: String.t() | nil,
          eel_election_id: String.t() | nil,
          eel_data_class: String.t() | nil
        }

  defstruct [
    :specversion,
    :id,
    :type,
    :source,
    :subject,
    :time,
    :datacontenttype,
    :dataschema,
    :data,
    :eel_stream_id,
    :eel_seq,
    :eel_hlc,
    :eel_prev_hash,
    :eel_correlation_id,
    :eel_causation_id,
    :eel_producer_key_id,
    :eel_signature,
    :eel_election_id,
    :eel_data_class
  ]

  @fields [
    :specversion,
    :id,
    :type,
    :source,
    :subject,
    :time,
    :datacontenttype,
    :dataschema,
    :data,
    :eel_stream_id,
    :eel_seq,
    :eel_hlc,
    :eel_prev_hash,
    :eel_correlation_id,
    :eel_causation_id,
    :eel_producer_key_id,
    :eel_signature,
    :eel_election_id,
    :eel_data_class
  ]

  @doc """
  Parse an envelope from a map with string or atom keys, as returned by
  `Jason.decode!/1`. Returns `{:ok, %Eel.Events.Envelope{}}` or
  `{:error, reason}`.
  """
  @spec from_map(map()) :: {:ok, t()} | {:error, term()}
  def from_map(map) when is_map(map) do
    attrs =
      Enum.reduce(@fields, %{}, fn field, acc ->
        str_key = to_string(field)

        value =
          case Map.fetch(map, str_key) do
            {:ok, v} -> v
            :error -> Map.get(map, field)
          end

        if is_nil(value), do: acc, else: Map.put(acc, field, value)
      end)

    {:ok, struct(__MODULE__, attrs)}
  rescue
    e -> {:error, Exception.message(e)}
  end

  @doc """
  Convert an envelope to a plain map with string keys matching the
  CloudEvents spec. Nil fields are omitted.
  """
  @spec to_map(t()) :: map()
  def to_map(%__MODULE__{} = envelope) do
    envelope
    |> Map.from_struct()
    |> Enum.reject(fn {_k, v} -> is_nil(v) end)
    |> Enum.map(fn {k, v} -> {to_string(k), v} end)
    |> Map.new()
  end

  @doc """
  Validate the envelope, checking that all required CloudEvents attributes
  are present and that `eel_data_class` (if set) is one of A..E.

  Returns `:ok` or `{:error, [field_atoms]}`.
  """
  @spec validate(t()) :: :ok | {:error, [atom()]}
  def validate(%__MODULE__{} = envelope) do
    required = [:specversion, :id, :type, :source]

    missing =
      Enum.filter(required, fn field ->
        value = Map.get(envelope, field)
        is_nil(value) or value == ""
      end)

    data_class_errors =
      if not is_nil(envelope.eel_data_class) and
           envelope.eel_data_class not in ~w(A B C D E) do
        [:eel_data_class]
      else
        []
      end

    case missing ++ data_class_errors do
      [] -> :ok
      errors -> {:error, errors}
    end
  end
end
