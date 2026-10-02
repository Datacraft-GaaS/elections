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

defmodule Eel.AuthTest do
  use ExUnit.Case, async: false

  alias Eel.Auth.Subject

  describe "Eel.Auth.Subject" do
    test "constructs with required fields" do
      subject = %Subject{
        id: "user-123",
        realm: "iems",
        roles: ["voter"],
        raw_jwt: "eyJ.eyJ.sig"
      }

      assert subject.id == "user-123"
      assert subject.realm == "iems"
      assert subject.roles == ["voter"]
      assert subject.email == nil
      assert subject.svid == nil
    end

    test "constructs with all fields" do
      subject = %Subject{
        id: "svc-abc",
        realm: "iems",
        email: "svc@example.com",
        roles: ["admin"],
        svid: "spiffe://example.com/svc",
        raw_jwt: "tok"
      }

      assert subject.svid == "spiffe://example.com/svc"
      assert subject.email == "svc@example.com"
    end

    test "raises on missing required field" do
      assert_raise ArgumentError, fn ->
        struct!(Subject, %{id: "x", realm: "r", roles: []})
      end
    end
  end

  describe "JWT base64 decode helpers" do
    test "url-safe base64 decodes a known payload" do
      payload = %{"sub" => "user-1", "exp" => 9_999_999_999}
      encoded = payload |> Jason.encode!() |> Base.url_encode64(padding: false)
      {:ok, json} = Base.url_decode64(encoded, padding: false)
      assert Jason.decode!(json) == payload
    end

    test "decodes a split JWT structure" do
      header = %{"alg" => "RS256", "kid" => "key1"} |> Jason.encode!() |> Base.url_encode64(padding: false)
      payload = %{"sub" => "u"} |> Jason.encode!() |> Base.url_encode64(padding: false)
      token = "#{header}.#{payload}.fakesig"

      [h, p, s] = String.split(token, ".")
      assert {:ok, _} = Base.url_decode64(h, padding: false)
      assert {:ok, _} = Base.url_decode64(p, padding: false)
      assert s == "fakesig"
    end
  end

  describe "Eel.Auth.Plug with Plug.Test" do
    test "missing Authorization header returns 401" do
      opts = Eel.Auth.Plug.init(jwks: :test_jwks)
      conn = Plug.Test.conn(:get, "/")
      conn = Eel.Auth.Plug.call(conn, opts)

      assert conn.status == 401
      assert conn.halted
      body = Jason.decode!(conn.resp_body)
      assert body["status"] == 401
      assert body["title"] == "Unauthorized"
    end

    test "optional plug allows missing token" do
      opts = Eel.Auth.Plug.init(jwks: :test_jwks, optional: true)
      conn = Plug.Test.conn(:get, "/")
      conn = Eel.Auth.Plug.call(conn, opts)

      refute conn.halted
      assert conn.status == nil
    end
  end

  describe "Eel.Auth.JWKS ETS table" do
    test "ETS table is created on start" do
      # Use a unique name so tests are isolated.
      opts = [keycloak_url: "http://localhost:9999", realm: "test", ttl_ms: 1000]
      pid = start_supervised!({Eel.Auth.JWKS, opts})
      assert is_pid(pid)
      assert :ets.info(:eel_auth_jwks_cache) != :undefined
    end
  end
end
