defmodule Eel.Resilience.Bulkhead do
  @moduledoc """
  Bulkhead pattern via a poolboy worker pool.

  Limits concurrency by checking out workers from a fixed-size pool.
  """

  @doc """
  Returns a child spec for the bulkhead pool supervisor.

  ## Options

  - `:name` - pool registration name (required).
  - `:size` - pool size (default: 10).
  - `:overflow` - max overflow workers (default: 2).
  """
  @spec child_spec(keyword()) :: Supervisor.child_spec()
  def child_spec(opts) do
    name = Keyword.fetch!(opts, :name)
    size = Keyword.get(opts, :size, 10)
    overflow = Keyword.get(opts, :overflow, 2)

    pool_opts = [
      name: {:local, name},
      worker_module: Eel.Resilience.Bulkhead.Worker,
      size: size,
      max_overflow: overflow
    ]

    :poolboy.child_spec(name, pool_opts, [])
  end

  @doc """
  Checks out a worker from `pool`, runs `fun.(worker_pid)`, and checks the worker back in.

  Returns the result of `fun.(worker_pid)` or `{:error, :checkout_timeout}`.
  """
  @spec checkout(atom(), (pid() -> any()), timeout()) :: any() | {:error, :checkout_timeout}
  def checkout(pool, fun, timeout \\ 5000) do
    try do
      :poolboy.transaction(pool, fun, timeout)
    catch
      :exit, {:timeout, _} -> {:error, :checkout_timeout}
    end
  end
end

defmodule Eel.Resilience.Bulkhead.Worker do
  @moduledoc """
  Placeholder GenServer worker for the Bulkhead pool.

  The worker itself is a pass-through; callers receive the worker pid
  and may use it to serialize work or track resource usage.
  """

  use GenServer

  @doc false
  def start_link(_args) do
    GenServer.start_link(__MODULE__, :ok, [])
  end

  @impl true
  def init(:ok), do: {:ok, :ok}
end
