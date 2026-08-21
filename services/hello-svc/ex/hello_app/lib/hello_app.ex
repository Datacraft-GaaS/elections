defmodule HelloApp do
  @moduledoc "Smoke-test hello application for toolchain validation."

  @spec greeting() :: map()
  def greeting, do: %{ok: true}
end
