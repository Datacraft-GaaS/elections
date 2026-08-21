defmodule Eel.LoadShedding.Controller do
  @moduledoc """
  GenServer implementing a CoDel-inspired load shedding controller.

  Tracks request latency and decides whether to shed incoming requests
  based on priority and the current shedding state.
  """

  use GenServer

  @default_name __MODULE__

  @type state :: %{
    dropping: boolean(),
    drop_next: integer() | nil,
    first_above_time: integer() | nil,
    count: non_neg_integer(),
    interval_ms: pos_integer(),
    max_latency_ms: pos_integer()
  }

  # Client API

  @doc """
  Starts the Controller GenServer.

  ## Options

  - `:name` - registration name (default: `Eel.LoadShedding.Controller`).
  - `:interval_ms` - CoDel interval window in milliseconds (default: 100).
  - `:max_latency_ms` - target maximum latency in milliseconds (default: 5).
  """
  def start_link(opts \\ []) do
    name = Keyword.get(opts, :name, @default_name)
    GenServer.start_link(__MODULE__, opts, name: name)
  end

  @doc """
  Returns `true` if a request of the given `priority` should be shed.

  Priorities: `:high` (never shed), `:normal` (shed if dropping), `:low` (shed aggressively).
  """
  @spec should_shed?(atom()) :: boolean()
  def should_shed?(priority) do
    GenServer.call(@default_name, {:should_shed?, priority})
  end

  @doc """
  Reports the observed latency of a completed request in milliseconds.

  Updates the internal CoDel algorithm state.
  """
  @spec report_latency(number()) :: :ok
  def report_latency(ms) do
    GenServer.cast(@default_name, {:report_latency, ms})
  end

  # Server callbacks

  @impl true
  def init(opts) do
    state = %{
      dropping: false,
      drop_next: nil,
      first_above_time: nil,
      count: 0,
      high_count: 0,
      interval_ms: Keyword.get(opts, :interval_ms, 100),
      max_latency_ms: Keyword.get(opts, :max_latency_ms, 5)
    }

    {:ok, state}
  end

  @impl true
  def handle_call({:should_shed?, :high}, _from, state) do
    {:reply, false, state}
  end

  def handle_call({:should_shed?, :normal}, _from, state) do
    {:reply, state.dropping, state}
  end

  def handle_call({:should_shed?, :low}, _from, state) do
    {:reply, state.dropping or state.first_above_time != nil, state}
  end

  def handle_call({:should_shed?, _priority}, _from, state) do
    {:reply, state.dropping, state}
  end

  @impl true
  def handle_cast({:report_latency, ms}, state) do
    now = System.monotonic_time(:millisecond)
    new_state = update_codel(state, ms, now)
    {:noreply, new_state}
  end

  defp update_codel(state, ms, now) when ms > state.max_latency_ms do
    first_above =
      if state.first_above_time == nil do
        now
      else
        state.first_above_time
      end

    high_count = state.high_count + 1
    sample_threshold = max(state.interval_ms, 2)
    time_elapsed = now - first_above >= state.interval_ms
    count_exceeded = high_count >= sample_threshold

    if time_elapsed or count_exceeded do
      count = state.count + 1
      drop_next = now + drop_interval(count, state.interval_ms)

      %{
        state
        | dropping: true,
          count: count,
          drop_next: drop_next,
          first_above_time: first_above,
          high_count: high_count
      }
    else
      %{state | first_above_time: first_above, high_count: high_count}
    end
  end

  defp update_codel(state, _ms, _now) do
    %{state | first_above_time: nil, dropping: false, drop_next: nil, count: 0, high_count: 0}
  end

  defp drop_interval(count, interval_ms) when count > 0 do
    round(interval_ms / :math.sqrt(count))
  end

  defp drop_interval(_count, interval_ms), do: interval_ms
end
