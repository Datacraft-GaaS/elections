defmodule Eel.Secrets.JITAdminCred do
  @moduledoc """
  Just-in-time short-lived admin credential from OpenBao.

  Fetches a dynamic credential bound to a specific role with a caller-specified
  TTL, suitable for break-glass admin operations.
  """

  @doc """
  Fetches a short-lived credential for `role` with the given TTL.

  `ttl_hours` is converted to a Vault-style duration string (e.g. `"4h"`).
  Returns `{:ok, cred_map}` or `{:error, reason}`.
  """
  @spec fetch(String.t(), non_neg_integer()) :: {:ok, map()} | {:error, term()}
  def fetch(role, ttl_hours \\ 4) do
    path = "database/creds/#{role}"
    url = "#{bao_url()}/v1/#{path}"
    ttl = "#{ttl_hours}h"

    headers = [
      {"x-vault-token", token()},
      {"accept", "application/json"},
      {"x-vault-wrap-ttl", ttl}
    ]

    request = Finch.build(:get, url, headers)

    case Finch.request(request, Eel.Secrets.Finch) do
      {:ok, %Finch.Response{status: 200, body: body}} ->
        case Jason.decode(body) do
          {:ok, %{"data" => data}} -> {:ok, data}
          {:ok, other} -> {:ok, other}
          {:error, reason} -> {:error, reason}
        end

      {:ok, %Finch.Response{status: 404}} ->
        {:error, :not_found}

      {:ok, %Finch.Response{status: status, body: body}} ->
        {:error, {:http_error, status, body}}

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp bao_url, do: Application.get_env(:eel_secrets, :bao_url, "http://localhost:8200")
  defp token, do: Application.get_env(:eel_secrets, :token, "")
end
