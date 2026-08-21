defmodule Eel.Auth do
  @moduledoc """
  Authentication and authorisation library for IEBC IEMS services.

  Provides Keycloak JWT verification, SpiceDB permission checks, a Plug
  for Phoenix/Cowboy pipelines, and a LiveView `on_mount` guard.

  ## Quick start

      # In your application supervisor:
      {Finch, name: Eel.Auth.Finch},
      {Eel.Auth.JWKS, keycloak_url: "https://auth.example.com", realm: "iems"}

      # In your Phoenix router pipeline:
      plug Eel.Auth.Plug, jwks: Eel.Auth.JWKS

      # In a live_session:
      on_mount: {Eel.Auth.LiveAuth, :require_authenticated}
  """
end
