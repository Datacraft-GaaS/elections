# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
