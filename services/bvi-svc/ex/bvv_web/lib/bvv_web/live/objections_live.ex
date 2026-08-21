defmodule BvvWeb.ObjectionsLive do
  @moduledoc "BVV objection filing and tracking LiveView."
  use BvvWeb, :live_view

  @impl true
  def mount(_params, _session, socket) do
    {:ok, assign(socket, :objections, [])}
  end

  @impl true
  def handle_event("file_objection", params, socket) do
    # TODO: call bvi-api POST /api/v1/bvv/objections
    {:noreply, put_flash(socket, :info, "Objection filed. ID: #{Ecto.UUID.generate()}")}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="bvv-objections">
      <h1>File an Objection</h1>
      <form phx-submit="file_objection">
        <input type="text" name="target_voter_id" placeholder="Target Voter ID" required />
        <select name="objection_type">
          <option value="rectification">Rectification</option>
          <option value="transfer_request">Transfer Request</option>
          <option value="removal_request">Removal Request</option>
        </select>
        <textarea name="justification" placeholder="Justification" required></textarea>
        <button type="submit">Submit Objection</button>
      </form>
    </div>
    """
  end
end
