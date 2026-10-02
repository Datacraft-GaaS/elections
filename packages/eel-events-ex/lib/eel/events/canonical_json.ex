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

defmodule Eel.Events.CanonicalJson do
  @moduledoc """
  RFC 8785 canonical JSON encoding.

  Rules:
  - Object keys sorted lexicographically by Unicode code point (BMP-safe).
  - Recursion: nested objects and arrays apply the same rules.
  - No insignificant whitespace.
  - Strings are JSON-escaped per RFC 8259.
  - Integers are rendered without decimal point; floats use shortest round-trip.
  """

  @doc """
  Encode `value` as RFC 8785 canonical JSON.

  Returns `{:ok, binary}` or `{:error, reason}`.
  """
  @spec encode(term()) :: {:ok, binary()} | {:error, term()}
  def encode(value) do
    {:ok, do_encode(value)}
  rescue
    e -> {:error, Exception.message(e)}
  end

  defp do_encode(nil), do: "null"
  defp do_encode(true), do: "true"
  defp do_encode(false), do: "false"

  defp do_encode(n) when is_integer(n), do: Integer.to_string(n)

  defp do_encode(f) when is_float(f) do
    # Jason uses Ryu algorithm for shortest round-trip IEEE 754 representation.
    Jason.encode!(f)
  end

  defp do_encode(s) when is_binary(s) do
    Jason.encode!(s)
  end

  defp do_encode(list) when is_list(list) do
    "[" <> Enum.map_join(list, ",", &do_encode/1) <> "]"
  end

  defp do_encode(%{} = map) do
    # Strip struct metadata if present, then sort by key (Unicode code point order).
    plain =
      if is_struct(map) do
        map |> Map.from_struct() |> Enum.reject(fn {_k, v} -> is_nil(v) end) |> Map.new()
      else
        map
      end

    inner =
      plain
      |> Enum.sort_by(fn {k, _} -> to_string(k) end)
      |> Enum.map_join(",", fn {k, v} ->
        Jason.encode!(to_string(k)) <> ":" <> do_encode(v)
      end)

    "{" <> inner <> "}"
  end
end
