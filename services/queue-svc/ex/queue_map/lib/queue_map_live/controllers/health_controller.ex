defmodule QueueMapLive.Controllers.HealthController do
  use QueueMapLive, :controller

  def index(conn, _params) do
    json(conn, %{ok: true, service: "queue_map_live"})
  end
end
