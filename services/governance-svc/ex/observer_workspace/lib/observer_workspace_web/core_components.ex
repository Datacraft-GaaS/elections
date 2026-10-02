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
