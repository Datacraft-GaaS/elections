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

defmodule Eel.Resilience.DegradationFlag do
  @moduledoc """
  Stores the current system degradation mode in `:persistent_term`.

  Modes: `:normal`, `:degraded`, `:read_only`.
  """

  @key {__MODULE__, :degradation_mode}

  @type mode :: :normal | :degraded | :read_only

  @doc """
  Sets the degradation mode. Must be `:normal`, `:degraded`, or `:read_only`.
  """
  @spec set(mode()) :: :ok
  def set(mode) when mode in [:normal, :degraded, :read_only] do
    :persistent_term.put(@key, mode)
    :ok
  end

  @doc """
  Returns the current degradation mode. Defaults to `:normal` if not set.
  """
  @spec get() :: mode()
  def get do
    :persistent_term.get(@key, :normal)
  end

  @doc """
  Returns `true` if the system is in `:degraded` or `:read_only` mode.
  """
  @spec degraded?() :: boolean()
  def degraded? do
    get() != :normal
  end

  @doc """
  Returns `true` if the system is in `:read_only` mode.
  """
  @spec read_only?() :: boolean()
  def read_only? do
    get() == :read_only
  end
end
