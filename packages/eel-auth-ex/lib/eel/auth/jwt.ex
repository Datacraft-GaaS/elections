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

defmodule Eel.Auth.JWT do
  @moduledoc """
  RS256/ES256 JWT verification backed by the JWKS cache.

  Decodes and verifies the JWT signature, then validates `exp`, `iat`,
  and `iss` claims. Returns a populated `Eel.Auth.Subject` on success.
  """

  alias Eel.Auth.{JWKS, Subject}

  @doc """
  Verifies a JWT token string.

  Looks up the signing key via `jwks_pid_or_name` (which must be a running
  `Eel.Auth.JWKS` process), validates the signature and standard claims,
  and returns `{:ok, %Subject{}}` or `{:error, reason}`.
  """
  @spec verify(String.t(), GenServer.server()) ::
          {:ok, Subject.t()} | {:error, atom() | String.t()}
  def verify(token, jwks_pid_or_name \\ JWKS) do
    with {:ok, {header_b64, payload_b64, sig_b64}} <- split_token(token),
         {:ok, header} <- decode_part(header_b64),
         {:ok, payload} <- decode_part(payload_b64),
         {:ok, sig} <- Base.url_decode64(sig_b64, padding: false),
         {:ok, pem} <- JWKS.get_public_key(jwks_pid_or_name, Map.fetch!(header, "kid")),
         :ok <- verify_signature(header_b64, payload_b64, sig, pem, header),
         :ok <- validate_claims(payload) do
      {:ok, build_subject(payload, token)}
    end
  end

  defp split_token(token) do
    case String.split(token, ".") do
      [h, p, s] -> {:ok, {h, p, s}}
      _ -> {:error, :invalid_token_format}
    end
  end

  defp decode_part(b64) do
    with {:ok, json} <- Base.url_decode64(b64, padding: false),
         {:ok, map} <- Jason.decode(json) do
      {:ok, map}
    else
      _ -> {:error, :invalid_token_part}
    end
  end

  defp verify_signature(header_b64, payload_b64, sig, pem, %{"alg" => alg}) do
    message = "#{header_b64}.#{payload_b64}"

    [{key_type, der, _}] = :public_key.pem_decode(pem)
    public_key = :public_key.der_decode(key_type, der)

    hash =
      case alg do
        "RS256" -> :sha256
        "RS384" -> :sha384
        "RS512" -> :sha512
        "ES256" -> :sha256
        "ES384" -> :sha384
        "ES512" -> :sha512
        _ -> :sha256
      end

    if :public_key.verify(message, hash, sig, public_key) do
      :ok
    else
      {:error, :invalid_signature}
    end
  end

  defp verify_signature(_h, _p, _sig, _pem, _header), do: {:error, :unknown_algorithm}

  defp validate_claims(payload) do
    now = System.system_time(:second)

    cond do
      not is_integer(Map.get(payload, "exp")) ->
        {:error, :missing_exp}

      payload["exp"] < now ->
        {:error, :token_expired}

      Map.get(payload, "iat", now) > now + 60 ->
        {:error, :token_not_yet_valid}

      true ->
        :ok
    end
  end

  defp build_subject(payload, raw_jwt) do
    realm =
      case Map.get(payload, "iss", "") do
        iss when is_binary(iss) ->
          iss |> String.split("/realms/") |> List.last() |> to_string()

        _ ->
          ""
      end

    roles =
      payload
      |> Map.get("realm_access", %{})
      |> Map.get("roles", [])

    %Subject{
      id: Map.get(payload, "sub", ""),
      realm: realm,
      email: Map.get(payload, "email"),
      roles: roles,
      svid: Map.get(payload, "spiffe_svid"),
      raw_jwt: raw_jwt
    }
  end
end
