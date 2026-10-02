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

defmodule BvvWeb.LookupLive do
  @moduledoc """
  BVV voter register lookup LiveView.

  EC-SEC-10: enumeration defence -- minimum 300 ms response, jittered.
  All responses have identical payload structure regardless of voter existence.

  Backed by bvi-api GET /api/v1/bvi/voters/{voter_id}/station
  (contract: packages/openapi-contracts/bvi/openapi.yaml, getVoterStation);
  every lookup is audit-logged server-side (FR-M07-020).
  """
  use BvvWeb, :live_view

  require Logger

  @min_response_ms 300

  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:result, nil)
     |> assign(:error, nil)
     |> assign(:loading, false)}
  end

  @impl true
  def handle_event("lookup", %{"national_id" => nid, "otp" => _otp}, socket) do
    socket = assign(socket, :loading, true)
    # Enforce minimum 300 ms + jitter to defeat timing oracle (EC-SEC-10).
    start = System.monotonic_time(:millisecond)
    result = perform_lookup(nid)
    elapsed = System.monotonic_time(:millisecond) - start
    jitter = :rand.uniform(50)
    delay = max(0, @min_response_ms + jitter - elapsed)
    if delay > 0, do: Process.sleep(delay)
    {:noreply, assign(socket, loading: false, result: result, error: nil)}
  end

  defp perform_lookup(national_id) do
    election_id = Application.get_env(:bvv_web, :election_id, "elec-2026-general")

    case Req.get("#{api_base()}/api/v1/bvi/voters/#{URI.encode_www_form(national_id)}/station",
           params: [election_id: election_id]
         ) do
      {:ok, %{status: 200, body: %{"polling_station_name" => name} = body}} ->
        %{
          found: true,
          station_name: name,
          station_code: body["station_code"],
          ward: nil,
          constituency: body["constituency_code"],
          county: body["county_code"],
          registration_status: "registered"
        }

      # Not found must be indistinguishable in shape/timing from a hit.
      {:ok, %{status: 404}} ->
        not_found_result()

      other ->
        Logger.warning("voter station lookup failed", response: inspect(other))
        not_found_result()
    end
  rescue
    e ->
      Logger.warning("voter station lookup raised", error: Exception.message(e))
      not_found_result()
  end

  defp not_found_result do
    %{
      found: false,
      station_name: nil,
      station_code: nil,
      ward: nil,
      constituency: nil,
      county: nil,
      registration_status: nil
    }
  end

  defp api_base, do: Application.get_env(:bvv_web, :bvi_api_url, "http://bvi-api:8080")

  @impl true
  def render(assigns) do
    ~H"""
    <div class="bvv-lookup">
      <h1>Voter Register Lookup</h1>
      <form phx-submit="lookup">
        <input type="text" name="national_id" placeholder="National ID" required />
        <input type="text" name="otp" placeholder="OTP" required />
        <button type="submit" disabled={@loading}>
          <%= if @loading, do: "Checking...", else: "Look Up" %>
        </button>
      </form>
      <%= if @result do %>
        <div class="result">
          <%= if @result.found do %>
            <p>Polling Station: <%= @result.station_name %> (<%= @result.station_code %>)</p>
            <p>Constituency: <%= @result.constituency %></p>
            <p>County: <%= @result.county %></p>
            <p>Status: <%= @result.registration_status %></p>
          <% else %>
            <p>No registration found for the provided details.</p>
          <% end %>
        </div>
      <% end %>
    </div>
    """
  end
end
