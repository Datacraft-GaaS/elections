defmodule Eel.EventStore.MixProject do
  use Mix.Project

  def project do
    [
      app: :eel_event_store,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      elixirc_options: [warnings_as_errors: true]
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:ecto, "~> 3.12"},
      {:jason, "~> 1.4"},
      {:ecto_sql, "~> 3.12"}
    ]
  end
end
