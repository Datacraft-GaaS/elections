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

defmodule Eel.Auth.SpiceDB do
  @moduledoc """
  Client for the SpiceDB gRPC-gateway JSON API.

  Calls the permissions check and resource lookup endpoints.
  Configure via `Application.put_env(:eel_auth, :spicedb_url, ...)` and
  `Application.put_env(:eel_auth, :spicedb_token, ...)`.
  """

  @default_url "http://localhost:50051"

  @type consistency :: :full | :at_least_as_fresh | :minimize_latency

  @doc """
  Checks whether `subject_id` has `permission` on the given resource.

  Returns `{:ok, %{permissionship: String.t()}}` or `{:error, reason}`.
  """
  @spec check_permission(String.t(), String.t(), String.t(), String.t(), keyword()) ::
          {:ok, map()} | {:error, term()}
  def check_permission(resource_type, resource_id, permission, subject_id, opts \\ []) do
    body = %{
      resource: %{objectType: resource_type, objectId: resource_id},
      permission: permission,
      subject: %{object: %{objectType: "user", objectId: subject_id}},
      consistency: consistency_map(Keyword.get(opts, :consistency, :at_least_as_fresh))
    }

    post("/v1/permissions/check", body)
  end

  @doc """
  Looks up resources of `resource_type` where `subject_id` has `permission`.

  Returns `{:ok, result}` or `{:error, reason}`.
  """
  @spec lookup_resources(String.t(), String.t(), String.t(), keyword()) ::
          {:ok, map()} | {:error, term()}
  def lookup_resources(resource_type, permission, subject_id, opts \\ []) do
    body = %{
      resourceObjectType: resource_type,
      permission: permission,
      subject: %{object: %{objectType: "user", objectId: subject_id}},
      consistency: consistency_map(Keyword.get(opts, :consistency, :at_least_as_fresh))
    }

    post("/v1/permissions/resources", body)
  end

  defp post(path, body) do
    base_url = Application.get_env(:eel_auth, :spicedb_url, @default_url)
    token = Application.get_env(:eel_auth, :spicedb_token, "")
    url = base_url <> path

    headers = [
      {"content-type", "application/json"},
      {"authorization", "Bearer #{token}"}
    ]

    case Jason.encode(body) do
      {:ok, json} ->
        request = Finch.build(:post, url, headers, json)

        case Finch.request(request, Eel.Auth.Finch) do
          {:ok, %Finch.Response{status: status, body: resp_body}} when status in 200..299 ->
            Jason.decode(resp_body)

          {:ok, %Finch.Response{status: status, body: resp_body}} ->
            {:error, {:http_error, status, resp_body}}

          {:error, reason} ->
            {:error, reason}
        end

      {:error, reason} ->
        {:error, {:encode_error, reason}}
    end
  end

  defp consistency_map(:full), do: %{fullyConsistent: true}
  defp consistency_map(:minimize_latency), do: %{minimizeLatency: true}
  defp consistency_map(:at_least_as_fresh), do: %{atLeastAsFresh: %{}}
  defp consistency_map(_), do: %{atLeastAsFresh: %{}}
end
