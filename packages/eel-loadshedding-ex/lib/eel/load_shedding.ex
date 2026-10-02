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

defmodule Eel.LoadShedding do
  @moduledoc """
  Load shedding for the Commission IEMS services.

  Provides a CoDel-inspired Plug and Controller that adaptively shed
  low-priority requests when latency exceeds acceptable thresholds.
  """

  @doc """
  Returns the supervisor child spec for load shedding infrastructure.

  Starts `Eel.LoadShedding.Controller` under a supervisor.
  """
  @spec child_spec(keyword()) :: Supervisor.child_spec()
  def child_spec(opts) do
    %{
      id: __MODULE__,
      start: {Supervisor, :start_link, [[{Eel.LoadShedding.Controller, opts}], [strategy: :one_for_one]]},
      type: :supervisor
    }
  end

  @doc """
  Returns `true` if a request of the given `priority` should be shed.
  """
  @spec shed_request?(atom()) :: boolean()
  def shed_request?(priority) do
    Eel.LoadShedding.Controller.should_shed?(priority)
  end
end
