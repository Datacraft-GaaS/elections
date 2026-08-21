defmodule ObserverWorkspaceWeb.HealthController do
  use ObserverWorkspaceWeb, :controller

  def index(conn, _params) do
    json(conn, %{status: "ok", service: "eel-observer-workspace"})
  end
end
