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
