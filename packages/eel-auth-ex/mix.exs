defmodule Eel.Auth.MixProject do
  use Mix.Project

  def project do
    [
      app: :eel_auth,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      elixirc_options: [warnings_as_errors: true]
    ]
  end

  def application do
    [
      extra_applications: [:logger, :public_key]
    ]
  end

  defp deps do
    [
      {:finch, "~> 0.19"},
      {:jason, "~> 1.4"},
      {:plug, "~> 1.17"},
      {:phoenix_live_view, "~> 1.0", optional: true}
    ]
  end
end
