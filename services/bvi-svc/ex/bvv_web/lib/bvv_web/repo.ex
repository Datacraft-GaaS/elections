defmodule BvvWeb.Repo do
  use Ecto.Repo,
    otp_app: :bvv_web,
    adapter: Ecto.Adapters.Postgres
end
