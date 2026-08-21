defmodule EelHlc.MixProject do
  use Mix.Project

  def project do
    [
      app: :eel_hlc,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "Hybrid Logical Clock for IEBC IEMS distributed systems"
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    []
  end
end
