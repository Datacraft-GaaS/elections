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

defmodule EccLive.Live.S44aLive do
  @moduledoc "Section 44A dual-officer verification flow."
  use EccLive, :live_view

  require Logger

  # Dual-officer states: :idle, :officer1_confirmed, :complete
  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:step, :idle)
     |> assign(:station_code, nil)
     |> assign(:officer1_id, nil)
     |> assign(:error, nil)}
  end

  @impl true
  def handle_event("officer1_confirm", %{"officer_id" => oid, "station_code" => sc} = params, socket) do
    Logger.info("s44a officer1 confirmed", officer_id: oid, station_code: sc)

    body = %{
      county_code: Map.get(params, "county_code", ""),
      station_code: sc,
      officer_id: oid
    }

    case Req.post("#{api_base()}/ecc/s44a/officer1", json: body) do
      {:ok, %{status: 201}} ->
        Logger.info("s44a officer1 recorded", station_code: sc)

        {:noreply,
         socket
         |> assign(:step, :officer1_confirmed)
         |> assign(:station_code, sc)
         |> assign(:officer1_id, oid)
         |> assign(:error, nil)}

      {:ok, %{status: status}} ->
        Logger.warning("s44a officer1 rejected", station_code: sc, status: status)
        {:noreply, assign(socket, :error, "Officer 1 confirmation rejected (HTTP #{status}).")}

      {:error, reason} ->
        Logger.warning("s44a officer1 request failed", error: inspect(reason))
        {:noreply, assign(socket, :error, "Could not reach ECC API. Try again.")}
    end
  end

  @impl true
  def handle_event("officer2_confirm", %{"officer_id" => oid}, socket) do
    Logger.info("s44a officer2 confirmed", officer_id: oid, station_code: socket.assigns.station_code)

    body = %{station_code: socket.assigns.station_code, officer_id: oid}

    case Req.post("#{api_base()}/ecc/s44a/officer2", json: body) do
      {:ok, %{status: 200, body: %{"confirmation_id" => id}}} ->
        Logger.info("s44a verification complete", station_code: socket.assigns.station_code, confirmation_id: id)
        {:noreply,
         socket
         |> assign(:step, :complete)
         |> assign(:error, nil)}

      {:ok, %{status: status}} ->
        Logger.warning("s44a officer2 rejected", station_code: socket.assigns.station_code, status: status)
        {:noreply, assign(socket, :error, "Officer 2 confirmation rejected (HTTP #{status}).")}

      {:error, reason} ->
        Logger.warning("s44a officer2 request failed", error: inspect(reason))
        {:noreply, assign(socket, :error, "Could not reach ECC API. Try again.")}
    end
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="ecc-s44a">
      <h1>Section 44A Dual-Officer Verification</h1>
      <%= if @error do %>
        <p class="error"><%= @error %></p>
      <% end %>
      <%= case @step do %>
        <% :idle -> %>
          <form phx-submit="officer1_confirm">
            <input type="text" name="county_code" placeholder="County Code" required />
            <input type="text" name="station_code" placeholder="Station Code" required />
            <input type="text" name="officer_id" placeholder="Officer 1 ID" required />
            <button type="submit">Confirm Officer 1</button>
          </form>
        <% :officer1_confirmed -> %>
          <p>Officer 1 confirmed for station <%= @station_code %>. Awaiting Officer 2.</p>
          <form phx-submit="officer2_confirm">
            <input type="text" name="officer_id" placeholder="Officer 2 ID" required />
            <button type="submit">Confirm Officer 2</button>
          </form>
        <% :complete -> %>
          <p class="success">Dual-officer verification complete for station <%= @station_code %>.</p>
      <% end %>
    </div>
    """
  end

  defp api_base, do: Application.get_env(:ecc_live, :ecc_api_url, "http://ecc-api:8080")
end
