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

defmodule Eel.PIITest do
  use ExUnit.Case, async: true

  alias Eel.PII
  alias Eel.PII.{Restricted, Sensitive, Public}

  # --- constructors ---

  test "restricted/2 wraps in Restricted with class :A by default" do
    r = PII.restricted("id-number")
    assert %Restricted{value: "id-number", class: :A} = r
  end

  test "restricted/2 accepts class :B" do
    r = PII.restricted("account-balance", :B)
    assert r.class == :B
  end

  test "restricted/2 rejects invalid class" do
    bad = List.first([:C])
    assert_raise FunctionClauseError, fn -> PII.restricted("x", bad) end
  end

  test "sensitive/2 wraps in Sensitive with class :C by default" do
    s = PII.sensitive("email@example.com")
    assert %Sensitive{value: "email@example.com", class: :C} = s
  end

  test "sensitive/2 accepts class :D" do
    s = PII.sensitive("123 Main St", :D)
    assert s.class == :D
  end

  test "sensitive/2 rejects invalid class" do
    bad = List.first([:A])
    assert_raise FunctionClauseError, fn -> PII.sensitive("x", bad) end
  end

  test "public/1 wraps in Public" do
    p = PII.public("visible value")
    assert %Public{value: "visible value"} = p
  end

  # --- class/1 ---

  test "class/1 returns correct atom for each tier" do
    assert PII.class(PII.restricted("v", :A)) == :A
    assert PII.class(PII.restricted("v", :B)) == :B
    assert PII.class(PII.sensitive("v", :C)) == :C
    assert PII.class(PII.sensitive("v", :D)) == :D
    assert PII.class(PII.public("v")) == :E
  end

  # --- redact/1 ---

  test "redact/1 returns correct strings" do
    assert PII.redact(PII.restricted("v", :A)) == "[REDACTED:A]"
    assert PII.redact(PII.restricted("v", :B)) == "[REDACTED:B]"
    assert PII.redact(PII.sensitive("v", :C)) == "[REDACTED:C]"
    assert PII.redact(PII.sensitive("v", :D)) == "[REDACTED:D]"
    assert PII.redact(PII.public("v")) == "[REDACTED:E]"
  end

  # --- reveal/2 ---

  test "reveal/2 returns {:ok, value} for Restricted" do
    r = PII.restricted("secret-id")
    assert {:ok, "secret-id"} = PII.reveal(r, "audit-test")
  end

  test "reveal/2 returns {:ok, value} for Sensitive" do
    s = PII.sensitive("user@example.com")
    assert {:ok, "user@example.com"} = PII.reveal(s, "email-send")
  end

  test "reveal/2 returns {:ok, value} for Public" do
    p = PII.public("Kenya")
    assert {:ok, "Kenya"} = PII.reveal(p, "display")
  end

  # --- String.Chars protocol ---

  test "to_string/1 never leaks value for Restricted" do
    r = PII.restricted("biometric-data")
    refute Kernel.to_string(r) =~ "biometric-data"
    assert Kernel.to_string(r) =~ "REDACTED"
  end

  test "to_string/1 never leaks value for Sensitive" do
    s = PII.sensitive("email@secret.io")
    refute Kernel.to_string(s) =~ "email@secret.io"
    assert Kernel.to_string(s) =~ "REDACTED"
  end

  test "to_string/1 never leaks value for Public" do
    p = PII.public("Nairobi")
    refute Kernel.to_string(p) =~ "Nairobi"
    assert Kernel.to_string(p) =~ "REDACTED"
  end

  # --- Inspect protocol ---

  test "inspect/1 never leaks value for Restricted" do
    r = PII.restricted("biometric")
    refute inspect(r) =~ "biometric"
    assert inspect(r) =~ "REDACTED"
  end

  test "inspect/1 never leaks value for Sensitive" do
    s = PII.sensitive("phone-number")
    refute inspect(s) =~ "phone-number"
    assert inspect(s) =~ "REDACTED"
  end

  test "inspect/1 never leaks value for Public" do
    p = PII.public("public-val")
    refute inspect(p) =~ "public-val"
    assert inspect(p) =~ "REDACTED"
  end
end
