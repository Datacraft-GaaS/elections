defmodule EelObservability.MixProject do
  use Mix.Project

  def project do
    [
      app: :eel_observability,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:opentelemetry_api, "~> 1.4"},
      {:opentelemetry, "~> 1.4"},
      {:opentelemetry_exporter, "~> 1.8"},
      {:jason, "~> 1.4"}
    ]
  end
end
