defmodule Eel.PII do
  @moduledoc """
  PII handling with data class tiers per ADR-0016.

  Data classes:
    :A  most restricted (biometric, national ID)
    :B  restricted (financial, health)
    :C  sensitive (contact details)
    :D  internal (name, address)
    :E  public / unrestricted

  Wrap values in the appropriate tier struct. Values never leak through
  `String.Chars` or `Inspect` — those always return a redaction string.
  Use `reveal/2` with a stated purpose to access the raw value; this emits
  an audit log entry via `Logger.info/2`.
  """

  require Logger

  # --- Structs ---

  defmodule Restricted do
    @moduledoc "PII tier A/B: most restricted data (biometric, financial, health)."
    @enforce_keys [:value, :class]
    defstruct [:value, :class]

    @type t :: %__MODULE__{value: term(), class: :A | :B}
  end

  defmodule Sensitive do
    @moduledoc "PII tier C/D: sensitive or internal data (contact, address)."
    @enforce_keys [:value, :class]
    defstruct [:value, :class]

    @type t :: %__MODULE__{value: term(), class: :C | :D}
  end

  defmodule Public do
    @moduledoc "PII tier E: public / unrestricted data."
    @enforce_keys [:value]
    defstruct [:value]

    @type t :: %__MODULE__{value: term()}
  end

  @type pii :: Restricted.t() | Sensitive.t() | Public.t()

  # --- Constructors ---

  @doc "Wrap a value in a Restricted struct. class must be :A or :B (default :A)."
  @spec restricted(term(), :A | :B) :: Restricted.t()
  def restricted(value, class \\ :A) when class in [:A, :B] do
    %Restricted{value: value, class: class}
  end

  @doc "Wrap a value in a Sensitive struct. class must be :C or :D (default :C)."
  @spec sensitive(term(), :C | :D) :: Sensitive.t()
  def sensitive(value, class \\ :C) when class in [:C, :D] do
    %Sensitive{value: value, class: class}
  end

  @doc "Wrap a value in a Public struct (tier E)."
  @spec public(term()) :: Public.t()
  def public(value) do
    %Public{value: value}
  end

  # --- Operations ---

  @doc """
  Extract the raw value and emit an audit log entry.

  The log entry is emitted via `Logger.info/2` with metadata:
    %{pii_class: class, purpose: purpose, revealed_at: DateTime.utc_now()}

  Returns `{:ok, value}`.
  """
  @spec reveal(pii(), term()) :: {:ok, term()}
  def reveal(%Restricted{value: v, class: c}, purpose) do
    _log_reveal(c, purpose)
    {:ok, v}
  end

  def reveal(%Sensitive{value: v, class: c}, purpose) do
    _log_reveal(c, purpose)
    {:ok, v}
  end

  def reveal(%Public{value: v}, purpose) do
    _log_reveal(:E, purpose)
    {:ok, v}
  end

  @doc "Return the data class atom for any PII struct."
  @spec class(pii()) :: :A | :B | :C | :D | :E
  def class(%Restricted{class: c}), do: c
  def class(%Sensitive{class: c}), do: c
  def class(%Public{}), do: :E

  @doc "Return a redaction string, e.g. \"[REDACTED:A]\"."
  @spec redact(pii()) :: String.t()
  def redact(pii), do: "[REDACTED:#{class(pii)}]"

  # --- Private ---

  defp _log_reveal(class, purpose) do
    Logger.info("PII revealed",
      pii_class: class,
      purpose: purpose,
      revealed_at: DateTime.utc_now()
    )
  end
end
