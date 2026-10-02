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

defmodule Eel.Auth.JWKS do
  @moduledoc """
  GenServer that maintains an ETS-backed JWKS key cache with TTL.

  Fetches public keys from Keycloak's well-known JWKS endpoint and
  caches them in `:eel_auth_jwks_cache` for fast lookup.
  """

  use GenServer

  @table :eel_auth_jwks_cache
  @default_ttl_ms 300_000

  @type opts :: [
          keycloak_url: String.t(),
          realm: String.t(),
          ttl_ms: non_neg_integer()
        ]

  # Client API

  @doc """
  Starts the JWKS cache GenServer.

  Options:
  - `keycloak_url` - base URL of the Keycloak server, e.g. `"https://auth.example.com"`
  - `realm` - Keycloak realm name
  - `ttl_ms` - key cache TTL in milliseconds (default: 300_000)
  """
  @spec start_link(opts()) :: GenServer.on_start()
  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc """
  Looks up a public key by `kid`. Fetches from Keycloak if missing or expired.

  Returns `{:ok, public_key_pem}` or `{:error, :not_found}`.
  """
  @spec get_public_key(GenServer.server(), String.t()) :: {:ok, String.t()} | {:error, :not_found}
  def get_public_key(server \\ __MODULE__, kid) do
    GenServer.call(server, {:get_public_key, kid})
  end

  # Server callbacks

  @impl true
  def init(opts) do
    case :ets.info(@table) do
      :undefined ->
        :ets.new(@table, [:named_table, :set, :public, read_concurrency: true])

      _ ->
        @table
    end

    state = %{
      keycloak_url: Keyword.fetch!(opts, :keycloak_url),
      realm: Keyword.fetch!(opts, :realm),
      ttl_ms: Keyword.get(opts, :ttl_ms, @default_ttl_ms)
    }

    {:ok, state}
  end

  @impl true
  def handle_call({:get_public_key, kid}, _from, state) do
    now = System.monotonic_time(:millisecond)

    result =
      case :ets.lookup(@table, kid) do
        [{^kid, pem, expires_at}] when expires_at > now ->
          {:ok, pem}

        _ ->
          fetch_and_cache(kid, state, now)
      end

    {:reply, result, state}
  end

  defp fetch_and_cache(kid, state, now) do
    url = "#{state.keycloak_url}/realms/#{state.realm}/protocol/openid-connect/certs"

    case Finch.build(:get, url) |> Finch.request(Eel.Auth.Finch) do
      {:ok, %Finch.Response{status: 200, body: body}} ->
        case Jason.decode(body) do
          {:ok, %{"keys" => keys}} ->
            Enum.each(keys, fn key ->
              key_kid = Map.get(key, "kid")
              pem = jwk_to_pem(key)
              expires_at = now + state.ttl_ms
              :ets.insert(@table, {key_kid, pem, expires_at})
            end)

            case :ets.lookup(@table, kid) do
              [{^kid, pem, _}] -> {:ok, pem}
              _ -> {:error, :not_found}
            end

          _ ->
            {:error, :not_found}
        end

      _ ->
        {:error, :not_found}
    end
  end

  defp jwk_to_pem(%{"kty" => "RSA", "n" => n, "e" => e}) do
    n_bin = Base.url_decode64!(n, padding: false)
    e_bin = Base.url_decode64!(e, padding: false)

    public_key = {:RSAPublicKey, :crypto.bytes_to_integer(n_bin), :crypto.bytes_to_integer(e_bin)}
    der = :public_key.der_encode(:RSAPublicKey, public_key)
    :public_key.pem_encode([{:RSAPublicKey, der, :not_encrypted}])
  end

  defp jwk_to_pem(key) do
    Jason.encode!(key)
  end
end
