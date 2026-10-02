defmodule EelEvents.MixProject do
  use Mix.Project

  def project do
    [
      app: :eel_events,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "CloudEvents envelope with the Commission IEMS extensions",
      elixirc_options: [warnings_as_errors: true]
    ]
  end

  def application do
    [extra_applications: [:logger, :crypto, :public_key]]
  end

  defp deps do
    [
      {:finch, "~> 0.19"},
      {:jason, "~> 1.4"},
      {:broadway, "~> 1.1"},
      {:ecto, "~> 3.12", optional: true}
    ]
  end
end
