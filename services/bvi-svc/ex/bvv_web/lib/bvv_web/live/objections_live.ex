defmodule BvvWeb.ObjectionsLive do
  @moduledoc """
  BVV objection filing and tracking LiveView.

  Filings persist through the BvvWeb.Objections context against the objections
  table; recent filings are listed below the form and new filings from any
  viewer arrive via the `eel:bvv:objections` PubSub topic.
  """
  use BvvWeb, :live_view

  alias BvvWeb.Objections

  require Logger

  @impl true
  def mount(_params, session, socket) do
    if connected?(socket), do: Objections.subscribe()

    socket =
      socket
      |> assign(:objector_session_hash, objector_session_hash(session))
      |> assign(:changeset, Objections.change_objection())
      |> assign(:objections, Objections.list_objections())

    {:ok, socket}
  end

  @impl true
  def handle_event("file_objection", params, socket) do
    attrs =
      params
      |> Map.take(["target_voter_id", "objection_type", "justification", "evidence_ref"])
      |> Map.put("objector_session_hash", socket.assigns.objector_session_hash)

    case Objections.create_objection(attrs) do
      {:ok, objection} ->
        {:noreply,
         socket
         |> put_flash(:info, "Objection filed. ID: #{objection.obj_id}")
         |> assign(:changeset, Objections.change_objection())}

      {:error, changeset} ->
        Logger.warning("objection filing rejected",
          fields: inspect(Keyword.keys(changeset.errors))
        )

        {:noreply,
         socket
         |> put_flash(:error, "Objection could not be filed. Check the highlighted fields.")
         |> assign(:changeset, changeset)}
    end
  end

  @impl true
  def handle_info({:objection_filed, objection}, socket) do
    objections =
      socket.assigns.objections
      |> Enum.reject(&(&1.id == objection.id))
      |> List.insert_at(0, objection)

    {:noreply, assign(socket, :objections, objections)}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="bvv-objections">
      <h1>File an Objection</h1>
      <%= if msg = @flash["info"] do %>
        <p class="flash flash-info"><%= msg %></p>
      <% end %>
      <%= if msg = @flash["error"] do %>
        <p class="flash flash-error"><%= msg %></p>
      <% end %>
      <form phx-submit="file_objection">
        <%= for {field, {msg, _}} <- @changeset.errors do %>
          <p class="field-error"><%= Phoenix.HTML.Form.humanize(field) %>: <%= translate_error(msg) %></p>
        <% end %>
        <input type="text" name="target_voter_id" placeholder="Target Voter ID" required />
        <select name="objection_type">
          <option value="rectification">Rectification</option>
          <option value="transfer_request">Transfer Request</option>
          <option value="removal_request">Removal Request</option>
        </select>
        <textarea name="justification" placeholder="Justification" required></textarea>
        <input type="text" name="evidence_ref" placeholder="Evidence reference (optional)" />
        <button type="submit">Submit Objection</button>
      </form>

      <h2>Recent Objections</h2>
      <ul class="objection-list">
        <%= for objection <- @objections do %>
          <li>
            <span class={"status status-#{objection.status}"}><%= objection.status %></span>
            <%= objection.objection_type %> on voter <code><%= objection.target_voter_id %></code>
            <time datetime={DateTime.to_iso8601(objection.filed_at)}>
              <%= DateTime.to_string(objection.filed_at) %>
            </time>
          </li>
        <% end %>
      </ul>
      <%= if Enum.empty?(@objections) do %>
        <p>No objections filed yet.</p>
      <% end %>
    </div>
    """
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  # The portal is anonymous: derive a stable pseudonymous hash from a browser
  # session token when one exists, otherwise mint an opaque per-session value.
  # Only the hash reaches the database, never a raw identifier.
  defp objector_session_hash(session) do
    case session["objector_token"] do
      token when is_binary(token) -> sha256(token)
      _ -> Base.encode16(:crypto.strong_rand_bytes(32), case: :lower)
    end
  end

  defp sha256(value), do: Base.encode16(:crypto.hash(:sha256, value), case: :lower)

  defp translate_error(msg) when is_binary(msg), do: msg
  defp translate_error({msg, _opts}) when is_binary(msg), do: msg
end
