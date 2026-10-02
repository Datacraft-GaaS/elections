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

defmodule Eel.Observability.Slo do
  @moduledoc """
  SLO (Service Level Objective) helpers.

  Records request metrics as OpenTelemetry events and provides
  compliance checks against defined targets.
  """

  @doc """
  Records a request metric for SLO tracking.

  Emits an OpenTelemetry event on the current span with service, route,
  status code, and duration attributes.
  """
  @spec record_request(String.t(), String.t(), non_neg_integer(), non_neg_integer()) :: :ok
  def record_request(service, route, status_code, duration_ms)
      when is_binary(service) and is_binary(route) and is_integer(status_code) and
             is_integer(duration_ms) do
    ctx = OpenTelemetry.Tracer.current_span_ctx()

    OpenTelemetry.Span.add_event(ctx, "slo.request", %{
      "service" => service,
      "route" => route,
      "http.status_code" => status_code,
      "duration_ms" => duration_ms
    })

    :ok
  end

  @doc """
  Checks SLO compliance for `service` over the last `window_seconds`.

  Returns `{:ok, true}` as a stub. A real implementation would query
  Prometheus for error rate and latency percentile data.
  """
  @spec slo_compliant?(String.t(), pos_integer()) :: {:ok, boolean()}
  def slo_compliant?(_service, _window_seconds) do
    {:ok, true}
  end
end
