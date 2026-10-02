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

defmodule Eel.Events.Signing do
  @moduledoc """
  Ed25519 signing and verification for `Eel.Events.Envelope` structs.

  The signed payload is the RFC 8785 canonical JSON of the envelope with
  the `eel_signature` field removed. The signature is stored as
  `"ed25519:<base64>"`.

  OTP 22+ provides Ed25519 natively via `:crypto`.
  """

  alias Eel.Events.{CanonicalJson, Envelope}

  @doc """
  Sign `envelope` with an Ed25519 private key.

  `private_key_pem` may be:
  - A raw binary (32 or 64 bytes) returned by `:crypto.generate_key/2`.
  - A PEM-encoded PKCS#8 private key string.

  Returns `{:ok, signed_envelope}` with `eel_signature` set.
  """
  @spec sign(Envelope.t(), binary()) :: {:ok, Envelope.t()} | {:error, term()}
  def sign(%Envelope{} = envelope, private_key_pem) do
    with {:ok, private_key} <- decode_ed25519_private_key(private_key_pem),
         {:ok, msg} <- canonical_message(envelope) do
      signature = :crypto.sign(:eddsa, :none, msg, [private_key, :ed25519])
      sig_b64 = Base.encode64(signature)
      {:ok, %{envelope | eel_signature: "ed25519:" <> sig_b64}}
    end
  end

  @doc """
  Verify the `eel_signature` on `envelope` using an Ed25519 public key.

  `public_key_pem` may be a raw 32-byte binary or a PEM-encoded public key.

  Returns `:ok` or `{:error, :invalid_signature | :missing_signature}`.
  """
  @spec verify(Envelope.t(), binary()) ::
          :ok | {:error, :invalid_signature | :missing_signature}
  def verify(%Envelope{eel_signature: nil}, _pub_key) do
    {:error, :missing_signature}
  end

  def verify(%Envelope{eel_signature: sig} = envelope, public_key_pem)
      when is_binary(sig) do
    with [scheme, sig_b64] <- String.split(sig, ":", parts: 2),
         true <- scheme == "ed25519",
         {:ok, sig_bytes} <- Base.decode64(sig_b64),
         {:ok, public_key} <- decode_ed25519_public_key(public_key_pem),
         {:ok, msg} <- canonical_message(envelope) do
      if :crypto.verify(:eddsa, :none, msg, sig_bytes, [public_key, :ed25519]) do
        :ok
      else
        {:error, :invalid_signature}
      end
    else
      _ -> {:error, :invalid_signature}
    end
  end

  @doc """
  Decode an Ed25519 private key from a PEM string or raw binary.

  Accepts raw 32/64-byte binaries directly for testing convenience.
  """
  @spec decode_ed25519_private_key(binary()) :: {:ok, binary()} | {:error, term()}
  def decode_ed25519_private_key(<<"-----BEGIN", _::binary>> = pem) do
    case :public_key.pem_decode(pem) do
      [entry] ->
        try do
          decoded = :public_key.pem_entry_decode(entry)
          extract_private_key(decoded)
        rescue
          e -> {:error, Exception.message(e)}
        end

      _ ->
        {:error, "expected exactly one PEM entry"}
    end
  end

  def decode_ed25519_private_key(key) when is_binary(key), do: {:ok, key}

  @doc """
  Decode an Ed25519 public key from a PEM string or raw binary.

  Accepts raw 32-byte binaries directly for testing convenience.
  """
  @spec decode_ed25519_public_key(binary()) :: {:ok, binary()} | {:error, term()}
  def decode_ed25519_public_key(<<"-----BEGIN", _::binary>> = pem) do
    case :public_key.pem_decode(pem) do
      [entry] ->
        try do
          decoded = :public_key.pem_entry_decode(entry)
          extract_public_key(decoded)
        rescue
          e -> {:error, Exception.message(e)}
        end

      _ ->
        {:error, "expected exactly one PEM entry"}
    end
  end

  def decode_ed25519_public_key(key) when is_binary(key), do: {:ok, key}

  # Private helpers

  defp canonical_message(%Envelope{} = envelope) do
    map =
      envelope
      |> Map.put(:eel_signature, nil)
      |> Envelope.to_map()

    CanonicalJson.encode(map)
  end

  defp extract_private_key({:ed_pri, :ed25519, _pub, priv}), do: {:ok, priv}
  defp extract_private_key({:ECPrivateKey, _ver, priv, _params, _pub}), do: {:ok, priv}

  defp extract_private_key(other),
    do: {:error, "unsupported private key format: #{inspect(other)}"}

  defp extract_public_key({:ed_pub, :ed25519, pub}), do: {:ok, pub}

  defp extract_public_key({:SubjectPublicKeyInfo, _algo, pub}) when is_binary(pub),
    do: {:ok, pub}

  defp extract_public_key(other),
    do: {:error, "unsupported public key format: #{inspect(other)}"}
end
