defmodule Eel.Events.Consumer do
  @moduledoc """
  Behaviour for consuming `Eel.Events.Envelope` messages from a Broadway
  pipeline or any other delivery mechanism.

  ## Example

      defmodule MyApp.EventConsumer do
        @behaviour Eel.Events.Consumer

        @impl Eel.Events.Consumer
        def handle_event(%Eel.Events.Envelope{type: type} = envelope, meta) do
          IO.puts("received \#{type} with correlation \#{envelope.eel_correlation_id}")
          :ok
        end
      end
  """

  alias Eel.Events.Envelope

  @doc """
  Handle a single CloudEvent envelope.

  `meta` contains delivery metadata (e.g. Kafka offset, Broadway message
  metadata) and is consumer-implementation-specific.

  Return `:ok` on success or `{:error, term()}` to signal failure. The
  pipeline is responsible for retry / dead-lettering on error.
  """
  @callback handle_event(envelope :: Envelope.t(), meta :: map()) ::
              :ok | {:error, term()}
end
