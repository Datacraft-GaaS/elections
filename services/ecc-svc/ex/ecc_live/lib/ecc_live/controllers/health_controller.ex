defmodule EccLive.Controllers.HealthController do
  use EccLive, :controller

  def index(conn, _params) do
    json(conn, %{status: "ok", service: "ecc_live"})
  end
end
