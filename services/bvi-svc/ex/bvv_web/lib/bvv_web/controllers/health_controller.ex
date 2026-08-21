defmodule BvvWeb.HealthController do
  use BvvWeb, :controller

  def index(conn, _params) do
    json(conn, %{status: "ok", service: "bvv_web"})
  end
end
