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

defmodule EccLive.Live.WorkforceLiveTest do
  @moduledoc "ExUnit unit tests for WorkforceLive."
  use ExUnit.Case, async: true

  alias EccLive.Live.WorkforceLive

  # ---------------------------------------------------------------------------
  # Module-level sanity
  # ---------------------------------------------------------------------------

  test "module is loaded" do
    assert Code.ensure_loaded?(WorkforceLive)
  end

  test "module exposes live view surface" do
    assert function_exported?(WorkforceLive, :mount, 3)
    assert function_exported?(WorkforceLive, :render, 1)
    assert function_exported?(WorkforceLive, :handle_info, 2)
  end

  # ---------------------------------------------------------------------------
  # Mount / readiness loading
  # ---------------------------------------------------------------------------

  test "mount assigns empty degraded readiness when the api is unreachable" do
    {:ok, socket} = WorkforceLive.mount(nil, nil, %Phoenix.LiveView.Socket{})

    assert socket.assigns.stations == []
    assert socket.assigns.totals == %{
             "active_postings" => 0,
             "open_kit_claims" => 0,
             "assignments" => 0
           }
    assert socket.assigns.degraded == true
  end

  # ---------------------------------------------------------------------------
  # Fetch handling
  # ---------------------------------------------------------------------------

  test "fetch_readiness handles error maps without raising" do
    readiness = WorkforceLive.fetch_readiness()

    assert is_map(readiness)
    assert Map.has_key?(readiness, "stations")
    assert Map.has_key?(readiness, "totals")
  end

  test "fetch_readiness degrades when unreachable" do
    Application.put_env(:ecc_live, :workforce_api_url, "http://127.0.0.1:1", persistent: true)

    on_exit(fn -> Application.delete_env(:ecc_live, :workforce_api_url) end)

    readiness = WorkforceLive.fetch_readiness()

    assert readiness["degraded"] == true
    assert readiness["stations"] == []
  end
end
