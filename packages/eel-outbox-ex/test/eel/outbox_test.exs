defmodule Eel.OutboxTest do
  use ExUnit.Case, async: true

  alias Eel.Outbox
  alias Eel.Outbox.{Blob, Entry}

  describe "Eel.Outbox.emit/2" do
    test "returns Multi with one outbox operation for a valid envelope" do
      envelope = %{
        "id" => "01950f8c-aaaa-bbbb-cccc-dddddddddddd",
        "type" => "eel.rts.form34a_captured.v1",
        "source" => "spiffe://eel.local/rts",
        "specversion" => "1.0"
      }

      result = Outbox.emit(Ecto.Multi.new(), envelope)

      assert %Ecto.Multi{} = result
      assert length(result.operations) == 1
    end

    test "derives topic from event type field" do
      envelope = %{"id" => "abc", "type" => "eel.vote.cast.v1", "source" => "s"}

      result = Outbox.emit(Ecto.Multi.new(), envelope)
      assert %Ecto.Multi{} = result

      [{_name, {:changeset, changeset, _opts}}] = result.operations
      assert Ecto.Changeset.get_field(changeset, :topic) == "eel.vote.cast.v1"
    end

    test "rejects envelope whose JSON exceeds 1 MB" do
      large_string = String.duplicate("x", 1_048_577)
      envelope = %{"id" => "big", "type" => "t", "data" => large_string}

      assert {:error, :payload_too_large} = Outbox.emit(Ecto.Multi.new(), envelope)
    end

    test "accepts atom-keyed envelope (struct-like)" do
      envelope = %{id: "abc", type: "eel.test.v1", source: "s"}

      result = Outbox.emit(Ecto.Multi.new(), envelope)
      assert %Ecto.Multi{} = result
    end

    test "accumulates alongside existing Multi operations" do
      envelope = %{"id" => "x1", "type" => "eel.test.v1", "source" => "s"}

      existing =
        Ecto.Multi.new()
        |> Ecto.Multi.run(:noop, fn _repo, _changes -> {:ok, :done} end)

      result = Outbox.emit(existing, envelope)
      assert %Ecto.Multi{} = result
      assert length(result.operations) == 2
    end
  end

  describe "Eel.Outbox.Entry changeset" do
    test "valid attrs produce a valid changeset" do
      attrs = %{
        event_id: "evt-1",
        event_type: "eel.test.v1",
        topic: "eel.test.v1",
        payload: ~s({"hello":"world"})
      }

      cs = Entry.changeset(%Entry{}, attrs)
      assert cs.valid?
    end

    test "missing required fields produce errors" do
      cs = Entry.changeset(%Entry{}, %{})
      refute cs.valid?
      assert cs.errors[:event_id]
      assert cs.errors[:event_type]
      assert cs.errors[:topic]
      assert cs.errors[:payload]
    end
  end

  describe "Eel.Outbox.Blob.maybe_offload/2" do
    test "returns json unchanged when below threshold" do
      json = ~s({"key":"value"})
      assert {:ok, ^json} = Blob.maybe_offload(json)
    end

    test "returns minio_ref when payload exceeds threshold" do
      large = String.duplicate("x", 524_289)
      assert {:ok, {:minio_ref, ref}} = Blob.maybe_offload(large)
      assert String.starts_with?(ref, "eel-outbox-blobs/events/")
    end

    test "respects custom threshold option" do
      json = "hello"
      assert {:ok, {:minio_ref, _}} = Blob.maybe_offload(json, threshold: 1)
    end
  end
end
