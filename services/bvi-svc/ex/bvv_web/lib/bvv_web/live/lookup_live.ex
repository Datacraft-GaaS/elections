defmodule BvvWeb.LookupLive do
  @moduledoc """
  BVV voter register lookup LiveView.

  EC-SEC-10: enumeration defence -- minimum 300 ms response, jittered.
  All responses have identical payload structure regardless of voter existence.
  """
  use BvvWeb, :live_view

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
  def handle_event("lookup", %{"national_id" => nid, "otp" => otp}, socket) do
    socket = assign(socket, :loading, true)
    # Enforce minimum 300 ms + jitter to defeat timing oracle (EC-SEC-10).
    start = System.monotonic_time(:millisecond)
    result = perform_lookup(nid, otp)
    elapsed = System.monotonic_time(:millisecond) - start
    jitter = :rand.uniform(50)
    delay = max(0, @min_response_ms + jitter - elapsed)
    if delay > 0, do: Process.sleep(delay)
    {:noreply, assign(socket, loading: false, result: result, error: nil)}
  end

  defp perform_lookup(_nid, _otp) do
    # TODO: call bvi-api POST /api/v1/bvv/lookup via Req
    %{
      name_display: "J. Doe",
      dob_year: 1985,
      station_name: "Sample Polling Station",
      station_code: "001",
      ward: "Sample Ward",
      constituency: "Sample Constituency",
      county: "Nairobi",
      registration_status: "active"
    }
  end

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
          <p>Name: <%= @result.name_display %></p>
          <p>Year of Birth: <%= @result.dob_year %></p>
          <p>Station: <%= @result.station_name %> (<%= @result.station_code %>)</p>
          <p>Ward: <%= @result.ward %></p>
          <p>Constituency: <%= @result.constituency %></p>
          <p>County: <%= @result.county %></p>
          <p>Status: <%= @result.registration_status %></p>
        </div>
      <% end %>
    </div>
    """
  end
end
