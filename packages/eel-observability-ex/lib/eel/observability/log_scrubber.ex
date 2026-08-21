defmodule Eel.Observability.LogScrubber do
  @moduledoc """
  Logger backend (`:gen_event` handler) that scrubs PII from log messages.

  When a log message metadata contains `:pii_class` with value `:a` or `:b`,
  or the message itself contains `[REDACTED`, the message body is replaced
  with a sanitised placeholder before forwarding to the IO device.

  ## Usage

      Eel.Observability.LogScrubber.add_to_logger()
  """

  @behaviour :gen_event

  @scrubbed_a "[LOG SCRUBBED: PII class A/B]"

  # Public API

  @doc """
  Adds this module as a Logger backend handler.
  """
  @spec add_to_logger() :: :ok | {:error, term()}
  def add_to_logger do
    :gen_event.add_handler(Logger, __MODULE__, [])
  end

  # gen_event callbacks

  @impl :gen_event
  def init(_args) do
    {:ok, %{io_device: :user, level: :debug}}
  end

  @impl :gen_event
  def handle_event({level, _gl, {Logger, msg, _ts, md}}, state) do
    scrubbed_msg = maybe_scrub(msg, md)
    formatted = "[#{level}] #{scrubbed_msg}\n"
    IO.write(state.io_device, formatted)
    {:ok, state}
  end

  def handle_event(:flush, state) do
    {:ok, state}
  end

  def handle_event(_event, state) do
    {:ok, state}
  end

  @impl :gen_event
  def handle_call({:configure, opts}, state) do
    new_state =
      state
      |> maybe_update(:io_device, opts)
      |> maybe_update(:level, opts)

    {:ok, :ok, new_state}
  end

  def handle_call(_request, state) do
    {:ok, :ok, state}
  end

  @impl :gen_event
  def handle_info(_msg, state) do
    {:ok, state}
  end

  @impl :gen_event
  def terminate(_reason, _state) do
    :ok
  end

  @impl :gen_event
  def code_change(_old_vsn, state, _extra) do
    {:ok, state}
  end

  # Private helpers

  defp maybe_scrub(msg, md) do
    pii_class = Keyword.get(md, :pii_class)
    msg_str = to_string(msg)

    if pii_class in [:a, :b] or String.contains?(msg_str, "[REDACTED") do
      @scrubbed_a
    else
      msg_str
    end
  end

  defp maybe_update(state, key, opts) do
    case Keyword.fetch(opts, key) do
      {:ok, val} -> Map.put(state, key, val)
      :error -> state
    end
  end
end
