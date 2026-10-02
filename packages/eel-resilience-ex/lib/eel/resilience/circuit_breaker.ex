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

defmodule Eel.Resilience.CircuitBreaker do
  @moduledoc """
  A GenServer implementing the circuit breaker pattern.

  States:
  - `:closed` - normal operation; failures are counted.
  - `:open` - requests are rejected immediately; resets after `timeout_ms`.
  - `:half_open` - probing state; limited requests allowed through.
  """

  use GenServer

  @type state_name :: :closed | :open | :half_open

  @type t :: %{
    state: state_name(),
    failure_count: non_neg_integer(),
    success_count: non_neg_integer(),
    threshold: pos_integer(),
    timeout_ms: pos_integer(),
    success_threshold: pos_integer(),
    opened_at: integer() | nil
  }

  # Client API

  @doc """
  Starts a CircuitBreaker GenServer.

  ## Options

  - `:name` - registration name (optional).
  - `:threshold` - failure count before opening (default: 5).
  - `:timeout_ms` - milliseconds before transitioning open -> half_open (default: 30_000).
  - `:success_threshold` - successes in half_open to close (default: 2).
  """
  def start_link(opts \\ []) do
    name = Keyword.get(opts, :name)
    gen_opts = if name, do: [name: name], else: []
    GenServer.start_link(__MODULE__, opts, gen_opts)
  end

  @doc """
  Executes `fun` through the circuit breaker.

  Returns `{:error, :circuit_open}` if the circuit is open.
  Otherwise returns the result of `fun.()`.
  """
  @spec call(GenServer.server(), (-> any()), timeout()) :: any() | {:error, :circuit_open}
  def call(pid_or_name, fun, timeout \\ 5000) do
    GenServer.call(pid_or_name, {:call, fun}, timeout)
  end

  @doc """
  Returns the current state map of the circuit breaker.
  """
  @spec state(GenServer.server()) :: t()
  def state(pid_or_name) do
    GenServer.call(pid_or_name, :state)
  end

  # Server callbacks

  @impl true
  def init(opts) do
    state = %{
      state: :closed,
      failure_count: 0,
      success_count: 0,
      threshold: Keyword.get(opts, :threshold, 5),
      timeout_ms: Keyword.get(opts, :timeout_ms, 30_000),
      success_threshold: Keyword.get(opts, :success_threshold, 2),
      opened_at: nil
    }

    {:ok, state}
  end

  @impl true
  def handle_call({:call, fun}, _from, %{state: :open} = data) do
    data = maybe_transition_to_half_open(data)

    case data.state do
      :open ->
        {:reply, {:error, :circuit_open}, data}

      :half_open ->
        {result, new_data} = execute(fun, data)
        {:reply, result, new_data}
    end
  end

  def handle_call({:call, fun}, _from, data) do
    {result, new_data} = execute(fun, data)
    {:reply, result, new_data}
  end

  def handle_call(:state, _from, data) do
    {:reply, data, data}
  end

  # Private helpers

  defp execute(fun, data) do
    try do
      result = fun.()

      case result do
        {:error, _} ->
          {result, record_failure(data)}

        _ ->
          {result, record_success(data)}
      end
    rescue
      e ->
        {{:error, e}, record_failure(data)}
    end
  end

  defp record_success(%{state: :half_open} = data) do
    count = data.success_count + 1

    if count >= data.success_threshold do
      %{data | state: :closed, failure_count: 0, success_count: 0, opened_at: nil}
    else
      %{data | success_count: count}
    end
  end

  defp record_success(data) do
    %{data | failure_count: 0}
  end

  defp record_failure(%{state: :half_open} = data) do
    %{data | state: :open, opened_at: System.monotonic_time(:millisecond), success_count: 0}
  end

  defp record_failure(data) do
    count = data.failure_count + 1

    if count >= data.threshold do
      %{data | state: :open, failure_count: count, opened_at: System.monotonic_time(:millisecond)}
    else
      %{data | failure_count: count}
    end
  end

  defp maybe_transition_to_half_open(%{state: :open, opened_at: opened_at, timeout_ms: timeout_ms} = data) do
    now = System.monotonic_time(:millisecond)

    if now - opened_at >= timeout_ms do
      %{data | state: :half_open, success_count: 0}
    else
      data
    end
  end

  defp maybe_transition_to_half_open(data), do: data
end
