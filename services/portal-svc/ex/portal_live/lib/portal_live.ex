defmodule PortalLive do
  @moduledoc """
  eel portal-svc: Phoenix LiveView application for real-time election results.

  Streams live aggregate counts from Kafka projections via LiveView.
  Displays Merkle root cross-verified against verifier-rs WASM.
  Provides SSE endpoint for third-party feeds.
  No PII exposed: strictly aggregate counts only (FR-M11-026).
  """

  @version "0.1.0"

  def version, do: @version
end
