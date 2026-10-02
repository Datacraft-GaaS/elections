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

defmodule Eel.Hlc.Clock do
  @moduledoc """
  GenServer that owns a single HLC state and advances it monotonically.

  Each call to `now/1` reads the OS wall clock and ticks the HLC forward.
  Each call to `observe/2` merges an externally received HLC into the state.
  """

  use GenServer

  alias Eel.Hlc

  # --- Public API ---

  @doc """
  Start the clock process.

  Options:
    - `node:` (required) string identifier for this node
    - `name:` optional GenServer name
  """
  @spec start_link(keyword()) :: GenServer.on_start()
  def start_link(opts) do
    node = Keyword.fetch!(opts, :node)
    name = Keyword.get(opts, :name, __MODULE__)
    GenServer.start_link(__MODULE__, node, name: name)
  end

  @doc "Return the current HLC, advancing the clock with the current wall time."
  @spec now(GenServer.server()) :: Hlc.t()
  def now(server \\ __MODULE__) do
    GenServer.call(server, :now)
  end

  @doc "Merge a received HLC into this clock's state."
  @spec observe(Hlc.t(), GenServer.server()) :: :ok
  def observe(%Hlc{} = received, server \\ __MODULE__) do
    GenServer.call(server, {:observe, received})
  end

  # --- GenServer callbacks ---

  @impl true
  def init(node) when is_binary(node) do
    wall = :erlang.system_time(:millisecond)
    hlc = Hlc.new(wall, 0, node)
    {:ok, hlc}
  end

  @impl true
  def handle_call(:now, _from, hlc) do
    wall = :erlang.system_time(:millisecond)
    next = Hlc.tick(hlc, wall)
    {:reply, next, next}
  end

  @impl true
  def handle_call({:observe, received}, _from, hlc) do
    next = Hlc.merge(hlc, received)
    {:reply, :ok, next}
  end
end
