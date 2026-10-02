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

defmodule Eel.Resilience.Retry do
  @moduledoc """
  Retry with exponential backoff and optional jitter.
  """

  @doc """
  Executes `fun` with retry logic.

  ## Options

  - `:max_attempts` - maximum number of attempts (default: 3).
  - `:base_delay_ms` - base delay in milliseconds (default: 100).
  - `:max_delay_ms` - maximum delay in milliseconds (default: 5000).
  - `:jitter` - add random jitter to delay (default: true).

  Returns the result of `fun.()` on success, or `{:error, :max_retries_exceeded}`.
  """
  @spec with_retry((-> any()), keyword()) :: any() | {:error, :max_retries_exceeded}
  def with_retry(fun, opts \\ []) do
    max_attempts = Keyword.get(opts, :max_attempts, 3)
    base_delay_ms = Keyword.get(opts, :base_delay_ms, 100)
    max_delay_ms = Keyword.get(opts, :max_delay_ms, 5000)
    jitter = Keyword.get(opts, :jitter, true)

    do_retry(fun, 0, max_attempts, base_delay_ms, max_delay_ms, jitter)
  end

  defp do_retry(_fun, attempt, max_attempts, _base, _max, _jitter)
       when attempt >= max_attempts do
    {:error, :max_retries_exceeded}
  end

  defp do_retry(fun, attempt, max_attempts, base_delay_ms, max_delay_ms, jitter) do
    try do
      result = fun.()

      case result do
        {:error, _} when attempt + 1 < max_attempts ->
          sleep_ms = compute_delay(attempt, base_delay_ms, max_delay_ms, jitter)
          Process.sleep(sleep_ms)
          do_retry(fun, attempt + 1, max_attempts, base_delay_ms, max_delay_ms, jitter)

        {:error, _} ->
          {:error, :max_retries_exceeded}

        _ ->
          result
      end
    rescue
      _ ->
        if attempt + 1 < max_attempts do
          sleep_ms = compute_delay(attempt, base_delay_ms, max_delay_ms, jitter)
          Process.sleep(sleep_ms)
          do_retry(fun, attempt + 1, max_attempts, base_delay_ms, max_delay_ms, jitter)
        else
          {:error, :max_retries_exceeded}
        end
    end
  end

  defp compute_delay(attempt, base_delay_ms, max_delay_ms, jitter) do
    delay = min(base_delay_ms * Integer.pow(2, attempt), max_delay_ms)

    if jitter do
      delay + :rand.uniform(max(div(delay, 2), 1))
    else
      delay
    end
  end
end
