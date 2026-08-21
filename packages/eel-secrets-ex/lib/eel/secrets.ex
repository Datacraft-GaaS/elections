defmodule Eel.Secrets do
  @moduledoc """
  OpenBao (Vault-compatible) secrets client for IEBC IEMS services.

  Provides KV v2 secret reads and Transit encrypt/decrypt operations.
  All HTTP calls go through Finch (`Eel.Secrets.Finch`).

  ## Configuration

      config :eel_secrets,
        bao_url: "http://localhost:8200",
        token: "your-openbao-token"
  """

  @default_url "http://localhost:8200"

  @doc """
  Returns a child spec that starts the Finch pool used by this library.

  Include in your application supervisor alongside other children.
  """
  @spec child_spec(keyword()) :: Supervisor.child_spec()
  def child_spec(opts) do
    %{
      id: __MODULE__,
      start: {Finch, :start_link, [[name: Eel.Secrets.Finch] ++ opts]},
      type: :worker,
      restart: :permanent,
      shutdown: 5000
    }
  end

  @doc """
  Reads a KV v2 secret at `path` from OpenBao.

  Returns `{:ok, data_map}` where `data_map` is the `data` field of the
  KV v2 response, or `{:error, reason}`.
  """
  @spec read(String.t(), keyword()) :: {:ok, map()} | {:error, term()}
  def read(path, _opts \\ []) do
    url = "#{bao_url()}/v1/secret/data/#{path}"
    get(url)
  end

  @doc """
  Encrypts `plaintext` using the Transit engine key `key_name`.

  Returns `{:ok, ciphertext}` or `{:error, reason}`.
  """
  @spec encrypt(String.t(), String.t()) :: {:ok, String.t()} | {:error, term()}
  def encrypt(plaintext, key_name) do
    url = "#{bao_url()}/v1/transit/encrypt/#{key_name}"
    b64 = Base.encode64(plaintext)
    body = %{"plaintext" => b64}

    case post(url, body) do
      {:ok, %{"data" => %{"ciphertext" => ct}}} -> {:ok, ct}
      {:ok, _} -> {:error, :unexpected_response}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Decrypts `ciphertext` using the Transit engine key `key_name`.

  Returns `{:ok, plaintext}` or `{:error, reason}`.
  """
  @spec decrypt(String.t(), String.t()) :: {:ok, String.t()} | {:error, term()}
  def decrypt(ciphertext, key_name) do
    url = "#{bao_url()}/v1/transit/decrypt/#{key_name}"
    body = %{"ciphertext" => ciphertext}

    case post(url, body) do
      {:ok, %{"data" => %{"plaintext" => b64}}} ->
        case Base.decode64(b64) do
          {:ok, plaintext} -> {:ok, plaintext}
          :error -> {:error, :invalid_base64}
        end

      {:ok, _} ->
        {:error, :unexpected_response}

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp get(url) do
    headers = [{"x-vault-token", token()}, {"accept", "application/json"}]
    request = Finch.build(:get, url, headers)

    case Finch.request(request, Eel.Secrets.Finch) do
      {:ok, %Finch.Response{status: 200, body: body}} ->
        case Jason.decode(body) do
          {:ok, %{"data" => %{"data" => data}}} -> {:ok, data}
          {:ok, other} -> {:ok, other}
          {:error, reason} -> {:error, reason}
        end

      {:ok, %Finch.Response{status: 404}} ->
        {:error, :not_found}

      {:ok, %Finch.Response{status: status, body: body}} ->
        {:error, {:http_error, status, body}}

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp post(url, body) do
    headers = [
      {"x-vault-token", token()},
      {"content-type", "application/json"},
      {"accept", "application/json"}
    ]

    case Jason.encode(body) do
      {:ok, json} ->
        request = Finch.build(:post, url, headers, json)

        case Finch.request(request, Eel.Secrets.Finch) do
          {:ok, %Finch.Response{status: status, body: resp}} when status in 200..299 ->
            Jason.decode(resp)

          {:ok, %Finch.Response{status: status, body: resp}} ->
            {:error, {:http_error, status, resp}}

          {:error, reason} ->
            {:error, reason}
        end

      {:error, reason} ->
        {:error, {:encode_error, reason}}
    end
  end

  defp bao_url, do: Application.get_env(:eel_secrets, :bao_url, @default_url)
  defp token, do: Application.get_env(:eel_secrets, :token, "")
end
