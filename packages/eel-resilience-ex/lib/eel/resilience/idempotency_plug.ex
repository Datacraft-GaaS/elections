defmodule Eel.Resilience.IdempotencyPlug do
  @moduledoc """
  Plug that enforces idempotency for mutating HTTP methods using an `Idempotency-Key` header.

  On POST/PUT/PATCH requests:
  - If the key has been seen before with the same body hash, returns the cached response.
  - If the key has been seen before with a different body hash, returns 422.
  - If the key is new, lets the request proceed and caches the response.

  Responses are stored in an ETS table named `:eel_idempotency_cache`.
  """

  @behaviour Plug

  import Plug.Conn

  @table :eel_idempotency_cache
  @idempotency_header "idempotency-key"
  @mutable_methods ~w(POST PUT PATCH)

  @impl Plug
  def init(opts), do: opts

  @impl Plug
  def call(conn, _opts) do
    ensure_table()

    if conn.method in @mutable_methods do
      case get_req_header(conn, @idempotency_header) do
        [key | _] -> handle_idempotency(conn, key)
        [] -> conn
      end
    else
      conn
    end
  end

  defp handle_idempotency(conn, key) do
    {:ok, body, conn} = read_body(conn)
    body_hash = :crypto.hash(:sha256, body) |> Base.encode16(case: :lower)

    case :ets.lookup(@table, key) do
      [{^key, ^body_hash, cached_status, cached_body}] ->
        conn
        |> put_resp_content_type("application/json")
        |> send_resp(cached_status, cached_body)
        |> halt()

      [{^key, _other_hash, _cached_status, _cached_body}] ->
        body_resp = Jason.encode!(%{status: 422, title: "Idempotency Key Conflict"})

        conn
        |> put_resp_content_type("application/problem+json")
        |> send_resp(422, body_resp)
        |> halt()

      [] ->
        conn
        |> put_private(:eel_idempotency_key, key)
        |> put_private(:eel_idempotency_hash, body_hash)
        |> register_before_send(&cache_response/1)
    end
  end

  defp cache_response(conn) do
    key = conn.private[:eel_idempotency_key]
    hash = conn.private[:eel_idempotency_hash]

    if key && hash do
      body = conn.resp_body || ""
      :ets.insert(@table, {key, hash, conn.status, body})
    end

    conn
  end

  defp ensure_table do
    case :ets.whereis(@table) do
      :undefined ->
        :ets.new(@table, [:named_table, :public, :set, read_concurrency: true])

      _ ->
        :ok
    end
  end
end
