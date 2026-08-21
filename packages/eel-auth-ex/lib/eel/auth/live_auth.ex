if Code.ensure_loaded?(Phoenix.LiveView) do
  defmodule Eel.Auth.LiveAuth do
    @moduledoc """
    Phoenix LiveView `on_mount` helper for authentication gates.

    Only compiled when `phoenix_live_view` is available. Declare it as a
    non-optional dep in your host app to enable this module.

    ## Usage

        live_session :authenticated, on_mount: {Eel.Auth.LiveAuth, :require_authenticated} do
          live "/dashboard", DashboardLive
        end
    """

    @doc """
    `on_mount` callback that requires an authenticated subject.

    Checks `socket.assigns[:current_subject]` and `session["user_token"]`.
    Redirects unauthenticated users to `/auth/login` and halts.
    """
    def on_mount(:require_authenticated, _params, session, socket) do
      cond do
        Map.get(socket.assigns, :current_subject) != nil ->
          {:cont, socket}

        Map.get(session, "user_token") != nil ->
          {:cont, socket}

        true ->
          socket = Phoenix.LiveView.redirect(socket, to: "/auth/login")
          {:halt, socket}
      end
    end
  end
end
