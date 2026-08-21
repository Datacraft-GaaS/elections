defmodule ObserverWorkspaceWeb.CoreComponents do
  @moduledoc """
  Minimal core UI components for EEL observer workspace.
  """
  use Phoenix.Component

  @doc """
  Renders flash notices.
  """
  attr :flash, :map, required: true

  def flash_group(assigns) do
    ~H"""
    <div aria-live="polite" role="alert">
      <%= if msg = @flash["info"] do %>
        <div class="bg-blue-100 text-blue-800 px-4 py-2 rounded mb-2"><%= msg %></div>
      <% end %>
      <%= if msg = @flash["error"] do %>
        <div class="bg-red-100 text-red-800 px-4 py-2 rounded mb-2"><%= msg %></div>
      <% end %>
    </div>
    """
  end
end
