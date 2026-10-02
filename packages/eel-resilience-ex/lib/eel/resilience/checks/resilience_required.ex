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
