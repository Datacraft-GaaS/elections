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

defmodule Eel.Resilience.Timeout do
  @moduledoc """
  Executes a function with a hard timeout, killing the task on expiry.
  """

  @doc """
  Runs `fun` in an async task. Returns the function result or `{:error, :timeout}`.

  If the task does not complete within `ms` milliseconds, the task process
  is killed and `{:error, :timeout}` is returned.
  """
  @spec with_timeout((-> any()), pos_integer()) :: any() | {:error, :timeout}
  def with_timeout(fun, ms) when is_integer(ms) and ms > 0 do
    task = Task.async(fun)

    case Task.yield(task, ms) do
      {:ok, result} ->
        result

      nil ->
        Task.shutdown(task, :brutal_kill)
        {:error, :timeout}
    end
  end
end
