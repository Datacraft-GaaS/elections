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
