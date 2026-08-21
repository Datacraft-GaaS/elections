defmodule ObserverWorkspaceWeb.JointStatementLive do
  use ObserverWorkspaceWeb, :live_view

  @impl true
  def mount(_params, _session, socket) do
    {:ok, assign(socket, :statements, [])}
  end

  @impl true
  def render(assigns) do
    ~H"""
    <div class="mx-auto max-w-2xl p-6">
      <h1 class="text-xl font-bold mb-4">Joint Statements</h1>
      <p class="text-gray-400">No joint statements published yet.</p>
    </div>
    """
  end
end
