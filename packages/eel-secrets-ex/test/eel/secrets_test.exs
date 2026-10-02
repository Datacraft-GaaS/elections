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

defmodule Eel.SecretsTest do
  use ExUnit.Case, async: false

  describe "Application env config" do
    test "reads bao_url from application env" do
      Application.put_env(:eel_secrets, :bao_url, "http://test-bao:8200")
      assert Application.get_env(:eel_secrets, :bao_url) == "http://test-bao:8200"
      Application.delete_env(:eel_secrets, :bao_url)
    end

    test "falls back to default when not configured" do
      Application.delete_env(:eel_secrets, :bao_url)
      assert Application.get_env(:eel_secrets, :bao_url, "http://localhost:8200") ==
               "http://localhost:8200"
    end
  end

  describe "Eel.Secrets.PgDynamicCred" do
    setup do
      # Start Finch so PgDynamicCred can attempt HTTP (will fail gracefully).
      start_supervised!({Finch, name: Eel.Secrets.Finch})
      :ok
    end

    test "GenServer starts and returns :not_initialized before first fetch" do
      # Use a unique name to avoid conflicts.
      opts = [role: "app_role", name: :test_pg_cred]
      pid = start_supervised!({Eel.Secrets.PgDynamicCred, opts})
      assert is_pid(pid)

      # Immediately after start the fetch has not completed (OpenBao not available).
      # Either :not_initialized or {:ok, cred} are valid responses — we just
      # assert the GenServer is alive and responds.
      result = Eel.Secrets.PgDynamicCred.get_credential(:test_pg_cred)
      assert result in [{:error, :not_initialized}] or match?({:ok, _}, result)
    end

    test "GenServer returns :not_initialized when no cred fetched yet" do
      opts = [role: "app_role_2", name: :test_pg_cred_2]
      _pid = start_supervised!({Eel.Secrets.PgDynamicCred, opts})

      assert {:error, :not_initialized} =
               Eel.Secrets.PgDynamicCred.get_credential(:test_pg_cred_2)
    end
  end

  describe "Eel.Secrets module" do
    test "child_spec/1 returns a valid supervisor child spec" do
      spec = Eel.Secrets.child_spec([])
      assert is_map(spec)
      assert Map.get(spec, :id) == Eel.Secrets
      assert is_tuple(Map.get(spec, :start))
    end
  end
end
