if Code.ensure_loaded?(Ecto.Type) do
  defmodule Eel.PII.EctoType.Restricted do
    @moduledoc """
    Ecto type for `Eel.PII.Restricted`.

    - `cast/1` and `load/1` wrap the value in a Restricted struct (class :A by default).
    - `dump/1` is forbidden for class :A and :B: raw PII must not reach the DB layer directly.
    """

    use Ecto.Type

    alias Eel.PII

    @impl true
    def type, do: :string

    @impl true
    def cast(value) when is_binary(value), do: {:ok, PII.restricted(value)}
    def cast(%PII.Restricted{} = r), do: {:ok, r}
    def cast(_), do: :error

    @impl true
    def load(value) when is_binary(value), do: {:ok, PII.restricted(value)}
    def load(%PII.Restricted{} = r), do: {:ok, r}
    def load(_), do: :error

    @impl true
    def dump(%PII.Restricted{}), do: {:error, :pii_dump_forbidden}
    def dump(_), do: :error
  end

  defmodule Eel.PII.EctoType.Sensitive do
    @moduledoc """
    Ecto type for `Eel.PII.Sensitive`.

    - `cast/1` and `load/1` wrap the value in a Sensitive struct (class :C by default).
    - `dump/1` is allowed for class :C and :D — the raw string is stored.
    """

    use Ecto.Type

    alias Eel.PII

    @impl true
    def type, do: :string

    @impl true
    def cast(value) when is_binary(value), do: {:ok, PII.sensitive(value)}
    def cast(%PII.Sensitive{} = s), do: {:ok, s}
    def cast(_), do: :error

    @impl true
    def load(value) when is_binary(value), do: {:ok, PII.sensitive(value)}
    def load(%PII.Sensitive{} = s), do: {:ok, s}
    def load(_), do: :error

    @impl true
    def dump(%PII.Sensitive{value: v}) when is_binary(v), do: {:ok, v}
    def dump(_), do: :error
  end

  defmodule Eel.PII.EctoType.Public do
    @moduledoc """
    Ecto type for `Eel.PII.Public`.

    - `cast/1` and `load/1` wrap the value in a Public struct.
    - `dump/1` is allowed — tier E data may reach the DB layer.
    """

    use Ecto.Type

    alias Eel.PII

    @impl true
    def type, do: :string

    @impl true
    def cast(value) when is_binary(value), do: {:ok, PII.public(value)}
    def cast(%PII.Public{} = p), do: {:ok, p}
    def cast(_), do: :error

    @impl true
    def load(value) when is_binary(value), do: {:ok, PII.public(value)}
    def load(%PII.Public{} = p), do: {:ok, p}
    def load(_), do: :error

    @impl true
    def dump(%PII.Public{value: v}) when is_binary(v), do: {:ok, v}
    def dump(_), do: :error
  end
end
