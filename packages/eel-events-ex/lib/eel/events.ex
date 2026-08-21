defmodule Eel.Events do
  @moduledoc """
  CloudEvents 1.0 envelope implementation for the IEBC IEMS project.

  Provides the following sub-modules:

  - `Eel.Events.Envelope` - struct with core + IEBC extension attributes,
    plus `from_map/1`, `to_map/1`, `validate/1`.
  - `Eel.Events.Builder` - fluent construction of envelopes.
  - `Eel.Events.CanonicalJson` - RFC 8785 canonical JSON (key-sorted, no
    whitespace, shortest-round-trip numbers).
  - `Eel.Events.Signing` - Ed25519 sign/verify using OTP `:crypto`.
  - `Eel.Events.Karapace` - async Karapace schema registry client (Finch).
  - `Eel.Events.Consumer` - behaviour for event handler modules.
  """
end
