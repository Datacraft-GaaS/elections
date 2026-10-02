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

defmodule Eel.Auth.Subject do
  @moduledoc """
  Represents an authenticated principal, either a human user or a workload.

  Built from a verified JWT. `svid` is populated for workload identities
  carrying a SPIFFE SVID.
  """

  @enforce_keys [:id, :realm, :roles, :raw_jwt]
  defstruct [
    :id,
    :realm,
    :email,
    :svid,
    :raw_jwt,
    roles: []
  ]

  @type t :: %__MODULE__{
          id: String.t(),
          realm: String.t(),
          email: String.t() | nil,
          roles: [String.t()],
          svid: String.t() | nil,
          raw_jwt: String.t()
        }
end
