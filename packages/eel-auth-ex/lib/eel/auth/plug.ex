defmodule Eel.Auth.Plug do
  @moduledoc """
  Plug that verifies JWT bearer tokens and populates `conn.assigns[:current_subject]`.

  On verification failure, responds with 401 and an `application/problem+json` body
  and halts the connection.

  ## Options

  - `jwks` - pid or registered name of the `Eel.Auth.JWKS` server (required)
  - `optional` - when `true`, missing tokens do not halt; default `false`
  """

  @behaviour Plug

  import Plug.Conn

  alias Eel.Auth.JWT

  @type opts :: [jwks: GenServer.server(), optional: boolean()]

  @impl true
  def init(opts) do
    %{
      jwks: Keyword.fetch!(opts, :jwks),
      optional: Keyword.get(opts, :optional, false)
    }
  end

  @impl true
  def call(conn, %{jwks: jwks, optional: optional}) do
    case get_req_header(conn, "authorization") do
      ["Bearer " <> token | _] ->
        case JWT.verify(token, jwks) do
          {:ok, subject} ->
            assign(conn, :current_subject, subject)

          {:error, _reason} ->
            unauthorized(conn)
        end

      _ ->
        if optional do
          conn
        else
          unauthorized(conn)
        end
    end
  end

  defp unauthorized(conn) do
    body =
      Jason.encode!(%{
        "type" => "about:blank",
        "title" => "Unauthorized",
        "status" => 401
      })

    conn
    |> put_resp_content_type("application/problem+json")
    |> send_resp(401, body)
    |> halt()
  end
end
