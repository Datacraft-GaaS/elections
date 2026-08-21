defmodule PortalLive.MixProject do
  use Mix.Project

  def project do
    [
      app: :portal_live,
      version: "0.1.0",
      elixir: "~> 1.17",
      elixirc_paths: elixirc_paths(Mix.env()),
      start_permanent: Mix.env() == :prod,
      aliases: aliases(),
      deps: deps(),
    ]
  end

  def application do
    [
      extra_applications: [:logger, :runtime_tools],
      mod: {PortalLive.Application, []},
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]

  defp deps do
    [
      {:phoenix, "~> 1.8"},
      {:phoenix_live_view, "~> 1.2"},
      {:phoenix_pubsub, "~> 2.2"},
      {:plug, "~> 1.20"},
      {:jason, "~> 1.4"},
    ]
  end

  defp aliases do
    [
      setup: ["deps.get"],
    ]
  end
end
