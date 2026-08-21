defmodule QueueMapLive.SseController do
  @moduledoc "SSE endpoint streaming queue updates as text/event-stream."
  use QueueMapLive, :controller

  require Logger

  def queue_updates(conn, _params) do
    conn =
      conn
      |> put_resp_content_type("text/event-stream")
      |> put_resp_header("cache-control", "no-cache")
      |> put_resp_header("connection", "keep-alive")
      |> send_chunked(200)

    Phoenix.PubSub.subscribe(QueueMapLive.PubSub, "queue:updates")
    Logger.info("SSE client connected", event: "sse_connect")
    sse_loop(conn)
  end

  defp sse_loop(conn) do
    receive do
      {:queue_update, payload} ->
        data = Jason.encode!(payload)
        chunk = "event: queue_update\ndata: #{data}\n\n"

        case Plug.Conn.chunk(conn, chunk) do
          {:ok, conn} -> sse_loop(conn)
          {:error, :closed} -> conn
        end

      :close ->
        conn
    after
      30_000 ->
        case Plug.Conn.chunk(conn, ": keepalive\n\n") do
          {:ok, conn} -> sse_loop(conn)
          {:error, :closed} -> conn
        end
    end
  end
end
