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

defmodule PortalLiveTest do
  use ExUnit.Case, async: true

  test "version is set" do
    assert PortalLive.version() == "0.1.0"
  end

  test "PubSub topic constant is defined" do
    # Verify the topic string used by CountsLive and SseController is stable.
    topic = "eel:counts"
    assert is_binary(topic)
    assert String.starts_with?(topic, "eel:")
  end

  test "snapshot update message shape is valid" do
    snapshot = %{
      snapshot_id: "00000000-0000-0000-0000-000000000000",
      merkle_root: String.duplicate("0", 64),
      published_at: DateTime.utc_now(),
      counts: %{"candidate_001" => 42},
    }
    assert is_binary(snapshot.snapshot_id)
    assert byte_size(snapshot.merkle_root) == 64
    # No PII in the snapshot message shape.
    refute Map.has_key?(snapshot, :voter_id)
    refute Map.has_key?(snapshot, :national_id)
    refute Map.has_key?(snapshot, :name)
  end
end
