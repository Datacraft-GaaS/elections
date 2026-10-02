# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

defmodule Eel.Events do
  @moduledoc """
  CloudEvents 1.0 envelope implementation for the Commission IEMS project.

  Provides the following sub-modules:

  - `Eel.Events.Envelope` - struct with core + the Commission extension attributes,
    plus `from_map/1`, `to_map/1`, `validate/1`.
  - `Eel.Events.Builder` - fluent construction of envelopes.
  - `Eel.Events.CanonicalJson` - RFC 8785 canonical JSON (key-sorted, no
    whitespace, shortest-round-trip numbers).
  - `Eel.Events.Signing` - Ed25519 sign/verify using OTP `:crypto`.
  - `Eel.Events.Karapace` - async Karapace schema registry client (Finch).
  - `Eel.Events.Consumer` - behaviour for event handler modules.
  """
end
