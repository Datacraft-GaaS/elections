defmodule Eel.LoadShedding.Plug do
  @moduledoc """
  CoDel-inspired Plug that sheds requests when the system is under pressure.

  Checks request priority from the `X-Priority` header and consults
  `Eel.LoadShedding.Controller` to decide whether to shed the request.

  When shedding, responds with HTTP 503 and a `application/problem+json` body.
  """

  @behaviour Plug

  import Plug.Conn

  @problem_body Jason.encode!(%{
    status: 503,
    title: "Service Unavailable",
    detail: "Load shedding active. Retry after 1 second.",
    type: "https://eel.local/problems/load-shedding"
  })

  @impl Plug
  def init(opts) do
    %{
      priority_header: Keyword.get(opts, :priority_header, "x-priority"),
      max_latency_ms: Keyword.get(opts, :max_latency_ms, 5),
      interval_ms: Keyword.get(opts, :interval_ms, 100),
      min_throughput: Keyword.get(opts, :min_throughput, 1)
    }
  end

  @impl Plug
  def call(conn, opts) do
    priority = extract_priority(conn, opts.priority_header)
    started_at = System.monotonic_time(:millisecond)

    if shed?(priority) do
      conn
      |> put_resp_header("retry-after", "1")
      |> put_resp_content_type("application/problem+json")
      |> send_resp(503, @problem_body)
      |> halt()
    else
      register_before_send(conn, fn sent_conn ->
        latency = System.monotonic_time(:millisecond) - started_at
        Eel.LoadShedding.Controller.report_latency(latency)
        sent_conn
      end)
    end
  end

  defp extract_priority(conn, header) do
    case get_req_header(conn, String.downcase(header)) do
      ["high" | _] -> :high
      ["low" | _] -> :low
      _ -> :normal
    end
  end

  defp shed?(priority) do
    try do
      Eel.LoadShedding.Controller.should_shed?(priority)
    catch
      :exit, _ -> false
    end
  end
end
