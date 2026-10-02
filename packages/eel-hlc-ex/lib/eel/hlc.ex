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

defmodule Eel.Hlc do
  @moduledoc """
  Hybrid Logical Clock (HLC) implementation.

  An HLC combines a physical wall-clock timestamp with a monotonic counter
  and a node identifier. This allows events to be totally ordered across
  distributed nodes while staying close to real time.

  Format: "<physical_ms>.<counter>.<node>"
  Example: "1723731000123.5.42"
  """

  @enforce_keys [:physical_ms, :counter, :node]
  defstruct [:physical_ms, :counter, :node]

  @type t :: %__MODULE__{
          physical_ms: non_neg_integer(),
          counter: non_neg_integer(),
          node: String.t()
        }

  @doc "Create a new HLC struct, validating inputs."
  @spec new(non_neg_integer(), non_neg_integer(), String.t()) :: t()
  def new(physical_ms, counter, node)
      when is_integer(physical_ms) and physical_ms >= 0 and
             is_integer(counter) and counter >= 0 and
             is_binary(node) do
    %__MODULE__{physical_ms: physical_ms, counter: counter, node: node}
  end

  @doc "Parse a dot-separated HLC string. Returns {:ok, hlc} or {:error, reason}."
  @spec from_string(String.t()) :: {:ok, t()} | {:error, String.t()}
  def from_string(str) when is_binary(str) do
    case String.split(str, ".") do
      [phys_str, ctr_str, node] ->
        with {phys, ""} <- Integer.parse(phys_str),
             true <- phys >= 0,
             {ctr, ""} <- Integer.parse(ctr_str),
             true <- ctr >= 0,
             true <- byte_size(node) > 0 do
          {:ok, new(phys, ctr, node)}
        else
          _ -> {:error, "invalid hlc fields in: #{str}"}
        end

      _ ->
        {:error, "expected exactly 3 dot-separated segments, got: #{str}"}
    end
  end

  def from_string(other), do: {:error, "expected string, got: #{inspect(other)}"}

  @doc "Format an HLC struct as its canonical dot-separated string."
  @spec to_string(t()) :: String.t()
  def to_string(%__MODULE__{physical_ms: p, counter: c, node: n}) do
    "#{p}.#{c}.#{n}"
  end

  @doc """
  Merge two HLC values (local and received) per the HLC algorithm.

  - If physical times differ, keep the higher physical time and reset counter.
  - If physical times are equal, use max(counter) + 1.
  - Node is taken from the local clock.
  """
  @spec merge(t(), t()) :: t()
  def merge(%__MODULE__{} = local, %__MODULE__{} = received) do
    cond do
      local.physical_ms > received.physical_ms ->
        %{local | counter: local.counter + 1}

      received.physical_ms > local.physical_ms ->
        %{local | physical_ms: received.physical_ms, counter: received.counter + 1}

      true ->
        # same physical time: keep higher counter + 1
        %{local | counter: max(local.counter, received.counter) + 1}
    end
  end

  @doc """
  Advance the clock with a new wall-clock reading.

  If wall_ms is ahead of physical_ms, advance to wall_ms and reset counter.
  Otherwise increment the counter (clock skew or same-millisecond event).
  """
  @spec tick(t(), non_neg_integer()) :: t()
  def tick(%__MODULE__{} = hlc, wall_ms) when is_integer(wall_ms) do
    if wall_ms > hlc.physical_ms do
      %{hlc | physical_ms: wall_ms, counter: 0}
    else
      %{hlc | counter: hlc.counter + 1}
    end
  end

  @doc "Compare two HLC values. Returns :lt, :eq, or :gt."
  @spec compare(t(), t()) :: :lt | :eq | :gt
  def compare(%__MODULE__{} = a, %__MODULE__{} = b) do
    cond do
      a.physical_ms < b.physical_ms -> :lt
      a.physical_ms > b.physical_ms -> :gt
      a.counter < b.counter -> :lt
      a.counter > b.counter -> :gt
      a.node < b.node -> :lt
      a.node > b.node -> :gt
      true -> :eq
    end
  end
end
