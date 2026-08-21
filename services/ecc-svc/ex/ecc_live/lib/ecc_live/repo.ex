defmodule EccLive.Repo do
  use Ecto.Repo,
    otp_app: :ecc_live,
    adapter: Ecto.Adapters.Postgres
end
