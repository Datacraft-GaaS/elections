defmodule Eel.EventsTest do
  use ExUnit.Case, async: true

  alias Eel.Events.{Builder, CanonicalJson, Envelope, Signing}

  describe "CanonicalJson.encode/1" do
    test "sorts object keys lexicographically" do
      assert {:ok, json} = CanonicalJson.encode(%{"b" => 1, "a" => 2})
      assert json == ~S({"a":2,"b":1})
    end

    test "sorts nested object keys recursively" do
      assert {:ok, json} = CanonicalJson.encode(%{"z" => %{"b" => 1, "a" => 2}})
      assert json == ~S({"z":{"a":2,"b":1}})
    end

    test "preserves array order" do
      assert {:ok, json} = CanonicalJson.encode([3, 1, 2])
      assert json == "[3,1,2]"
    end

    test "encodes null" do
      assert {:ok, "null"} = CanonicalJson.encode(nil)
    end

    test "encodes booleans" do
      assert {:ok, "true"} = CanonicalJson.encode(true)
      assert {:ok, "false"} = CanonicalJson.encode(false)
    end

    test "encodes integers without decimal point" do
      assert {:ok, "42"} = CanonicalJson.encode(42)
    end

    test "encodes strings with JSON escaping" do
      assert {:ok, ~S("hello")} = CanonicalJson.encode("hello")
    end

    test "encodes empty map" do
      assert {:ok, "{}"} = CanonicalJson.encode(%{})
    end
  end

  describe "Envelope.from_map/1 and to_map/1" do
    test "round-trips a map with string keys" do
      original = %{
        "specversion" => "1.0",
        "id" => "01950f8c-aaaa-bbbb-cccc-dddddddddddd",
        "type" => "eel.rts.form34a_captured.v1",
        "source" => "spiffe://eel.local/rts"
      }

      assert {:ok, envelope} = Envelope.from_map(original)
      assert envelope.specversion == "1.0"
      assert envelope.id == "01950f8c-aaaa-bbbb-cccc-dddddddddddd"
      assert envelope.type == "eel.rts.form34a_captured.v1"
      assert envelope.source == "spiffe://eel.local/rts"

      result = Envelope.to_map(envelope)
      assert result["specversion"] == "1.0"
      assert result["type"] == "eel.rts.form34a_captured.v1"
      refute Map.has_key?(result, "subject")
    end

    test "accepts atom keys in source map" do
      assert {:ok, env} =
               Envelope.from_map(%{specversion: "1.0", id: "x", type: "t", source: "s"})

      assert env.specversion == "1.0"
    end

    test "omits nil fields from to_map output" do
      {:ok, env} = Envelope.from_map(%{"specversion" => "1.0", "id" => "x", "type" => "t", "source" => "s"})
      map = Envelope.to_map(env)
      refute Map.has_key?(map, "eel_signature")
      refute Map.has_key?(map, "subject")
    end
  end

  describe "Envelope.validate/1" do
    test "returns :ok for valid envelope" do
      {:ok, env} = Envelope.from_map(%{"specversion" => "1.0", "id" => "x", "type" => "t", "source" => "s"})
      assert :ok = Envelope.validate(env)
    end

    test "returns errors for missing required fields" do
      env = %Envelope{}
      assert {:error, errors} = Envelope.validate(env)
      assert :specversion in errors
      assert :id in errors
      assert :type in errors
      assert :source in errors
    end

    test "returns error for invalid eel_data_class" do
      {:ok, env} = Envelope.from_map(%{"specversion" => "1.0", "id" => "x", "type" => "t", "source" => "s"})
      bad = %{env | eel_data_class: "Z"}
      assert {:error, [:eel_data_class]} = Envelope.validate(bad)
    end
  end

  describe "Builder" do
    test "builds a valid envelope with required fields" do
      assert {:ok, envelope} =
               Builder.new("eel.test.v1", "spiffe://eel.local/test", %{key: "val"})
               |> Builder.build()

      assert envelope.specversion == "1.0"
      assert envelope.type == "eel.test.v1"
      assert envelope.source == "spiffe://eel.local/test"
      assert is_binary(envelope.id)
      assert String.length(envelope.id) == 36
    end

    test "builder fluent API sets extension fields" do
      assert {:ok, envelope} =
               Builder.new("eel.test.v1", "spiffe://eel.local/test", %{})
               |> Builder.with_election("2027-general")
               |> Builder.with_data_class("B")
               |> Builder.with_correlation("corr-123")
               |> Builder.with_causation("cause-456")
               |> Builder.with_subject("form-34a/47-0001-001")
               |> Builder.with_hlc("1723731000123.5.42")
               |> Builder.with_stream("stream-id", 42, "sha256:abc")
               |> Builder.build()

      assert envelope.eel_election_id == "2027-general"
      assert envelope.eel_data_class == "B"
      assert envelope.eel_correlation_id == "corr-123"
      assert envelope.eel_causation_id == "cause-456"
      assert envelope.subject == "form-34a/47-0001-001"
      assert envelope.eel_hlc == "1723731000123.5.42"
      assert envelope.eel_stream_id == "stream-id"
      assert envelope.eel_seq == 42
      assert envelope.eel_prev_hash == "sha256:abc"
    end
  end

  describe "Signing" do
    test "sign and verify with raw Ed25519 keys" do
      {pub_key, priv_key} = :crypto.generate_key(:eddsa, :ed25519)

      {:ok, envelope} =
        Builder.new("eel.test.v1", "spiffe://eel.local/test", %{})
        |> Builder.build()

      assert {:ok, signed} = Signing.sign(envelope, priv_key)
      assert is_binary(signed.eel_signature)
      assert String.starts_with?(signed.eel_signature, "ed25519:")

      assert :ok = Signing.verify(signed, pub_key)
    end

    test "verify returns :missing_signature when no signature present" do
      {:ok, envelope} =
        Builder.new("eel.test.v1", "spiffe://eel.local/test", %{})
        |> Builder.build()

      {pub_key, _priv_key} = :crypto.generate_key(:eddsa, :ed25519)
      assert {:error, :missing_signature} = Signing.verify(envelope, pub_key)
    end

    test "verify returns :invalid_signature when signature does not match" do
      {pub_key, priv_key} = :crypto.generate_key(:eddsa, :ed25519)
      {_other_pub, other_priv} = :crypto.generate_key(:eddsa, :ed25519)

      {:ok, envelope} =
        Builder.new("eel.test.v1", "spiffe://eel.local/test", %{})
        |> Builder.build()

      {:ok, signed} = Signing.sign(envelope, other_priv)
      assert {:error, :invalid_signature} = Signing.verify(signed, pub_key)

      _ = priv_key
    end
  end
end
