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

defmodule Eel.Auth do
  @moduledoc """
  Authentication and authorisation library for the Commission IEMS services.

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
