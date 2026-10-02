defmodule EelPii.MixProject do
  use Mix.Project

  def project do
    [
      app: :eel_pii,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "PII data class tiers and Ecto types for the Commission IEMS (ADR-0016)"
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:ecto, "~> 3.12", optional: true}
    ]
  end
end
