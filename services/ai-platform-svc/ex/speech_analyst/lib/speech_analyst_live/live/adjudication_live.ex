defmodule SpeechAnalystLive.Live.AdjudicationLive do
  @moduledoc "Analyst adjudication workflow: confirm or reject pending flags."
  use SpeechAnalystLive, :live_view

  @speech_integrity_base Application.compile_env(
                            :speech_analyst_live,
                            :speech_integrity_url,
                            "http://localhost:8000"
                          )

  @impl true
  def mount(_params, _session, socket) do
    {:ok,
     socket
     |> assign(:current_flag, nil)
     |> assign(:notes, "")
     |> assign(:analyst_id, "analyst@eel")
     |> assign(:submitted, false)
     |> assign(:error, nil)}
  end

  @impl true
  def handle_event("load_next", _params, socket) do
    case fetch_pending_flag() do
      {:ok, flag} -> {:noreply, assign(socket, current_flag: flag, submitted: false, error: nil)}
      {:error, msg} -> {:noreply, assign(socket, error: msg)}
    end
  end

  @impl true
  def handle_event("update_notes", %{"notes" => notes}, socket) do
    {:noreply, assign(socket, :notes, notes)}
  end

  @impl true
  def handle_event("decide", %{"decision" => decision}, socket) do
    flag = socket.assigns.current_flag

    case submit_decision(flag["flag_id"], decision, socket.assigns.analyst_id, socket.assigns.notes) do
      :ok ->
        {:noreply, assign(socket, submitted: true, current_flag: nil, notes: "")}

      {:error, msg} ->
        {:noreply, assign(socket, error: msg)}
    end
  end

  defp fetch_pending_flag do
    case Req.get(@speech_integrity_base <> "/v1/speech/flags") do
      {:ok, %{status: 200, body: flags}} ->
        pending = Enum.find(flags, fn f -> f["analyst_decision"] == "pending" end)

        if pending do
          {:ok, pending}
        else
          {:error, "no pending flags"}
        end

      {:ok, resp} ->
        {:error, "unexpected status #{resp.status}"}

      {:error, reason} ->
        {:error, inspect(reason)}
    end
  end

  defp submit_decision(flag_id, decision, analyst_id, notes) do
    body = %{flag_id: flag_id, decision: decision, decision_by: analyst_id, notes: notes}
    url = @speech_integrity_base <> "/v1/speech/flags/#{flag_id}/decision"

    case Req.patch(url, json: body) do
      {:ok, %{status: 200}} -> :ok
      {:ok, resp} -> {:error, "api error #{resp.status}"}
      {:error, reason} -> {:error, inspect(reason)}
    end
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="adjudication">
      <h1>Adjudication</h1>

      <%= if @error do %>
        <p class="error"><%= @error %></p>
      <% end %>

      <%= if @submitted do %>
        <p class="success">Decision submitted.</p>
      <% end %>

      <%= if @current_flag do %>
        <div class="flag-card">
          <p><strong>Flag ID:</strong> <%= @current_flag["flag_id"] %></p>
          <p><strong>Item ID:</strong> <%= @current_flag["item_id"] %></p>
          <p><strong>Type:</strong> <%= @current_flag["flag_type"] %></p>
          <p><strong>Confidence:</strong> <%= @current_flag["confidence"] %></p>
          <textarea phx-change="update_notes" name="notes" placeholder="Notes..."><%= @notes %></textarea>
          <div class="actions">
            <button phx-click="decide" phx-value-decision="confirmed">Confirm</button>
            <button phx-click="decide" phx-value-decision="rejected">Reject</button>
          </div>
        </div>
      <% else %>
        <button phx-click="load_next">Load Next Pending Flag</button>
      <% end %>
    </div>
    """
  end
end
