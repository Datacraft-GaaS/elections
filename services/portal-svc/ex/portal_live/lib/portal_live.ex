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

defmodule PortalLive do
  @moduledoc """
  eel portal-svc: Phoenix LiveView application for real-time election results.

  Streams live aggregate counts from Kafka projections via LiveView.
  Displays Merkle root cross-verified against verifier-rs WASM.
  Provides SSE endpoint for third-party feeds.
  No PII exposed: strictly aggregate counts only (FR-M11-026).
  """

  @version "0.1.0"

  def version, do: @version
end
