defmodule Eel.Events.Karapace do
  @moduledoc """
  Async HTTP client for the Karapace schema registry.

  Add to your supervision tree:

      children = [
        {Eel.Events.Karapace, []}
      ]

  Configure the base URL via application environment:

      config :eel_events, :karapace_url, "https://karapace.example.com"
  """

  @finch_name __MODULE__.Finch

  @doc """
  Returns a child spec that starts a dedicated Finch pool for Karapace HTTP
  requests. Pass Finch pool options in `opts` (e.g. `pools: %{...}`).
  """
  @spec child_spec(keyword()) :: Supervisor.child_spec()
  def child_spec(opts) do
    finch_opts = Keyword.merge([name: @finch_name], opts)
    Finch.child_spec(finch_opts)
  end

  @doc """
  Fetch a schema from Karapace.

  `version` may be `:latest` (default) or an integer schema version.

  Returns `{:ok, schema_json}` or `{:error, reason}`.
  """
  @spec lookup_schema(String.t(), :latest | pos_integer()) ::
          {:ok, String.t()} | {:error, term()}
  def lookup_schema(subject, version \\ :latest) when is_binary(subject) do
    base_url = Application.get_env(:eel_events, :karapace_url, "http://localhost:8081")
    version_str = if version == :latest, do: "latest", else: Integer.to_string(version)
    url = "#{base_url}/subjects/#{URI.encode(subject)}/versions/#{version_str}"

    request = Finch.build(:get, url)

    case Finch.request(request, @finch_name) do
      {:ok, %Finch.Response{status: 200, body: body}} ->
        {:ok, body}

      {:ok, %Finch.Response{status: status, body: body}} ->
        {:error, {:http_error, status, body}}

      {:error, reason} ->
        {:error, reason}
    end
  end
end
