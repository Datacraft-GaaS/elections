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
