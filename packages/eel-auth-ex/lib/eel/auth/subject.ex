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
