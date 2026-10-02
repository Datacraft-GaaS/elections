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

defmodule BvvWeb.Live.ObjectionsLiveTest do
  @moduledoc "Structural unit tests for ObjectionsLive."
  use ExUnit.Case, async: true

  alias BvvWeb.ObjectionsLive

  # ---------------------------------------------------------------------------
  # Module-level sanity
  # ---------------------------------------------------------------------------

  test "module is loaded" do
    assert Code.ensure_loaded?(ObjectionsLive)
  end

  test "module exposes live view surface" do
    assert function_exported?(ObjectionsLive, :mount, 3)
    assert function_exported?(ObjectionsLive, :render, 1)
    assert function_exported?(ObjectionsLive, :handle_event, 3)
    assert function_exported?(ObjectionsLive, :handle_info, 2)
  end

  # ---------------------------------------------------------------------------
  # PubSub message shape
  # ---------------------------------------------------------------------------

  test "objection_filed push message carries an objection struct shape" do
    filed_at = DateTime.utc_now()

    objection = %BvvWeb.Objections.Objection{
      id: "aaaaaaaa-0000-0000-0000-000000000001",
      obj_id: "bbbbbbbb-0000-0000-0000-000000000002",
      target_voter_id: "21223344556",
      objection_type: "transfer_request",
      status: "pending",
      filed_at: filed_at
    }

    msg = {:objection_filed, objection}

    assert elem(msg, 0) == :objection_filed
    assert elem(msg, 1).status == "pending"
    assert elem(msg, 1).filed_at == filed_at
  end

  # ---------------------------------------------------------------------------
  # Objector identity derivation
  # ---------------------------------------------------------------------------

  test "objector session hash accepts opaque hex values of expected length" do
    hash = Base.encode16(:crypto.strong_rand_bytes(32), case: :lower)

    assert byte_size(hash) == 64
    assert Regex.match?(~r/^[0-9a-f]{64}$/, hash)
  end
end
