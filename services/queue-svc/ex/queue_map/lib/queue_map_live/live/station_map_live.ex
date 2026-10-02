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

defmodule QueueMapLive.Live.StationMapLive do
  @moduledoc "Public queue map: polling station wait times by urgency."
  use QueueMapLive, :live_view

  require Logger

  # Urgency thresholds in minutes
  @green_max 15
  @amber_max 30

  @impl true
  def mount(_params, _session, socket) do
    if connected?(socket) do
      Phoenix.PubSub.subscribe(QueueMapLive.PubSub, "queue:updates")
    end

    {:ok,
     socket
     |> assign(:stations, sample_stations())}
  end

  @impl true
  def handle_info({:queue_update, payload}, socket) do
    Logger.info("queue update received", event: "queue_update", station: payload[:station_code])

    updated =
      Enum.map(socket.assigns.stations, fn station ->
        if station.code == payload[:station_code] do
          Map.merge(station, %{wait_minutes: payload[:wait_minutes], headcount: payload[:headcount]})
        else
          station
        end
      end)

    {:noreply, assign(socket, :stations, updated)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="eel-queue-map">
      <h1>Polling Station Queue Status</h1>
      <div class="station-list">
        <%= for station <- @stations do %>
          <div class={"station-card urgency-#{urgency_class(station.wait_minutes)}"}>
            <div class="station-name"><%= station.name %></div>
            <div class="station-code"><%= station.code %></div>
            <div class="station-wait">
              Wait: <strong><%= station.wait_minutes %> min</strong>
            </div>
            <div class="station-headcount">
              Queue: <strong><%= station.headcount %></strong> people
            </div>
            <div class={"urgency-badge urgency-#{urgency_class(station.wait_minutes)}"}>
              <%= urgency_label(station.wait_minutes) %>
            </div>
          </div>
        <% end %>
      </div>
    </div>
    """
  end

  defp urgency_class(wait) when wait < @green_max, do: "green"
  defp urgency_class(wait) when wait < @amber_max, do: "amber"
  defp urgency_class(_), do: "red"

  defp urgency_label(wait) when wait < @green_max, do: "Low"
  defp urgency_label(wait) when wait < @amber_max, do: "Moderate"
  defp urgency_label(_), do: "High"

  defp sample_stations do
    [
      %{code: "KEN-001", name: "Kenyatta Primary School", wait_minutes: 8, headcount: 24},
      %{code: "KEN-002", name: "Nairobi West Community Hall", wait_minutes: 22, headcount: 67},
      %{code: "KEN-003", name: "Westlands Sports Ground", wait_minutes: 35, headcount: 112}
    ]
  end
end
