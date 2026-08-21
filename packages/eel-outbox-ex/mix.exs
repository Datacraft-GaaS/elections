defmodule EelOutbox.MixProject do
  use Mix.Project

  def project do
    [
      app: :eel_outbox,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "Transactional outbox pattern for IEBC IEMS event publishing",
      elixirc_options: [warnings_as_errors: true]
    ]
  end

  def application do
    [extra_applications: [:logger]]
  end

  defp deps do
    [
      {:ecto, "~> 3.12"},
      {:jason, "~> 1.4"},
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:finch, "~> 0.19", optional: true}
    ]
  end
end
