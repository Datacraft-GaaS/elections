defmodule Eel.Events.Builder do
  @moduledoc """
  Fluent builder for `Eel.Events.Envelope` structs.

  ## Example

      {:ok, envelope} =
        Eel.Events.Builder.new("eel.rts.form34a_captured.v1", "spiffe://eel.local/rts", %{})
        |> Eel.Events.Builder.with_election("2027-general")
        |> Eel.Events.Builder.with_data_class("B")
        |> Eel.Events.Builder.build()
  """

  import Bitwise

  alias Eel.Events.Envelope

  @type t :: %__MODULE__{
          type: String.t(),
          source: String.t(),
          data: term(),
          subject: String.t() | nil,
          eel_stream_id: String.t() | nil,
          eel_seq: integer() | nil,
          eel_prev_hash: String.t() | nil,
          eel_correlation_id: String.t() | nil,
          eel_causation_id: String.t() | nil,
          eel_election_id: String.t() | nil,
          eel_data_class: String.t() | nil,
          eel_hlc: String.t() | nil
        }

  defstruct [
    :type,
    :source,
    :data,
    :subject,
    :eel_stream_id,
    :eel_seq,
    :eel_prev_hash,
    :eel_correlation_id,
    :eel_causation_id,
    :eel_election_id,
    :eel_data_class,
    :eel_hlc
  ]

  @doc """
  Start a new builder with required CloudEvents fields. Generates a random
  UUIDv4 `id` and sets `specversion` to `"1.0"`.
  """
  @spec new(String.t(), String.t(), term()) :: t()
  def new(type, source, data) when is_binary(type) and is_binary(source) do
    %__MODULE__{type: type, source: source, data: data}
  end

  @doc "Set the CloudEvents `subject` attribute."
  @spec with_subject(t(), String.t()) :: t()
  def with_subject(%__MODULE__{} = b, subject), do: %{b | subject: subject}

  @doc "Set the stream tracing fields: `eel_stream_id`, `eel_seq`, `eel_prev_hash`."
  @spec with_stream(t(), String.t(), integer(), String.t()) :: t()
  def with_stream(%__MODULE__{} = b, stream_id, seq, prev_hash) do
    %{b | eel_stream_id: stream_id, eel_seq: seq, eel_prev_hash: prev_hash}
  end

  @doc "Set `eel_correlation_id`."
  @spec with_correlation(t(), String.t()) :: t()
  def with_correlation(%__MODULE__{} = b, correlation_id),
    do: %{b | eel_correlation_id: correlation_id}

  @doc "Set `eel_causation_id`."
  @spec with_causation(t(), String.t()) :: t()
  def with_causation(%__MODULE__{} = b, causation_id),
    do: %{b | eel_causation_id: causation_id}

  @doc "Set `eel_election_id`."
  @spec with_election(t(), String.t()) :: t()
  def with_election(%__MODULE__{} = b, election_id),
    do: %{b | eel_election_id: election_id}

  @doc ~S(Set `eel_data_class`. Must be one of "A".."E".)
  @spec with_data_class(t(), String.t()) :: t()
  def with_data_class(%__MODULE__{} = b, class) when class in ~w(A B C D E),
    do: %{b | eel_data_class: class}

  @doc "Set `eel_hlc` (hybrid logical clock timestamp string)."
  @spec with_hlc(t(), String.t()) :: t()
  def with_hlc(%__MODULE__{} = b, hlc_string), do: %{b | eel_hlc: hlc_string}

  @doc """
  Materialise the builder into an `Eel.Events.Envelope`.

  Returns `{:ok, %Eel.Events.Envelope{}}` or `{:error, reason}`.
  """
  @spec build(t()) :: {:ok, Envelope.t()} | {:error, term()}
  def build(%__MODULE__{} = b) do
    envelope = %Envelope{
      specversion: "1.0",
      id: generate_uuid(),
      type: b.type,
      source: b.source,
      data: b.data,
      subject: b.subject,
      time: DateTime.to_iso8601(DateTime.utc_now()),
      datacontenttype: "application/json",
      eel_stream_id: b.eel_stream_id,
      eel_seq: b.eel_seq,
      eel_hlc: b.eel_hlc,
      eel_prev_hash: b.eel_prev_hash,
      eel_correlation_id: b.eel_correlation_id,
      eel_causation_id: b.eel_causation_id,
      eel_election_id: b.eel_election_id,
      eel_data_class: b.eel_data_class
    }

    case Envelope.validate(envelope) do
      :ok -> {:ok, envelope}
      {:error, errors} -> {:error, {:validation_failed, errors}}
    end
  end

  defp generate_uuid do
    <<a::32, b::16, c::16, d::16, e::48>> = :crypto.strong_rand_bytes(16)
    c4 = (c &&& 0x0FFF) ||| 0x4000
    d4 = (d &&& 0x3FFF) ||| 0x8000

    :io_lib.format(
      "~8.16.0b-~4.16.0b-~4.16.0b-~4.16.0b-~12.16.0b",
      [a, b, c4, d4, e]
    )
    |> IO.iodata_to_binary()
  end
end
