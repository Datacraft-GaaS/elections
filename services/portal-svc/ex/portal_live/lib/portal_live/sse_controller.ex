defmodule PortalLive.SseController do
  @moduledoc """
  SSE endpoint for third-party aggregate count feeds (ADR-0011 Tier 5).
  Public, unauthenticated. No PII. Strictly aggregate counts.
  Rate limiting enforced at edge (ADR-0010); no auth token required.
  """
  use Phoenix.Controller, formats: [:html, :json]

  @topic "eel:counts"
  @keepalive_ms 15_000

  def counts(conn, _params) do
    conn =
      conn
      |> put_resp_content_type("text/event-stream")
      |> put_resp_header("cache-control", "no-cache")
      |> put_resp_header("x-accel-buffering", "no")
      |> send_chunked(200)

    Phoenix.PubSub.subscribe(PortalLive.PubSub, @topic)
    sse_loop(conn)
  end

  defp sse_loop(conn) do
    receive do
      {:snapshot_updated, snapshot} ->
        data = Jason.encode!(%{
          snapshot_id: snapshot.snapshot_id,
          merkle_root: snapshot.merkle_root,
          published_at: snapshot.published_at,
          counts: snapshot.counts,
        })
        case chunk(conn, "data: #{data}\n\n") do
          {:ok, conn} -> sse_loop(conn)
          {:error, _} -> conn
        end
    after
      @keepalive_ms ->
        case chunk(conn, ": keepalive\n\n") do
          {:ok, conn} -> sse_loop(conn)
          {:error, _} -> conn
        end
    end
  end
end
