defmodule Eel.Secrets.PgDynamicCred do
  @moduledoc """
  GenServer that holds a live PostgreSQL dynamic credential from OpenBao.

  Automatically schedules renewal at `lease_renewal_ratio * TTL` so the
  credential stays valid without manual intervention.

  ## Options

  - `role` - OpenBao database role name (required)
  - `mount` - secret engine mount path (default: `"database"`)
  - `lease_renewal_ratio` - renew when this fraction of TTL has elapsed (default: `0.8`)
  """

  use GenServer

  require Logger

  @type opts :: [
          role: String.t(),
          mount: String.t(),
          lease_renewal_ratio: float()
        ]

  defstruct [:role, :mount, :lease_renewal_ratio, :credential, :lease_id, :lease_duration]

  # Client API

  @doc """
  Starts the dynamic credential GenServer.
  """
  @spec start_link(opts()) :: GenServer.on_start()
  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts, name: Keyword.get(opts, :name, __MODULE__))
  end

  @doc """
  Returns the current credential.

  Returns `{:ok, %{username: String.t(), password: String.t()}}` or
  `{:error, :not_initialized}` before the first fetch completes.
  """
  @spec get_credential(GenServer.server()) ::
          {:ok, %{username: String.t(), password: String.t()}} | {:error, :not_initialized}
  def get_credential(server \\ __MODULE__) do
    GenServer.call(server, :get_credential)
  end

  @doc """
  Forces an immediate lease renewal.
  """
  @spec renew(GenServer.server()) :: :ok | {:error, term()}
  def renew(server \\ __MODULE__) do
    GenServer.call(server, :renew)
  end

  # Server callbacks

  @impl true
  def init(opts) do
    role = Keyword.fetch!(opts, :role)
    mount = Keyword.get(opts, :mount, "database")
    ratio = Keyword.get(opts, :lease_renewal_ratio, 0.8)

    state = %__MODULE__{
      role: role,
      mount: mount,
      lease_renewal_ratio: ratio,
      credential: nil,
      lease_id: nil,
      lease_duration: nil
    }

    send(self(), :fetch)
    {:ok, state}
  end

  @impl true
  def handle_call(:get_credential, _from, %{credential: nil} = state) do
    {:reply, {:error, :not_initialized}, state}
  end

  def handle_call(:get_credential, _from, %{credential: cred} = state) do
    {:reply, {:ok, cred}, state}
  end

  def handle_call(:renew, _from, state) do
    case do_renew(state) do
      {:ok, new_state} -> {:reply, :ok, new_state}
      {:error, reason} -> {:reply, {:error, reason}, state}
    end
  end

  @impl true
  def handle_info(:fetch, state) do
    path = "#{state.mount}/creds/#{state.role}"

    case fetch_credential(path) do
      {:ok, cred, lease_id, lease_duration} ->
        schedule_renewal(state.lease_renewal_ratio, lease_duration)

        new_state = %{
          state
          | credential: cred,
            lease_id: lease_id,
            lease_duration: lease_duration
        }

        {:noreply, new_state}

      {:error, reason} ->
        Logger.error("[Eel.Secrets.PgDynamicCred] Initial fetch failed: #{inspect(reason)}")
        Process.send_after(self(), :fetch, 5_000)
        {:noreply, state}
    end
  end

  def handle_info(:renew, state) do
    case do_renew(state) do
      {:ok, new_state} ->
        {:noreply, new_state}

      {:error, reason} ->
        Logger.warning("[Eel.Secrets.PgDynamicCred] Renewal failed: #{inspect(reason)}, refetching")
        send(self(), :fetch)
        {:noreply, state}
    end
  end

  defp fetch_credential(path) do
    url = "#{bao_url()}/v1/#{path}"
    headers = [{"x-vault-token", token()}, {"accept", "application/json"}]
    request = Finch.build(:get, url, headers)

    case Finch.request(request, Eel.Secrets.Finch) do
      {:ok, %Finch.Response{status: 200, body: body}} ->
        case Jason.decode(body) do
          {:ok, %{"data" => %{"username" => user, "password" => pass}, "lease_id" => lease_id, "lease_duration" => lease_duration}} ->
            {:ok, %{username: user, password: pass}, lease_id, lease_duration}

          {:ok, _other} ->
            {:error, :unexpected_response}

          {:error, reason} ->
            {:error, reason}
        end

      {:ok, %Finch.Response{status: status, body: body}} ->
        {:error, {:http_error, status, body}}

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp do_renew(%{lease_id: nil}), do: {:error, :no_lease}

  defp do_renew(state) do
    url = "#{bao_url()}/v1/sys/leases/renew"
    body = %{"lease_id" => state.lease_id}
    headers = [{"x-vault-token", token()}, {"content-type", "application/json"}]

    case Jason.encode(body) do
      {:ok, json} ->
        request = Finch.build(:put, url, headers, json)

        case Finch.request(request, Eel.Secrets.Finch) do
          {:ok, %Finch.Response{status: status, body: resp}} when status in 200..299 ->
            case Jason.decode(resp) do
              {:ok, %{"lease_duration" => new_duration}} ->
                schedule_renewal(state.lease_renewal_ratio, new_duration)
                {:ok, %{state | lease_duration: new_duration}}

              _ ->
                {:ok, state}
            end

          {:ok, %Finch.Response{status: status, body: resp}} ->
            {:error, {:http_error, status, resp}}

          {:error, reason} ->
            {:error, reason}
        end

      {:error, reason} ->
        {:error, {:encode_error, reason}}
    end
  end

  defp schedule_renewal(ratio, lease_duration_seconds) do
    ms = trunc(lease_duration_seconds * ratio * 1000)
    Process.send_after(self(), :renew, ms)
  end

  defp bao_url, do: Application.get_env(:eel_secrets, :bao_url, "http://localhost:8200")
  defp token, do: Application.get_env(:eel_secrets, :token, "")
end
