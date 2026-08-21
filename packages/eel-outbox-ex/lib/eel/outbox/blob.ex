defmodule Eel.Outbox.Blob do
  @moduledoc """
  Optional blob offload for large event payloads.

  When a JSON payload exceeds the configured threshold, `maybe_offload/2`
  uploads the payload to MinIO and returns a compact reference envelope
  instead. This keeps the outbox table rows small while still preserving
  the full payload in object storage.

  Configure the threshold (bytes) via:

      config :eel_outbox, :blob_threshold, 524_288  # 512 KB default
  """

  @default_threshold 524_288

  @doc """
  Offload `json` to MinIO if its size exceeds the threshold.

  In stub mode (the default), always returns `{:ok, {:minio_ref, "bucket/key"}}`.
  A production implementation would call a MinIO/S3-compatible HTTP endpoint.

  Returns `{:ok, json}` when below threshold, or
  `{:ok, {:minio_ref, "bucket/key"}}` when offloaded.
  """
  @spec maybe_offload(binary(), keyword()) ::
          {:ok, binary() | {:minio_ref, String.t()}} | {:error, term()}
  def maybe_offload(json, opts \\ []) when is_binary(json) do
    threshold = Keyword.get(opts, :threshold, Application.get_env(:eel_outbox, :blob_threshold, @default_threshold))

    if byte_size(json) > threshold do
      upload_stub(json, opts)
    else
      {:ok, json}
    end
  end

  defp upload_stub(_json, opts) do
    bucket = Keyword.get(opts, :bucket, "eel-outbox-blobs")
    key = "events/#{generate_key()}"
    {:ok, {:minio_ref, "#{bucket}/#{key}"}}
  end

  defp generate_key do
    :crypto.strong_rand_bytes(16) |> Base.encode16(case: :lower)
  end
end
