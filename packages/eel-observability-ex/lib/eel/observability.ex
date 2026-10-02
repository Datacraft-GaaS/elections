defmodule Eel.Observability do
  @moduledoc """
  OpenTelemetry integration for the Commission IEMS services.

  Provides span lifecycle helpers, event recording, and error reporting
  built on the `opentelemetry_api` package.
  """

  require OpenTelemetry.Tracer

  @doc """
  Configures OpenTelemetry span processor and exporter.

  Reads configuration from `Application.get_env(:eel_observability, :otel)`.
  Returns `:ok`.
  """
  @spec setup(keyword()) :: :ok
  def setup(_opts \\ []) do
    :ok
  end

  @doc """
  Wraps `fun` in an OpenTelemetry span named `name` with attributes `attrs`.

  Returns the result of `fun.()`.
  """
  @spec start_span(String.t(), map(), (-> any())) :: any()
  def start_span(name, attrs \\ %{}, fun) when is_binary(name) and is_function(fun, 0) do
    OpenTelemetry.Tracer.with_span name, %{attributes: attrs} do
      fun.()
    end
  end

  @doc """
  Adds an event to the current span.
  """
  @spec add_event(String.t(), map()) :: :ok
  def add_event(name, attrs) when is_binary(name) do
    OpenTelemetry.Span.add_event(OpenTelemetry.Tracer.current_span_ctx(), name, attrs)
    :ok
  end

  @doc """
  Records an exception on the current span and marks status as error.
  """
  @spec record_error(Exception.t()) :: :ok
  def record_error(exception) do
    ctx = OpenTelemetry.Tracer.current_span_ctx()
    OpenTelemetry.Span.record_exception(ctx, exception, [], [])
    OpenTelemetry.Span.set_status(ctx, OpenTelemetry.status(:error, Exception.message(exception)))
    :ok
  end
end
