if Code.ensure_compiled(Credo.Check) == {:module, Credo.Check} do
  defmodule Eel.Resilience.Checks.ResilienceRequired do
    @moduledoc """
    Credo check that warns when HTTP client calls are not wrapped with resilience patterns.

    Currently a placeholder that always returns no issues. The intent is to detect
    direct `Finch.request/3` calls not wrapped by `Eel.Resilience.Retry` or
    `Eel.Resilience.CircuitBreaker`.
    """

    use Credo.Check,
      category: :warning,
      base_priority: :normal,
      explanations: [
        check: """
        HTTP client calls should be wrapped with a resilience pattern such as
        Eel.Resilience.Retry.with_retry/2 or Eel.Resilience.CircuitBreaker.call/3
        to prevent cascading failures.
        """,
        params: []
      ]

    @doc """
    Runs the check on `source_file`. Returns a list of issues.
    """
    def run(source_file, params) do
      issue_meta = IssueMeta.for(source_file, params)
      Credo.Code.prewalk(source_file, &traverse(&1, &2, issue_meta))
    end

    defp traverse(ast, issues, _issue_meta) do
      {ast, issues}
    end
  end
end
