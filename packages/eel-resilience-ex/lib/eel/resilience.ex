defmodule Eel.Resilience do
  @moduledoc """
  Top-level module for the Eel.Resilience package.

  Provides circuit breaking, retry with jitter, bulkhead isolation,
  timeout wrappers, idempotency, and degradation flags.
  """

  @doc """
  Returns the package version.
  """
  def version, do: "0.1.0"
end
