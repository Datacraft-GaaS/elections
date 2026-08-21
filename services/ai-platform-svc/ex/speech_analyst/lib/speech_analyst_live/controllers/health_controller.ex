defmodule SpeechAnalystLive.Controllers.HealthController do
  use SpeechAnalystLive, :controller

  def index(conn, _params) do
    json(conn, %{ok: true, service: "speech_analyst_live"})
  end
end
